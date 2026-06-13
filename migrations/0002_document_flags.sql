ALTER TABLE documents ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0;
ALTER TABLE documents ADD COLUMN is_bookmarked INTEGER NOT NULL DEFAULT 0;
ALTER TABLE documents ADD COLUMN is_pinned INTEGER NOT NULL DEFAULT 0;

CREATE INDEX IF NOT EXISTS documents_favorite_idx ON documents(is_favorite);
CREATE INDEX IF NOT EXISTS documents_bookmarked_idx ON documents(is_bookmarked);
CREATE INDEX IF NOT EXISTS documents_pinned_idx ON documents(is_pinned);
