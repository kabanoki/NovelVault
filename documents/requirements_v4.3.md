# 小説アーカイブ・ビューアー 要件定義書 v4.3

## 0. 利用前提

本アプリは**個人利用のためのローカル保存・閲覧**を目的とする。
取得コンテンツの再配布・公開機能は持たない。

---

## 1. プロダクト概要

閉鎖されたWebサイトの小説を Wayback Machine などから取得し、ローカルに保存・閲覧・検索できるMac向けデスクトップアプリ。

「サイト → 作品 → ページ（章）」の階層で管理し、タイトル検索・全文検索・お気に入り機能に対応する。

---

## 2. ユースケース

- 閉鎖された個人サイトやブログに掲載されていた小説を、Webアーカイブ経由で取得して手元に残したい
- サイトごとに構造（HTML）が異なるため、柔軟に取得設定したい
- 取得した複数サイト・複数作品を横断的に検索して読み返したい
- **特に気に入った章・話をお気に入り登録して、すぐに読み返せるようにしたい**

---

## 3. 機能要件

### 3.1 取得機能

- 取得URLは以下の3種別を内部的に区別する：

| source_type | 内容 |
|-------------|------|
| `normal` | 通常のURL（現存サイト） |
| `wayback` | Wayback Machine URL（`https://web.archive.org/web/...`） |
| `local` | ローカルHTMLファイル（将来対応） |

- Wayback URLは相対リンク解決が特殊なため、URL種別ごとに専用の処理を行う
- 同一サイト・作品でも通常URLとWayback URLが混在する可能性があるため、`source_type` はページ単位で保持する
- 取得方法は2種類：
  - **手動取得**：URLを1つずつ指定して取得（MVP対象）
  - **一括取得**：目次ページから各話リンクを巡回して全ページ自動取得（Phase 7以降）
- 取得時はサーバ負荷を考慮し、リクエスト間隔（ウェイト）を設定できる
- 文字コードは `auto / utf-8 / shift_jis / euc-jp` に対応（古い個人サイト対策）
- MVP 0.1 では `utf-8` のみ対応とし、`auto / shift_jis` は Phase 4 で追加する
- 取得の各段階でステータスを記録し、後から失敗箇所を特定・再取得できる

### 3.2 保存・管理機能

- 階層構造：`サイト → 作品 → ページ（章）`
- 保存先：SQLite（メタデータ＋本文テキスト）+ 元HTMLはファイルとして別途保存
- 元HTMLの保存パスルール：

```
~/Library/Application Support/NovelArchiveViewer/
  app.db
  originals/
    sites/{site_id}/works/{work_id}/pages/{page_id}.html
```

- `content_html_path` にはアプリデータディレクトリからの**相対パス**を保存する（データ移行・バックアップ時の絶対パス変化への対策）
- ページ保存の順序：DBにページレコードを作成して `page_id` を採番してから、HTMLファイルを保存する
- **トランザクション境界**：HTMLファイル保存失敗時は **DBレコードをロールバックしない**。`fetch_status = save_failed` を記録して再試行できる状態にする
- 作品のリネーム・削除・並び替えが可能（`sort_order` 管理）
- 削除時の挙動：物理削除 + 確認ダイアログ（MVPでは `ON DELETE CASCADE`、将来的に `deleted_at` 検討）
- 取得元URL・取得日時・文字コード（指定値・検出値）・取得ステータスをメタデータとして記録

### 3.3 検索機能

- **タイトル検索**：サイト名・作品名・ページタイトル（Phase 2〜）
- **全文検索**：本文内のキーワード（SQLite FTS5、Phase 5以降）
- 検索結果から該当ページに直接ジャンプ可能
- FTS5仮想テーブルは `pages.id`（rowid）と紐づけ、ページジャンプを実現する
- FTS5の同期は **SQLiteトリガー** で行う（アプリ側同期は更新経路が増えた際に漏れやすいため）

### 3.4 閲覧機能

- アプリ内リーダーで本文を表示
- 作品内の前後ページにナビゲーション
- サイドバーから階層を辿って閲覧
- フォントサイズ調整（最低限）

