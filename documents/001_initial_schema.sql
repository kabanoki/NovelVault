-- =============================================================================
-- 001_initial_schema.sql
-- 小説アーカイブ・ビューアー 初期スキーマ
-- 対応要件定義: v4.2
-- =============================================================================

-- =============================================================================
-- テーブル定義
-- =============================================================================

-- -----------------------------------------------------------------------------
-- sites: サイト
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS sites (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  name        TEXT    NOT NULL,
  base_url    TEXT    NOT NULL,
  created_at  TEXT    NOT NULL,  -- UTC ISO8601 例: 2026-04-26T12:34:56.789Z
  updated_at  TEXT    NOT NULL
);


-- -----------------------------------------------------------------------------
-- site_profiles: サイトプロファイル
-- 1サイトに複数プロファイルを持てる（作品ごとにHTML構造が異なるケースに対応）
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS site_profiles (
  id            INTEGER PRIMARY KEY AUTOINCREMENT,
  site_id       INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  name          TEXT    NOT NULL,
  profile_json  TEXT    NOT NULL,  -- サイトプロファイルJSON（仕様は別途定義）
  created_at    TEXT    NOT NULL,
  updated_at    TEXT    NOT NULL
);


-- -----------------------------------------------------------------------------
-- works: 作品
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS works (
  id               INTEGER PRIMARY KEY AUTOINCREMENT,
  site_id          INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE,
  site_profile_id  INTEGER REFERENCES site_profiles(id) ON DELETE SET NULL,
  -- プロファイル削除時はNULLに。作品自体は残す。
  title            TEXT    NOT NULL,
  author_name      TEXT,            -- NULL許可（閉鎖サイトでは取得できないケースがある）
  description      TEXT,            -- NULL許可
  source_url       TEXT,
  sort_order       INTEGER NOT NULL, -- 10刻み採番: MAX(sort_order)+10
  created_at       TEXT    NOT NULL,
  updated_at       TEXT    NOT NULL
);


-- -----------------------------------------------------------------------------
-- pages: ページ（章）
-- -----------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS pages (
  id                  INTEGER PRIMARY KEY AUTOINCREMENT,
  work_id             INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE,

  -- 表示・並び順
  page_number         INTEGER,        -- サイト上の章番号（「第3話」→ 3）。番外編等はNULL
  sort_order          INTEGER NOT NULL, -- アプリ内並び順（10刻み採番）。ソートはこちらを使う

  -- コンテンツ
  title               TEXT,
  source_url          TEXT,
  source_type         TEXT    NOT NULL
    CHECK (source_type IN ('normal', 'wayback', 'local')),
  requested_encoding  TEXT,           -- 指定した文字コード（utf-8 / shift_jis / euc-jp 等）
  detected_encoding   TEXT,           -- 実際に検出された文字コード
  content_text        TEXT,           -- 本文テキスト（全文検索・閲覧に使用）
  content_html_path   TEXT,           -- アプリデータディレクトリからの相対パス

  -- 取得ステータス
  -- pending     : 未取得（Phase 7以降の一括取得で事前登録時に使用）
  -- success     : 取得・抽出・保存すべて成功
  -- fetch_failed: HTTP取得失敗
  -- parse_failed: HTML解析・本文抽出失敗
  -- save_failed : DBレコード作成後の保存処理失敗（HTMLファイル保存、FTS同期等）
  -- skipped     : スキップ（Phase 7以降）
  fetch_status        TEXT    NOT NULL DEFAULT 'pending'
    CHECK (fetch_status IN ('pending', 'success', 'fetch_failed', 'parse_failed', 'save_failed', 'skipped')),
  fetch_error         TEXT,           -- エラー内容（失敗時のみ）

  fetched_at          TEXT,           -- 取得完了日時（成功時のみセット）
  created_at          TEXT    NOT NULL,
  updated_at          TEXT    NOT NULL

  -- Phase 8 以降で追加予定:
  -- canonical_url   TEXT,            -- Wayback元URL（例: http://example.com/001.html）
  -- archived_at     TEXT             -- Waybackタイムスタンプ（例: 20040604075856）
);


-- =============================================================================
-- インデックス
-- =============================================================================

-- works: サイト配下の作品一覧表示（sort_order順）
CREATE INDEX IF NOT EXISTS idx_works_site_sort
  ON works(site_id, sort_order);

-- site_profiles: サイト配下のプロファイル一覧表示
CREATE INDEX IF NOT EXISTS idx_site_profiles_site
  ON site_profiles(site_id);

-- works: プロファイル削除時の ON DELETE SET NULL 対象検索
CREATE INDEX IF NOT EXISTS idx_works_site_profile
  ON works(site_profile_id);

-- pages: 作品配下のページ一覧表示（sort_order順）
CREATE INDEX IF NOT EXISTS idx_pages_work_sort
  ON pages(work_id, sort_order);

-- pages: 重複チェック・再取得時の検索
CREATE INDEX IF NOT EXISTS idx_pages_source_url
  ON pages(source_url);

-- pages: 同一作品内で同じ取得元URLを重複登録しない
CREATE UNIQUE INDEX IF NOT EXISTS uq_pages_work_source_type_url
  ON pages(work_id, source_type, source_url)
  WHERE source_url IS NOT NULL;

-- pages: 失敗ページの抽出・再取得対象の絞り込み
CREATE INDEX IF NOT EXISTS idx_pages_fetch_status
  ON pages(fetch_status);


-- =============================================================================
-- FTS5 全文検索（Phase 5 以降で有効化）
-- =============================================================================

-- pages_fts: pages.id（rowid）と紐づけてページジャンプを実現
-- content='pages'  → pages テーブルの内容を参照
-- content_rowid='id' → pages.id を rowid として使用
CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
  title,
  content_text,
  content     = 'pages',
  content_rowid = 'id',
  tokenize    = 'trigram'  -- 日本語本文の部分一致に対応
);


-- =============================================================================
-- FTS5 同期トリガー（Phase 5 以降で有効化）
-- pages テーブルの INSERT / UPDATE / DELETE に追従してFTS5を同期する
-- アプリ側実装漏れを防ぐためSQLiteトリガーで管理
-- =============================================================================

-- INSERT 時: FTS5へ追加
CREATE TRIGGER IF NOT EXISTS pages_ai
  AFTER INSERT ON pages
BEGIN
  INSERT INTO pages_fts(rowid, title, content_text)
  VALUES (new.id, new.title, new.content_text);
END;

-- DELETE 時: FTS5から削除
-- FTS5のcontentテーブル連携では削除前の値が必要なため BEFORE DELETE を使用
CREATE TRIGGER IF NOT EXISTS pages_bd
  BEFORE DELETE ON pages
BEGIN
  INSERT INTO pages_fts(pages_fts, rowid, title, content_text)
  VALUES ('delete', old.id, old.title, old.content_text);
END;

-- UPDATE 時: FTS5の旧レコードを削除して新レコードを追加
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


-- =============================================================================
-- マイグレーション管理テーブル
-- アプリ起動時にこのテーブルを確認し、未適用のマイグレーションを順番に実行する
-- =============================================================================
CREATE TABLE IF NOT EXISTS schema_migrations (
  version     TEXT    NOT NULL PRIMARY KEY,  -- 例: '001'
  applied_at  TEXT    NOT NULL               -- UTC ISO8601
);

-- このファイル自体の適用を記録
-- ※ 実際の挿入はマイグレーションランナー側で行う（二重適用防止のため）
-- INSERT INTO schema_migrations(version, applied_at) VALUES ('001', '<timestamp>');
