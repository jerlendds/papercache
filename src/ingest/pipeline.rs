use sqlx::SqlitePool;
use std::io::Read;
use tokio::sync::broadcast;

use crate::{
    db::{
        assets::{AssetStore, upsert_cover_asset},
        chunks::replace_chunks,
        documents,
        models::DocumentStatus,
    },
    index::commands::IndexCommand,
    ingest::{chunker::chunk_pages, classifier, html, pdf, thumbnails},
    util::{hash::sha256_file, time::now_rfc3339},
    web::events::AppEvent,
};

pub async fn ingest_pdf(
    db: &SqlitePool,
    asset_store: &AssetStore,
    tantivy_tx: &tokio::sync::mpsc::Sender<IndexCommand>,
    event_tx: &broadcast::Sender<AppEvent>,
    document_id: &str,
) -> anyhow::Result<()> {
    let document = documents::get(db, document_id).await?;
    if document.status == DocumentStatus::Failed.as_str() {
        return Ok(());
    }

    let path = std::path::PathBuf::from(&document.path);
    if !path.exists() {
        sqlx::query("UPDATE documents SET status = 'missing', updated_at = ? WHERE id = ?")
            .bind(now_rfc3339())
            .bind(document_id)
            .execute(db)
            .await?;
        let _ = tantivy_tx
            .send(IndexCommand::DeleteDocument {
                document_id: document_id.to_string(),
            })
            .await;
        return Ok(());
    }

    if let Err(error) = validate_supported_file(&path) {
        let message = error.to_string();
        documents::mark_failed(db, document_id, &message).await?;
        anyhow::bail!(message);
    }

    let _ = event_tx.send(AppEvent::DocumentStage {
        document_id: document_id.to_string(),
        stage: "hash".to_string(),
    });
    let source_hash = tokio::task::spawn_blocking({
        let path = path.clone();
        move || sha256_file(&path)
    })
    .await??;

    sqlx::query(
        "UPDATE documents SET status = 'processing', sha256 = ?, updated_at = ? WHERE id = ?",
    )
    .bind(&source_hash)
    .bind(now_rfc3339())
    .bind(document_id)
    .execute(db)
    .await?;

    let _ = event_tx.send(AppEvent::DocumentStage {
        document_id: document_id.to_string(),
        stage: "extract_text".to_string(),
    });
    let extract_result = tokio::task::spawn_blocking({
        let path = path.clone();
        move || extract_text(&path)
    })
    .await;
    let extract = match extract_result {
        Ok(Ok(extract)) => extract,
        Ok(Err(error)) => {
            let message = error.to_string();
            documents::mark_failed(db, document_id, &message).await?;
            anyhow::bail!(message);
        }
        Err(error) => {
            let message = if error.is_panic() {
                format!("PDF text extraction panicked: {error}")
            } else {
                format!("PDF text extraction task failed: {error}")
            };
            documents::mark_failed(db, document_id, &message).await?;
            anyhow::bail!(message);
        }
    };

    let all_text = extract
        .pages
        .iter()
        .map(|page| page.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let doi = classifier::detect_doi(&all_text);
    let arxiv_id = classifier::detect_arxiv(&all_text);
    let title = extract
        .title
        .clone()
        .or_else(|| document.title.clone())
        .or_else(|| Some(document.file_name.clone()));
    let classification = classifier::classify(
        title.as_deref(),
        doi.as_deref(),
        arxiv_id.as_deref(),
        &all_text,
    );

    let chunks = chunk_pages(document_id, &source_hash, &extract.pages);
    replace_chunks(db, document_id, &chunks).await?;

    let cover_path = asset_store.cover_path(document_id, &source_hash);
    let relative_cover = AssetStore::relative_cover_path(document_id, &source_hash);
    let title_for_cover = title.as_deref().unwrap_or(&document.file_name).to_string();
    let _ = tokio::task::spawn_blocking(move || {
        thumbnails::write_placeholder_cover(&cover_path, &title_for_cover)
    })
    .await?;
    let cover_asset_id =
        upsert_cover_asset(db, document_id, &relative_cover, Some(&source_hash)).await?;

    let now = now_rfc3339();
    sqlx::query(
        r#"
        UPDATE documents
        SET title = ?, doi = ?, arxiv_id = ?, page_count = ?, status = ?,
            classification_json = ?, cover_asset_id = ?, error = NULL, updated_at = ?
        WHERE id = ?
        "#,
    )
    .bind(title)
    .bind(doi)
    .bind(arxiv_id)
    .bind(extract.page_count)
    .bind(DocumentStatus::Ready.as_str())
    .bind(serde_json::to_string(&classification)?)
    .bind(cover_asset_id)
    .bind(&now)
    .bind(document_id)
    .execute(db)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO document_processing_state
          (document_id, source_file_size, source_modified_at, source_sha256, text_extraction_version,
           thumbnail_version, classifier_version, chunker_version, updated_at)
        VALUES (?, ?, ?, ?, ?, 'placeholder-v1', 'rules-v1', 'chunker-v1', ?)
        ON CONFLICT(document_id) DO UPDATE SET
          source_file_size = excluded.source_file_size,
          source_modified_at = excluded.source_modified_at,
          source_sha256 = excluded.source_sha256,
          text_extraction_version = excluded.text_extraction_version,
          thumbnail_version = excluded.thumbnail_version,
          classifier_version = excluded.classifier_version,
          chunker_version = excluded.chunker_version,
          indexed_in_tantivy = 0,
          updated_at = excluded.updated_at
        "#,
    )
    .bind(document_id)
    .bind(document.file_size)
    .bind(document.modified_at)
    .bind(&source_hash)
    .bind(&extract.text_extraction_version)
    .bind(now)
    .execute(db)
    .await?;

    let _ = tantivy_tx
        .send(IndexCommand::UpsertDocument {
            document_id: document_id.to_string(),
        })
        .await;
    let _ = event_tx.send(AppEvent::DocumentReady {
        document_id: document_id.to_string(),
        folder_id: document.folder_id,
    });
    Ok(())
}

