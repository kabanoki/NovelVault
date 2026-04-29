-- =============================================================================
-- 002_add_favorites.sql
-- お気に入り機能の追加
-- 対応要件定義: v4.3
-- =============================================================================

CREATE TABLE IF NOT EXISTS favorites (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  page_id     INTEGER NOT NULL UNIQUE
              REFERENCES pages(id) ON DELETE CASCADE,
  created_at  TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_favorites_created_at
  ON favorites(created_at);
