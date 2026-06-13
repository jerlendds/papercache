use std::path::PathBuf;

use sqlx::SqlitePool;
use tantivy::{
    Index, IndexWriter, TantivyDocument, Term, doc,
    query::QueryParser,
    schema::{FAST, Field, INDEXED, STORED, STRING, Schema, TEXT, Value},
};
use tokio::sync::mpsc;

use crate::{
    db::models::{Chunk, Document},
    index::commands::IndexCommand,
    util::time::now_rfc3339,
};

#[derive(Clone)]
pub struct IndexHandle {
    pub index: Index,
    pub schema: SearchSchema,
}

#[derive(Clone)]
pub struct SearchSchema {
    pub document_id: Field,
    pub chunk_id: Field,
    pub title: Field,
    pub authors: Field,
    pub year: Field,
    pub path: Field,
    pub page_start: Field,
    pub page_end: Field,
    pub text: Field,
    pub topics: Field,
}

pub fn open_or_create(index_dir: PathBuf) -> anyhow::Result<IndexHandle> {
    std::fs::create_dir_all(&index_dir)?;
    let mut builder = Schema::builder();
    let schema = SearchSchema {
        document_id: builder.add_text_field("document_id", STRING | STORED),
        chunk_id: builder.add_text_field("chunk_id", STRING | STORED),
        title: builder.add_text_field("title", TEXT | STORED),
        authors: builder.add_text_field("authors", TEXT | STORED),
        year: builder.add_i64_field("year", INDEXED | STORED | FAST),
        path: builder.add_text_field("path", STRING | STORED),
        page_start: builder.add_i64_field("page_start", STORED),
        page_end: builder.add_i64_field("page_end", STORED),
        text: builder.add_text_field("text", TEXT | STORED),
        topics: builder.add_text_field("topics", TEXT | STORED),
    };
    let built_schema = builder.build();
    let index = Index::open_in_dir(&index_dir)
        .or_else(|_| Index::create_in_dir(&index_dir, built_schema))?;
    Ok(IndexHandle { index, schema })
}

pub async fn run_worker(
    db: SqlitePool,
    mut rx: mpsc::Receiver<IndexCommand>,
    index_dir: PathBuf,
) -> anyhow::Result<()> {
    let handle = open_or_create(index_dir)?;
    let mut writer = handle.index.writer(50_000_000)?;
    while let Some(command) = rx.recv().await {
        let result = match command {
            IndexCommand::UpsertDocument { document_id } => {
                upsert_document(&db, &handle.schema, &mut writer, &document_id).await
            }
            IndexCommand::DeleteDocument { document_id } => {
                delete_document(&handle.schema, &mut writer, &document_id)
            }
            IndexCommand::RebuildAll => rebuild_all(&db, &handle.schema, &mut writer).await,
        };
        if let Err(error) = result {
            tracing::error!(?error, "tantivy command failed");
        } else if let Err(error) = writer.commit() {
            tracing::error!(?error, "tantivy commit failed");
        }
    }
    Ok(())
}

async fn rebuild_all(
    db: &SqlitePool,
    schema: &SearchSchema,
    writer: &mut IndexWriter,
) -> anyhow::Result<()> {
    writer.delete_all_documents()?;
    let ids = sqlx::query_scalar::<_, String>("SELECT id FROM documents WHERE status = 'ready'")
        .fetch_all(db)
        .await?;
    for id in ids {
        upsert_document(db, schema, writer, &id).await?;
    }
    Ok(())
}

