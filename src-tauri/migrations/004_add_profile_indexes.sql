-- =============================================================================
-- 004_add_profile_indexes.sql
-- プロファイル参照の外部キー操作を高速化するインデックス
-- =============================================================================

CREATE INDEX IF NOT EXISTS idx_site_profiles_site
  ON site_profiles(site_id);

CREATE INDEX IF NOT EXISTS idx_works_site_profile
  ON works(site_profile_id);
