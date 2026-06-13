use std::{any::Any, panic::AssertUnwindSafe, path::Path, sync::Mutex};

use lopdf::Document;

use crate::ingest::chunker::PageText;

static PANIC_HOOK_LOCK: Mutex<()> = Mutex::new(());

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
        let text = extract_page_text(&doc, *page_number, path);
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

fn extract_page_text(doc: &Document, page_number: u32, path: &Path) -> String {
    match catch_unwind_silent(|| doc.extract_text(&[page_number])) {
        Ok(Ok(text)) => text,
        Ok(Err(error)) => {
            tracing::warn!(
                page_number,
                path = %path.display(),
                error = %error,
                "PDF page text extraction failed"
            );
            String::new()
        }
        Err(error) => {
            tracing::warn!(
                page_number,
                path = %path.display(),
                error = %error,
                "PDF page text extraction panicked"
            );
            String::new()
        }
    }
}

fn catch_unwind_silent<F, T>(action: F) -> Result<T, String>
where
    F: FnOnce() -> T,
{
    let _guard = PANIC_HOOK_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(AssertUnwindSafe(action));
    std::panic::set_hook(previous_hook);
    result.map_err(panic_payload_to_string)
}

fn panic_payload_to_string(payload: Box<dyn Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "unknown panic during PDF text extraction".to_string()
}
