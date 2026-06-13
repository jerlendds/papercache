use serde::Serialize;

use crate::util::{hash::blake3_hex, time::now_rfc3339};

#[derive(Debug, Clone)]
pub struct PageText {
    pub page: i64,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TextChunk {
    pub id: String,
    pub chunk_index: i64,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub text: String,
    pub token_count: Option<i64>,
    pub metadata_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub fn chunk_pages(document_id: &str, source_hash: &str, pages: &[PageText]) -> Vec<TextChunk> {
    let target = 4_000;
    let overlap = 500;
    let mut combined = Vec::new();
    for page in pages {
        combined.push((page.page, page.text.as_str()));
    }

    let mut chunks = Vec::new();
    let mut text = String::new();
    let mut page_start = None;
    let mut page_end = None;

    for (page, page_text) in combined {
        if page_start.is_none() && !page_text.trim().is_empty() {
            page_start = Some(page);
        }
        if !page_text.trim().is_empty() {
            page_end = Some(page);
        }
        if !text.is_empty() {
            text.push_str("\n\n");
        }
        text.push_str(page_text);

        while text.len() >= target {
            let split = boundary_at_or_before(&text, target)
                .unwrap_or_else(|| char_boundary_at_or_before(&text, target));
            let current = text[..split].trim().to_string();
            push_chunk(
                &mut chunks,
                document_id,
                source_hash,
                current,
                page_start,
                page_end,
            );
            let keep_from = char_boundary_at_or_before(&text, split.saturating_sub(overlap));
            text = text[keep_from..].to_string();
            page_start = page_end;
        }
    }

    if !text.trim().is_empty() {
        push_chunk(
            &mut chunks,
            document_id,
            source_hash,
            text.trim().to_string(),
            page_start,
            page_end,
        );
    }

    chunks
}

fn push_chunk(
    chunks: &mut Vec<TextChunk>,
    document_id: &str,
    source_hash: &str,
    text: String,
    page_start: Option<i64>,
    page_end: Option<i64>,
) {
    let chunk_index = chunks.len() as i64;
    let now = now_rfc3339();
    let id = blake3_hex(&format!("{document_id}:{source_hash}:{chunk_index}"));
    let token_count = text.split_whitespace().count() as i64;
    chunks.push(TextChunk {
        id,
        chunk_index,
        page_start,
        page_end,
        text,
        token_count: Some(token_count),
        metadata_json: None,
        created_at: now.clone(),
        updated_at: now,
    });
}

fn boundary_at_or_before(text: &str, max: usize) -> Option<usize> {
    text.char_indices()
        .take_while(|(idx, _)| *idx <= max)
        .filter(|(_, ch)| ch.is_whitespace() || *ch == '.')
        .map(|(idx, ch)| idx + ch.len_utf8())
        .last()
}

fn char_boundary_at_or_before(text: &str, max: usize) -> usize {
    if max >= text.len() {
        return text.len();
    }
    let mut idx = max;
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[cfg(test)]
mod tests {
    use super::{PageText, chunk_pages};

    #[test]
    fn chunk_pages_handles_multibyte_overlap_boundary() {
        let text = format!("{}ł{}", "a".repeat(3491), " b".repeat(400));
        let chunks = chunk_pages("doc", "hash", &[PageText { page: 1, text }]);

        assert!(chunks.len() > 1);
        assert!(
            chunks
                .iter()
                .all(|chunk| chunk.text.is_char_boundary(chunk.text.len()))
        );
    }

    #[test]
    fn chunk_pages_handles_multibyte_split_fallback() {
        let text = "ł".repeat(2_500);
        let chunks = chunk_pages("doc", "hash", &[PageText { page: 1, text }]);

        assert!(chunks.len() > 1);
    }
}