### 3.5 お気に入り機能 ★新規追加

- **登録単位**：ページ単位（特定の章・話）
- **リストの種類**：1種類のみ（シンプル管理）。複数リスト分割は持たない
- **登録情報**：ページIDと登録日時のみ。メモ・タグ・コメントは持たない
- **表示形式**：「サイト名 → 作品名 → ページタイトル」の階層形式
  - 章番号だけでは内容が分からないため、サイト名・作品名・ページタイトルを必ず表示
- **表示順**：サイト・作品でグループ化（同一作品のお気に入りページが固まって表示される）
- **ジャンプ**：リストからワンクリックで該当ページへ飛べる
- **削除連動**：ページ削除時はお気に入りも自動削除（`ON DELETE CASCADE`）
- **重複防止**：同一ページを2回登録できない（UNIQUE制約）

---

## 4. 非機能要件

### 4.1 一般

| 項目 | 内容 |
|------|------|
| 対応OS | macOS |
| UI | MacらしいネイティブUI、軽快な動作 |
| パフォーマンス | 数百〜数千ページの本文を扱っても検索が高速 |
| データの永続性 | アプリをアンインストールしてもデータは残る形で保存 |

### 4.2 タイムスタンプ運用

- `created_at` / `updated_at` は **Rust側でセット**する
- 形式：**UTC ISO8601 文字列**（例：`2026-04-26T12:34:56.789Z`）
- SQLiteのDEFAULTやトリガーには依存しない（テスト容易性のため）

### 4.3 並び順（sort_order）の採番ルール

- 新規作成時：`MAX(sort_order) + 10` をセット
- 10刻みにすることで、間への挿入時に再採番せずに済む（`5`, `15` など）
- 並び替え操作で隙間が枯渇したら再採番（10, 20, 30, ...）

### 4.4 必須インデックス

要件レベルで以下を必須インデックスとする：

| テーブル | インデックス | 用途 |
|---------|------------|------|
| `works` | `(site_id, sort_order)` | サイト配下の作品一覧表示 |
| `pages` | `(work_id, sort_order)` | 作品配下のページ一覧表示 |
| `pages` | `(source_url)` | 重複チェック・再取得時の検索 |
| `pages` | `(fetch_status)` | 失敗ページの抽出・再取得 |
| `favorites` | `(page_id)` UNIQUE | 重複登録防止・お気に入り判定 |

### 4.5 マイグレーション機構

- スキーマ変更に対応するため、**マイグレーション機構を持つ**
- `001_initial_schema.sql` から始めるバージョン管理形式
- 採用ツールは詳細設計で確定（候補：refinery / sqlx-migrate）

---

## 5. 想定するサイト構造のパターン

| パターン | 構造 | 例 |
|---------|------|-----|
| 目次ページ型 | 目次ページに各話リンクが並ぶ | 個人サイト全般 |
| 連番URL型 | `001.html`, `002.html` のように連番 | 古い個人サイト |
| ブログ型 | 記事一覧から各エントリへ | fc2blog 等 |
| 単一ページ型 | 1ページに作品全文 | - |

→ サイトごとの **サイトプロファイル（JSON）** で吸収する。

---

## 6. 技術スタック

| 領域 | 採用技術 |
|------|---------|
| フレームワーク | Tauri |
| フロントエンド | React + TypeScript |
| スタイリング | Tailwind CSS |
| バックエンド | Rust |
| HTTPクライアント | reqwest（Rust） |
| HTMLパース | scraper（Rust） |
| データベース | SQLite |
| 全文検索 | SQLite FTS5 |

---

## 7. 画面構成（案）

```
┌─────────────────────────────────────────────────┐
│  [+ サイト追加]  [検索バー]  [★お気に入り]      │
├──────────────┬──────────────────────────────────┤
│ サイドバー     │  メインエリア                    │
│              │                                  │
│ ▼ サイトA     │   作品タイトル                    │
│   ▼ 作品1    │   ─────────                    │
│     第1話 ★  │                                  │
│     第2話    │   本文表示                        │
│   ▶ 作品2    │   [★お気に入り登録/解除]         │
│ ▶ サイトB     │                                  │
│              │   [前のページ] [次のページ]        │
└──────────────┴──────────────────────────────────┘
```

