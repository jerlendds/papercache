use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Classification {
    pub document_type: String,
    pub topics: Vec<String>,
    pub status: Option<String>,
    pub confidence: f32,
    pub source: String,
}

pub fn classify(
    title: Option<&str>,
    doi: Option<&str>,
    arxiv_id: Option<&str>,
    text: &str,
) -> Classification {
    let haystack = format!(
        "{}\n{}",
        title.unwrap_or_default(),
        text.chars().take(12_000).collect::<String>()
    )
    .to_lowercase();

    let paper_signals = doi.is_some()
        || arxiv_id.is_some()
        || ["references", "abstract", "introduction"]
            .iter()
            .filter(|needle| haystack.contains(**needle))
            .count()
            >= 2;

    let topic_map = [
        (
            "machine learning",
            ["machine learning", "neural", "deep learning", "transformer"].as_slice(),
        ),
        (
            "retrieval",
            ["retrieval", "search", "indexing", "ranking"].as_slice(),
        ),
        (
            "rag",
            ["retrieval augmented", "rag", "question answering"].as_slice(),
        ),
        (
            "databases",
            ["database", "sqlite", "query planner", "transaction"].as_slice(),
        ),
        (
            "systems",
            ["distributed system", "scheduler", "runtime", "consistency"].as_slice(),
        ),
        (
            "security",
            ["security", "privacy", "authentication", "encryption"].as_slice(),
        ),
    ];
    let topics = topic_map
        .iter()
        .filter_map(|(topic, needles)| {
            needles
                .iter()
                .any(|needle| haystack.contains(needle))
                .then(|| (*topic).to_string())
        })
        .collect::<Vec<_>>();

    Classification {
        document_type: if paper_signals {
            "research_paper".to_string()
        } else {
            "document".to_string()
        },
        confidence: if paper_signals { 0.62 } else { 0.35 },
        topics,
        status: None,
        source: "rules-v1".to_string(),
    }
}

pub fn detect_doi(text: &str) -> Option<String> {
    text.split_whitespace()
        .find(|token| token.starts_with("10.") && token.contains('/'))
        .map(|token| {
            token
                .trim_matches(|c: char| c == ',' || c == '.')
                .to_string()
        })
}

pub fn detect_arxiv(text: &str) -> Option<String> {
    let lower = text.to_lowercase();
    lower.find("arxiv:").map(|idx| {
        lower[idx + 6..]
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_matches(|c: char| c == ',' || c == '.')
            .to_string()
    })
}
