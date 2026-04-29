-- =============================================================================
-- 005_use_trigram_fts.sql
-- 日本語本文検索の部分一致を改善するためFTS5 trigramへ移行
-- =============================================================================

DROP TRIGGER IF EXISTS pages_ai;
DROP TRIGGER IF EXISTS pages_bd;
DROP TRIGGER IF EXISTS pages_bu;
DROP TRIGGER IF EXISTS pages_au;
DROP TABLE IF EXISTS pages_fts;

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

INSERT INTO pages_fts(pages_fts) VALUES ('rebuild');