主要画面（最終形）：
1. **メイン画面**：サイドバー＋本文ビューワー
2. **サイト登録画面**：URL・セレクタ・取得設定
3. **取得進捗画面**：一括取得時の進捗表示（Phase 7以降）
4. **検索結果画面**：タイトル／本文ヒット箇所の一覧
5. **お気に入り一覧画面** ★新規：サイト・作品でグループ化された一覧

### お気に入り一覧画面イメージ

```
★ お気に入り一覧

▼ サイトA
  ▼ 作品1
    ・第1話 「出会い」
    ・第5話 「決意」
  ▼ 作品2
    ・第3話 「再会」

▼ サイトB
  ▼ 作品3
    ・最終話 「旅立ち」
```

各行クリックで該当ページへジャンプ。

---

## 8. データモデル

```sql
-- サイト
sites
  - id          INTEGER PRIMARY KEY
  - name        TEXT NOT NULL
  - base_url    TEXT NOT NULL
  - created_at  TEXT NOT NULL  -- UTC ISO8601, Rust側でセット
  - updated_at  TEXT NOT NULL

-- サイトプロファイル
site_profiles
  - id            INTEGER PRIMARY KEY
  - site_id       INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE
  - name          TEXT NOT NULL
  - profile_json  TEXT NOT NULL
  - created_at    TEXT NOT NULL
  - updated_at    TEXT NOT NULL

-- 作品
works
  - id               INTEGER PRIMARY KEY
  - site_id          INTEGER NOT NULL REFERENCES sites(id) ON DELETE CASCADE
  - site_profile_id  INTEGER REFERENCES site_profiles(id) ON DELETE SET NULL
  - title            TEXT NOT NULL
  - author_name      TEXT
  - description      TEXT
  - source_url       TEXT
  - sort_order       INTEGER NOT NULL
  - created_at       TEXT NOT NULL
  - updated_at       TEXT NOT NULL

-- ページ（章）
pages
  - id                  INTEGER PRIMARY KEY
  - work_id             INTEGER NOT NULL REFERENCES works(id) ON DELETE CASCADE
  - page_number         INTEGER          -- サイト上の章番号
  - sort_order          INTEGER NOT NULL -- アプリ内並び順
  - title               TEXT
  - source_url          TEXT
  - source_type         TEXT NOT NULL    -- normal / wayback / local
  - requested_encoding  TEXT
  - detected_encoding   TEXT
  - content_text        TEXT
  - content_html_path   TEXT
  - fetch_status        TEXT NOT NULL    -- pending / success / fetch_failed / parse_failed / save_failed / skipped
  - fetch_error         TEXT
  - fetched_at          TEXT
  - created_at          TEXT NOT NULL
  - updated_at          TEXT NOT NULL
  - UNIQUE(work_id, source_type, source_url) WHERE source_url IS NOT NULL

-- お気に入り ★新規
favorites
  - id          INTEGER PRIMARY KEY
  - page_id     INTEGER NOT NULL UNIQUE REFERENCES pages(id) ON DELETE CASCADE
  - created_at  TEXT NOT NULL

-- 全文検索用（FTS5仮想テーブル）Phase 5以降
CREATE VIRTUAL TABLE pages_fts USING fts5(
  title,
  content_text,
  content='pages',
  content_rowid='id',
  tokenize='trigram'
);
```

### page_number と sort_order の使い分け

| カラム | 意味 | 例 |
|--------|------|-----|
| `page_number` | サイト上の章番号（取得元の番号をそのまま記録） | 「第3話」→ `3`、「番外編」→ `NULL` |
| `sort_order` | アプリ内での並び順（ユーザーが並び替え可能） | `10`, `20`, `30`, ... |

### fetch_status 定義

