use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::{fmt, str::FromStr};

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $value),+
                }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl FromStr for $name {
            type Err = anyhow::Error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value {
                    $($value => Ok(Self::$variant),)+
                    _ => anyhow::bail!("invalid {}: {}", stringify!($name), value),
                }
            }
        }
    };
}

string_enum!(DocumentStatus {
    Discovered => "discovered",
    Processing => "processing",
    Ready => "ready",
    Failed => "failed",
    Missing => "missing",
});

string_enum!(JobStatus {
    Queued => "queued",
    Running => "running",
    Succeeded => "succeeded",
    Failed => "failed",
    Canceled => "canceled",
});

string_enum!(JobKind {
    ScanFolder => "scan_folder",
    IngestPdf => "ingest_pdf",
    ReindexDocument => "reindex_document",
    RenderCover => "render_cover",
});

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct ImportedFolder {
    pub id: String,
    pub path: String,
    pub recursive: i64,
    pub enabled: i64,
    pub last_scan_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Document {
    pub id: String,
    pub folder_id: Option<String>,
    pub path: String,
    pub canonical_path: Option<String>,
    pub file_name: String,
    pub file_size: Option<i64>,
    pub modified_at: Option<String>,
    pub sha256: Option<String>,
    pub title: Option<String>,
    pub authors_json: Option<String>,
    pub year: Option<i64>,
    pub doi: Option<String>,
    pub arxiv_id: Option<String>,
    pub page_count: Option<i64>,
    pub status: String,
    pub error: Option<String>,
    pub classification_json: Option<String>,
    pub cover_asset_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub chunk_index: i64,
    pub page_start: Option<i64>,
    pub page_end: Option<i64>,
    pub text: String,
    pub token_count: Option<i64>,
    pub metadata_json: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct Job {
    pub id: String,
    pub kind: String,
    pub payload_json: String,
    pub status: String,
    pub priority: i64,
    pub attempts: i64,
    pub max_attempts: i64,
    pub error: Option<String>,
    pub run_after: Option<String>,
    pub locked_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

pub type JobId = String;
