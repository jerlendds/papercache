use sqlx::SqlitePool;

use crate::ingest::chunker::TextChunk;

pub async fn replace_chunks(
    db: &SqlitePool,
    document_id: &str,
    chunks: &[TextChunk],
) -> anyhow::Result<()> {
    let mut tx = db.begin().await?;
    sqlx::query("DELETE FROM chunks WHERE document_id = ?")
        .bind(document_id)
        .execute(&mut *tx)
        .await?;

    for chunk in chunks {
        sqlx::query(
            r#"
            INSERT INTO chunks
              (id, document_id, chunk_index, page_start, page_end, text, token_count, metadata_json, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&chunk.id)
        .bind(document_id)
        .bind(chunk.chunk_index)
        .bind(chunk.page_start)
        .bind(chunk.page_end)
        .bind(&chunk.text)
        .bind(chunk.token_count)
        .bind(&chunk.metadata_json)
        .bind(&chunk.created_at)
        .bind(&chunk.updated_at)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}