| ステータス | 意味 | 主に使われるフェーズ |
|-----------|------|--------------------|
| `pending` | 未取得（事前登録のみ） | Phase 7以降の一括取得 |
| `success` | 取得・抽出・保存すべて成功 | 全フェーズ |
| `fetch_failed` | HTTP取得失敗 | 全フェーズ |
| `parse_failed` | HTML解析・本文抽出失敗 | 全フェーズ |
| `save_failed` | DBレコード作成後の保存処理失敗 | 全フェーズ |
| `skipped` | スキップ | Phase 7以降 |

### お気に入りテーブル設計の補足

| 判断 | 理由 |
|------|------|
| `page_id` に UNIQUE 制約 | 同一ページの重複登録を防ぐ |
| `ON DELETE CASCADE` | ページ削除時にお気に入りも自動削除 |
| メモ・タグカラムを持たない | シンプル管理が要件のため。将来の拡張時にカラム追加で対応 |
| 単一リスト | 複数リスト分割は要件外。将来 `favorite_lists` テーブルで拡張可能 |

---

## 9. サイトプロファイル JSON 仕様

```json
{
  "name": "サイト名",
  "base_url": "https://example.com/",
  "source_type": "normal",
  "encoding": "auto",
  "index_pattern": {
    "url": "https://example.com/index.html",
    "link_selector": "a.chapter-link",
    "link_url_pattern": null
  },
  "page_pattern": {
    "title_selector": "h1",
    "content_selector": "div#honbun",
    "remove_selectors": ["script", "style", ".footer", ".nav"]
  },
  "fetch_options": {
    "interval_ms": 1000,
    "user_agent": "NovelArchiveViewer/0.1"
  }
}
```

---

## 10. Tauriコマンド一覧

### サイト管理
| コマンド | 説明 |
|---------|------|
| `siteCreate` | サイトを新規登録 |
| `siteList` | サイト一覧を取得 |
| `siteUpdate` | サイト情報を更新 |
| `siteDelete` | サイトを削除（CASCADE） |

### サイトプロファイル管理
| コマンド | 説明 |
|---------|------|
| `siteProfileCreate` | プロファイルを新規登録 |
| `siteProfileList` | サイトIDでプロファイル一覧 |
| `siteProfileUpdate` | プロファイルを更新 |
| `siteProfileDelete` | プロファイルを削除 |

### 作品管理
| コマンド | 説明 |
|---------|------|
| `workCreate` | 作品を新規登録 |
| `workListBySite` | サイトIDで作品一覧を取得 |
| `workUpdate` | 作品情報を更新 |
| `workDelete` | 作品を削除（CASCADE） |

### ページ管理
| コマンド | 説明 |
|---------|------|
| `pageCreate` | ページを新規登録 |
| `pageListByWork` | 作品IDでページ一覧を取得 |
| `pageGet` | ページ1件を取得 |
| `pageUpdate` | ページ情報を更新 |
| `pageDelete` | ページを削除 |

### 取得・保存
| コマンド | 説明 |
|---------|------|
| `fetchPageByUrl` | URLからHTMLを取得・本文抽出・DBレコード作成・HTMLファイル保存 |

### 検索
| コマンド | 説明 |
|---------|------|
| `searchTitles` | タイトル検索（Phase 2〜） |
| `searchFullText` | 全文検索（Phase 5〜） |
| `rebuildSearchIndex` | FTS5インデックスを再構築（Phase 5〜） |

### お気に入り ★新規
| コマンド | 説明 |
|---------|------|
| `favoriteAdd` | ページをお気に入りに追加 |
| `favoriteRemove` | お気に入りから削除 |
| `favoriteList` | お気に入り一覧を取得（サイト・作品でグループ化済み） |
| `favoriteCheck` | 特定ページがお気に入り済みか確認 |

---

## 11. 開発フェーズ（MVP優先順）

