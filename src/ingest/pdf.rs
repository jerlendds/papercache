use std::path::Path;

use lopdf::Document;

use crate::ingest::chunker::PageText;

#[derive(Debug, Clone)]
pub struct PdfExtract {
    pub title: Option<String>,
    pub page_count: i64,
    pub pages: Vec<PageText>,
}

pub fn extract(path: &Path) -> anyhow::Result<PdfExtract> {
    let doc = Document::load(path)?;
    let pages_map = doc.get_pages();
    let mut pages = Vec::new();
    for (page_number, _) in pages_map.iter() {
        let text = doc.extract_text(&[*page_number]).unwrap_or_default();
        pages.push(PageText {
            page: *page_number as i64,
            text,
        });
    }

    let title = doc
        .trailer
        .get(b"Info")
        .ok()
        .and_then(|obj| obj.as_reference().ok())
        .and_then(|id| doc.get_dictionary(id).ok())
        .and_then(|dict| dict.get(b"Title").ok())
        .and_then(|obj| obj.as_str().ok())
        .and_then(|bytes| String::from_utf8(bytes.to_vec()).ok())
        .filter(|value| !value.trim().is_empty());

    Ok(PdfExtract {
        title,
        page_count: pages_map.len() as i64,
        pages,
    })
}
