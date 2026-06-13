use std::path::Path;

use crate::ingest::chunker::PageText;

#[derive(Debug, Clone)]
pub struct HtmlExtract {
    pub title: Option<String>,
    pub pages: Vec<PageText>,
}

pub fn extract(path: &Path) -> anyhow::Result<HtmlExtract> {
    let html = std::fs::read_to_string(path)?;
    let title = extract_title(&html);
    let text = html_to_text(&html);
    Ok(HtmlExtract {
        title,
        pages: vec![PageText { page: 1, text }],
    })
}

fn extract_title(html: &str) -> Option<String> {
    let lower = html.to_ascii_lowercase();
    let start = lower.find("<title")?;
    let title_open_end = lower[start..].find('>')? + start + 1;
    let title_close = lower[title_open_end..].find("</title>")? + title_open_end;
    let title = decode_entities(&html[title_open_end..title_close]);
    let title = collapse_whitespace(&title);
    (!title.is_empty()).then_some(title)
}

fn html_to_text(html: &str) -> String {
    let mut text = String::new();
    let mut in_tag = false;
    let mut in_script = false;
    let mut in_style = false;
    let mut tag = String::new();

    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                let tag_name = tag
                    .trim()
                    .trim_start_matches('/')
                    .split_whitespace()
                    .next()
                    .unwrap_or("")
                    .to_ascii_lowercase();
                let is_closing = tag.trim_start().starts_with('/');
                if tag_name == "script" {
                    in_script = !is_closing;
                } else if tag_name == "style" {
                    in_style = !is_closing;
                } else if matches!(
                    tag_name.as_str(),
                    "br" | "p" | "div" | "li" | "tr" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6"
                ) {
                    text.push('\n');
                }
                tag.clear();
                in_tag = false;
            } else {
                tag.push(ch);
            }
            continue;
        }

        if ch == '<' {
            in_tag = true;
            tag.clear();
            continue;
        }

        if !in_script && !in_style {
            text.push(ch);
        }
    }

    collapse_whitespace(&decode_entities(&text))
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::{extract_title, html_to_text};

    #[test]
    fn html_to_text_strips_tags_and_scripts() {
        let text = html_to_text(
            "<html><head><title>T</title><style>.x{}</style><script>alert(1)</script></head><body><h1>Hello</h1><p>A&nbsp;B &amp; C</p></body></html>",
        );

        assert!(text.contains("Hello"));
        assert!(text.contains("A B & C"));
        assert!(!text.contains("alert"));
    }

    #[test]
    fn extracts_title() {
        assert_eq!(
            extract_title("<title>A &amp; B</title>").as_deref(),
            Some("A & B")
        );
    }
}