struct TextExtract {
    title: Option<String>,
    page_count: i64,
    pages: Vec<crate::ingest::chunker::PageText>,
    text_extraction_version: String,
}

fn validate_supported_file(path: &std::path::Path) -> anyhow::Result<()> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();

    match extension.as_str() {
        "pdf" => validate_pdf_header(path),
        "html" | "htm" => Ok(()),
        other => anyhow::bail!("unsupported document extension: {other}"),
    }
}

fn validate_pdf_header(path: &std::path::Path) -> anyhow::Result<()> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() == 0 {
        anyhow::bail!("empty file; not a valid PDF");
    }
    if metadata.len() < 5 {
        anyhow::bail!("file is too small to be a valid PDF");
    }

    let mut file = std::fs::File::open(path)?;
    let mut header = [0_u8; 5];
    file.read_exact(&mut header)?;
    if &header != b"%PDF-" {
        anyhow::bail!("invalid PDF header; file does not start with %PDF-");
    }
    Ok(())
}

fn extract_text(path: &std::path::Path) -> anyhow::Result<TextExtract> {
    let extension = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    match extension.as_str() {
        "pdf" => {
            let extract = pdf::extract(path)?;
            Ok(TextExtract {
                title: extract.title,
                page_count: extract.page_count,
                pages: extract.pages,
                text_extraction_version: "lopdf-v1".to_string(),
            })
        }
        "html" | "htm" => {
            let extract = html::extract(path)?;
            Ok(TextExtract {
                title: extract.title,
                page_count: 1,
                pages: extract.pages,
                text_extraction_version: "html-basic-v1".to_string(),
            })
        }
        other => anyhow::bail!("unsupported document extension: {other}"),
    }
}