async fn upsert_document(
    db: &SqlitePool,
    schema: &SearchSchema,
    writer: &mut IndexWriter,
    document_id: &str,
) -> anyhow::Result<()> {
    delete_document(schema, writer, document_id)?;
    let document = sqlx::query_as::<_, Document>("SELECT * FROM documents WHERE id = ?")
        .bind(document_id)
        .fetch_one(db)
        .await?;
    let chunks = sqlx::query_as::<_, Chunk>(
        "SELECT * FROM chunks WHERE document_id = ? ORDER BY chunk_index",
    )
    .bind(document_id)
    .fetch_all(db)
    .await?;
    let topics = document
        .classification_json
        .as_deref()
        .and_then(|value| serde_json::from_str::<serde_json::Value>(value).ok())
        .and_then(|value| value.get("topics").cloned())
        .and_then(|value| value.as_array().cloned())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();

    for chunk in chunks {
        writer.add_document(doc!(
            schema.document_id => document.id.clone(),
            schema.chunk_id => chunk.id,
            schema.title => document.title.clone().unwrap_or_else(|| document.file_name.clone()),
            schema.authors => document.authors_json.clone().unwrap_or_default(),
            schema.year => document.year.unwrap_or_default(),
            schema.path => document.path.clone(),
            schema.page_start => chunk.page_start.unwrap_or_default(),
            schema.page_end => chunk.page_end.unwrap_or_default(),
            schema.text => chunk.text,
            schema.topics => topics.clone(),
        ))?;
    }

    sqlx::query(
        r#"
        UPDATE document_processing_state
        SET indexed_in_tantivy = 1, updated_at = ?
        WHERE document_id = ?
        "#,
    )
    .bind(now_rfc3339())
    .bind(document_id)
    .execute(db)
    .await?;
    Ok(())
}

fn delete_document(
    schema: &SearchSchema,
    writer: &mut IndexWriter,
    document_id: &str,
) -> anyhow::Result<()> {
    writer.delete_term(Term::from_field_text(schema.document_id, document_id));
    Ok(())
}

#[derive(Debug, serde::Serialize)]
pub struct SearchHit {
    pub document_id: String,
    pub chunk_id: String,
    pub title: String,
    pub path: String,
    pub page_start: i64,
    pub page_end: i64,
    pub snippet: String,
    pub score: f32,
}

pub fn search(
    index_dir: PathBuf,
    query: &str,
    limit: usize,
    offset: usize,
) -> anyhow::Result<Vec<SearchHit>> {
    let handle = open_or_create(index_dir)?;
    let reader = handle.index.reader()?;
    let searcher = reader.searcher();
    let parser = QueryParser::for_index(
        &handle.index,
        vec![
            handle.schema.title,
            handle.schema.text,
            handle.schema.topics,
        ],
    );
    let parsed = parser.parse_query(query)?;
    let collector = tantivy::collector::TopDocs::with_limit(limit)
        .and_offset(offset)
        .order_by_score();
    let top_docs = searcher.search(&parsed, &collector)?;
    let mut hits = Vec::new();
    for (score, address) in top_docs {
        let doc: TantivyDocument = searcher.doc(address)?;
        let text = text_value(&doc, handle.schema.text);
        hits.push(SearchHit {
            document_id: text_value(&doc, handle.schema.document_id),
            chunk_id: text_value(&doc, handle.schema.chunk_id),
            title: text_value(&doc, handle.schema.title),
            path: text_value(&doc, handle.schema.path),
            page_start: i64_value(&doc, handle.schema.page_start),
            page_end: i64_value(&doc, handle.schema.page_end),
            snippet: make_snippet(&text),
            score,
        });
    }
    Ok(hits)
}

fn text_value(doc: &TantivyDocument, field: Field) -> String {
    doc.get_first(field)
        .and_then(|value| value.as_str())
        .unwrap_or_default()
        .to_string()
}

fn i64_value(doc: &TantivyDocument, field: Field) -> i64 {
    doc.get_first(field)
        .and_then(|value| value.as_i64())
        .unwrap_or_default()
}

fn make_snippet(text: &str) -> String {
    let mut snippet = text.chars().take(260).collect::<String>();
    if text.chars().count() > 260 {
        snippet.push_str("...");
    }
    snippet
}
