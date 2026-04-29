-- =============================================================================
-- 003_add_wayback_metadata.sql
-- Wayback URL メタデータの追加
-- =============================================================================

ALTER TABLE pages ADD COLUMN canonical_url TEXT;
ALTER TABLE pages ADD COLUMN archived_at TEXT;