| フェーズ | 内容 | 備考 |
|---------|------|------|
| Phase 1 | Tauri + React + SQLite 接続・基本UI・マイグレーション機構 | 環境構築 |
| Phase 2 | サイト・作品・ページを手動登録できる | CRUD・タイトル検索 |
| Phase 3 | 手動URL指定で本文取得・抽出・保存（utf-8） | コア機能 |
| Phase 4 | 元HTML保存・auto/shift_jis 文字コード対応 | オフライン対応 |
| Phase 5 | SQLite FTS5 全文検索・トリガー同期 | 検索機能 |
| Phase 6 | サイトプロファイル保存・切替 | プロファイル管理 |
| **Phase 6.5** | **お気に入り機能（追加・削除・一覧）** ★新規 | 軽量機能のため早期投入可能 |
| Phase 7 | 目次ページから一括取得 | 自動化 |
| Phase 8 | Wayback特有URL対応・`canonical_url`/`archived_at` 追加 | アーカイブ対応 |
| Phase 9 | 残りの文字コード対応（euc-jp 等） | 古いサイト対応 |
| Phase 10 | エクスポート・バックアップ・設定画面 | MVPではページ本文のテキスト出力、SQLite DBバックアップ、重複URL診断、リーダーフォントサイズ設定に対応 |

※お気に入り機能は実装が軽量なため、Phase 5 完了後すぐに投入できる。Phase 2 完了時点でも投入可能だが、検索機能との UI 統合を考慮して Phase 6.5 に配置。

### MVP 0.1 のゴール

```
URLを1つ入力する
    ↓
title_selector / content_selector を指定する
    ↓
本文を取得する（fetch_status を記録）
    ↓
DBレコードを作成 → 相対パスでHTMLを保存
    ↓
アプリ内で読める・タイトル検索できる
```

---

## 12. 詳細設計フェーズの成果物と順序

| 順序 | 成果物 | 状態 |
|------|--------|------|
| 1 | `001_initial_schema.sql` | 作成済み |
| 1.5 | `002_add_favorites.sql` ★新規 | 本要件で作成 |
| 2 | Tauriコマンド型定義（TS / Rust） | 作成済み |
| 2.5 | お気に入り用型定義の追加 ★新規 | 本要件で作成 |
| 3 | 画面一覧・画面遷移図 | 未着手 |
| 4 | Reactコンポーネント構成 | 未着手 |
| 5 | Rust側取得・保存処理フロー | 未着手 |
| 6 | サイトプロファイルJSON仕様確定 | 未着手 |

---

## 13. 未決事項・今後の検討項目

- [ ] サイトプロファイル設定UIの形（GUIで要素ピッカー or JSON直接編集）
- [ ] `link_url_pattern` の具体的な書式（正規表現 or テンプレート文字列）
- [ ] 画像の扱い（小説中の挿絵など）
- [ ] 誤削除対策（将来的に `deleted_at` による論理削除を検討）
- [x] エクスポート機能（テキスト出力など）の要否 → MVPでは選択中ページの本文を `.txt` 出力する
- [x] アプリ全体のグローバル設定保存先（`app_settings` テーブル or 設定ファイル） → MVPではリーダー表示設定を `localStorage` に保存し、DB同期が必要な設定は後続で再検討する
- [ ] マイグレーションツールの選定（refinery / sqlx-migrate 等）
- [ ] お気に入り機能の将来拡張（複数リスト・メモ・タグ）の要否

---

## 14. 対象サイト例（動作確認用）

| URL | パターン | 状態 |
|-----|---------|------|
| `http://rinrin.saiin.net/~yokubounotou/ss/slmbst/index01.html` | 目次ページ型 | 現存 |
| `https://web.archive.org/web/20040604075856/http://www.onyx.dti.ne.jp/~sultan/Dcup000.html` | 連番型 | Wayback |
| `http://www.ni.bekkoame.ne.jp/ruly/kansei.html` | 個人サイト | 現存（要動作確認） |
| `https://web.archive.org/web/20120124192301/http://sakaimaroudo.blog118.fc2.com/blog-entry-3.html` | ブログ型 | Wayback |

---

## 変更履歴

| バージョン | 変更内容 |
|-----------|---------|
| v4.2 | 初期確定版（タイムスタンプ運用、sort_order採番、必須インデックス、トランザクション境界、マイグレーション機構を明記） |
| v4.3 | お気に入り機能を追加（ページ単位、単一リスト、サイト・作品グループ化、ワンクリックジャンプ） |
