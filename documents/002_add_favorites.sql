-- =============================================================================
-- 002_add_favorites.sql
-- お気に入り機能の追加
-- 対応要件定義: v4.3
-- =============================================================================

-- -----------------------------------------------------------------------------
-- favorites: お気に入り
-- ページ単位で登録、単一リスト、メモ無し、削除連動
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS favorites (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  page_id     INTEGER NOT NULL UNIQUE
              REFERENCES pages(id) ON DELETE CASCADE,
  -- UNIQUE: 同一ページの重複登録を防ぐ
  -- ON DELETE CASCADE: ページ削除時にお気に入りも自動削除
  created_at  TEXT NOT NULL  -- UTC ISO8601、登録順表示にも使う
);


-- -----------------------------------------------------------------------------
-- インデックス
-- -----------------------------------------------------------------------------
-- page_id は UNIQUE 制約により自動的にインデックスが作成されるため追加不要
-- favoriteCheck（特定ページのお気に入り判定）はこの自動インデックスで高速化される

-- 登録順表示用のインデックス（任意・パフォーマンス用）
-- レコード数が少ない想定（数百件以下）のため、必須ではないが将来のために用意
CREATE INDEX IF NOT EXISTS idx_favorites_created_at
  ON favorites(created_at);


-- =============================================================================
-- 参考: お気に入り一覧取得クエリ（サイト・作品でグループ化）
-- =============================================================================
-- フロントエンド側でグループ化する前提のため、SQLは平坦なJOIN結果を返す。
-- ORDER BY で sites.name → works.sort_order → pages.sort_order 順に並べる。
--
-- SELECT
--   f.id           AS favorite_id,
--   f.created_at   AS favorited_at,
--   p.id           AS page_id,
--   p.title        AS page_title,
--   p.page_number  AS page_number,
--   w.id           AS work_id,
--   w.title        AS work_title,
--   s.id           AS site_id,
--   s.name         AS site_name
-- FROM favorites f
-- JOIN pages p  ON f.page_id  = p.id
-- JOIN works w  ON p.work_id  = w.id
-- JOIN sites s  ON w.site_id  = s.id
-- ORDER BY s.name, w.sort_order, p.sort_order;
