-- =============================================================================
-- 006_unique_page_source_urls.sql
-- 同一作品内の同一取得元URL重複を防止
-- =============================================================================

-- 既存DBに重複URLが残っていても migration が失敗しないよう、ここでは
-- unique index ではなく今後の INSERT/UPDATE を防ぐ trigger を追加する。
-- 新規DBは 001_initial_schema.sql の unique index で同じ制約を持つ。

CREATE TRIGGER IF NOT EXISTS trg_pages_prevent_duplicate_source_url_insert
  BEFORE INSERT ON pages
  WHEN new.source_url IS NOT NULL
   AND EXISTS (
    SELECT 1
    FROM pages
    WHERE work_id = new.work_id
      AND source_type = new.source_type
      AND source_url = new.source_url
  )
BEGIN
  SELECT RAISE(ABORT, 'duplicate page source_url');
END;

CREATE TRIGGER IF NOT EXISTS trg_pages_prevent_duplicate_source_url_update
  BEFORE UPDATE OF work_id, source_type, source_url ON pages
  WHEN new.source_url IS NOT NULL
   AND EXISTS (
    SELECT 1
    FROM pages
    WHERE work_id = new.work_id
      AND source_type = new.source_type
      AND source_url = new.source_url
      AND id <> old.id
  )
BEGIN
  SELECT RAISE(ABORT, 'duplicate page source_url');
END;
