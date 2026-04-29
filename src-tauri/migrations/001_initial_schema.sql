-- =============================================================================
-- 001_initial_schema.sql
-- 小説アーカイブ・ビューアー 初期スキーマ
-- 対応要件定義: v4.2
-- =============================================================================

CREATE TABLE IF NOT EXISTS sites (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT    NOT NULL,
  base_url    TEXT    NOT NULL,
  created_at  TEXT    NOT NULL,
  updated_at  TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS site_profiles (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  site_id       INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  name          TEXT    NOT NULL,
  profile_json  TEXT    NOT NULL,
  created_at    TEXT    NOT NULL,
  updated_at    TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS works (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  site_id          INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  site_profile_id  INTEGER REFERENCES site_profiles(id) ON DELETE SET NULL,
  title            TEXT    NOT NULL,
  author_name      TEXT,
  description      TEXT,
  source_url       TEXT,
  sort_order       INTEGER NOT NULL,
  created_at       TEXT    NOT NULL,
  updated_at       TEXT    NOT NULL
);

CREATE TABLE IF NOT EXISTS pages (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  work_id             INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,
  page_number         INTEGER,
  sort_order          INTEGER NOT NULL,
  title               TEXT,
  source_url          TEXT,
  source_type         TEXT    NOT NULL
    CHECK (source_type IN ('normal', 'wayback', 'local')),
  requested_encoding  TEXT,
  detected_encoding   TEXT,
  content_text        TEXT,
  content_html_path   TEXT,
  fetch_status        TEXT    NOT NULL DEFAULT 'pending'
    CHECK (fetch_status IN ('pending', 'success', 'fetch_failed', 'parse_failed', 'save_failed', 'skipped')),
  fetch_error         TEXT,
  fetched_at          TEXT,
  created_at          TEXT    NOT NULL,
  updated_at          TEXT    NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_works_site_sort
  ON works(site_id, sort_order);

CREATE INDEX IF NOT EXISTS idx_site_profiles_site
  ON site_profiles(site_id);

CREATE INDEX IF NOT EXISTS idx_works_site_profile
  ON works(site_profile_id);

CREATE INDEX IF NOT EXISTS idx_pages_work_sort
  ON pages(work_id, sort_order);

CREATE INDEX IF NOT EXISTS idx_pages_source_url
  ON pages(source_url);

CREATE UNIQUE INDEX IF NOT EXISTS uq_pages_work_source_type_url
  ON pages(work_id, source_type, source_url)
  WHERE source_url IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_pages_fetch_status
  ON pages(fetch_status);

CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
  title,
  content_text,
  content     = 'pages',
  content_rowid = 'id',
  tokenize    = 'trigram'
);

CREATE TRIGGER IF NOT EXISTS pages_ai
  AFTER INSERT ON pages
BEGIN
  INSERT INTO pages_fts(rowid, title, content_text)
  VALUES (new.id, new.title, new.content_text);
END;

CREATE TRIGGER IF NOT EXISTS pages_bd
  BEFORE DELETE ON pages
BEGIN
  INSERT INTO pages_fts(pages_fts, rowid, title, content_text)
  VALUES ('delete', old.id, old.title, old.content_text);
END;

CREATE TRIGGER IF NOT EXISTS pages_bu
  BEFORE UPDATE ON pages
BEGIN
  INSERT INTO pages_fts(pages_fts, rowid, title, content_text)
  VALUES ('delete', old.id, old.title, old.content_text);
END;

CREATE TRIGGER IF NOT EXISTS pages_au
  AFTER UPDATE ON pages
BEGIN
  INSERT INTO pages_fts(rowid, title, content_text)
  VALUES (new.id, new.title, new.content_text);
END;

CREATE TABLE IF NOT EXISTS schema_migrations (
  version     TEXT    NOT NULL PRIMARY KEY,
  applied_at  TEXT    NOT NULL
);
