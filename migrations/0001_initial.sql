PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS imported_folders (
  id TEXT PRIMARY KEY,
  path TEXT NOT NULL UNIQUE,
  recursive INTEGER NOT NULL DEFAULT 1,
  enabled INTEGER NOT NULL DEFAULT 1,
  last_scan_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS documents (
  id TEXT PRIMARY KEY,
  folder_id TEXT REFERENCES imported_folders(id) ON DELETE SET NULL,
  path TEXT NOT NULL UNIQUE,
  canonical_path TEXT,
  file_name TEXT NOT NULL,
  file_size INTEGER,
  modified_at TEXT,
  sha256 TEXT,
  title TEXT,
  authors_json TEXT,
  year INTEGER,
  doi TEXT,
  arxiv_id TEXT,
  page_count INTEGER,
  status TEXT NOT NULL,
  error TEXT,
  classification_json TEXT,
  cover_asset_id TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS documents_sha256_idx ON documents(sha256);
CREATE INDEX IF NOT EXISTS documents_status_idx ON documents(status);
CREATE INDEX IF NOT EXISTS documents_folder_idx ON documents(folder_id);

CREATE TABLE IF NOT EXISTS document_processing_state (
  document_id TEXT PRIMARY KEY REFERENCES documents(id) ON DELETE CASCADE,
  source_file_size INTEGER,
  source_modified_at TEXT,
  source_sha256 TEXT,
  text_extraction_version TEXT,
  thumbnail_version TEXT,
  classifier_version TEXT,
  chunker_version TEXT,
  indexed_in_tantivy INTEGER NOT NULL DEFAULT 0,
  indexed_in_vector_store INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS chunks (
  id TEXT PRIMARY KEY,
  document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  chunk_index INTEGER NOT NULL,
  page_start INTEGER,
  page_end INTEGER,
  text TEXT NOT NULL,
  token_count INTEGER,
  metadata_json TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  UNIQUE(document_id, chunk_index)
);

CREATE INDEX IF NOT EXISTS chunks_document_idx ON chunks(document_id);

CREATE TABLE IF NOT EXISTS assets (
  id TEXT PRIMARY KEY,
  document_id TEXT REFERENCES documents(id) ON DELETE CASCADE,
  kind TEXT NOT NULL,
  mime TEXT NOT NULL,
  path TEXT NOT NULL,
  sha256 TEXT,
  width INTEGER,
  height INTEGER,
  metadata_json TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS assets_document_idx ON assets(document_id);
CREATE INDEX IF NOT EXISTS assets_kind_idx ON assets(kind);

CREATE TABLE IF NOT EXISTS jobs (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  status TEXT NOT NULL,
  priority INTEGER NOT NULL DEFAULT 0,
  attempts INTEGER NOT NULL DEFAULT 0,
  max_attempts INTEGER NOT NULL DEFAULT 3,
  error TEXT,
  run_after TEXT,
  locked_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS jobs_claim_idx
ON jobs(status, priority, run_after, created_at);

CREATE TABLE IF NOT EXISTS job_events (
  id TEXT PRIMARY KEY,
  job_id TEXT NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
  event_type TEXT NOT NULL,
  message TEXT,
  metadata_json TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS app_settings (
  key TEXT PRIMARY KEY,
  value_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
