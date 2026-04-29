# 結合試験書

対応要件定義: v4.3  
対応設計書: 001_initial_schema.sql / 002_add_favorites.sql / 003_add_wayback_metadata.sql

---

## 概要

インメモリSQLiteを使ったコマンド間の連携・データの整合性を検証する結合試験。  
テストツール: Rust標準テスト (`cargo test`)

---

## テスト対象一覧

| ID | テスト観点 |
|----|-----------|
| I-01 | サイト作成・一覧取得 |
| I-02 | サイト更新 |
| I-03 | サイト削除 |
| I-04 | サイト削除時の作品CASCADE確認 |
| I-05 | 作品作成・一覧取得 |
| I-06 | 作品作成時のsort_order採番（10刻み） |
| I-07 | 作品更新 |
| I-08 | 作品削除時のページCASCADE確認 |
| I-09 | ページ作成・一覧取得・1件取得 |
| I-10 | ページのsort_order採番（10刻み） |
| I-11 | ページ更新（content_text → fetch_status自動判定） |
| I-12 | ページ削除 |
| I-13 | お気に入り追加・判定 |
| I-14 | お気に入り重複防止（UNIQUE制約） |
| I-15 | お気に入り一覧取得（サイト・作品でグループ化） |
| I-16 | お気に入り削除 |
| I-17 | ページ削除時のお気に入りCASCADE確認 |
| I-18 | タイトル検索（サイト・作品・ページ横断） |
| I-19 | タイトル検索の大文字小文字非区別 |
| I-20 | 空クエリのタイトル検索 |
| I-21 | Waybackメタデータの保存・取得 |
| I-22 | バリデーション: サイト名が空でVALIDATION_ERROR |
| I-23 | バリデーション: 作品名が空でVALIDATION_ERROR |
| I-24 | バリデーション: 不正source_typeでVALIDATION_ERROR |
| I-25 | NOT_FOUND: 存在しないサイトIDで作品作成するとNOT_FOUND |
| I-26 | NOT_FOUND: 存在しないIDでsiteUpdateするとNOT_FOUND |
| I-27 | サイトプロファイルCRUD |
| I-28 | プロファイルJSONバリデーション（不正JSON） |
| I-29 | プロファイル削除時に作品のsite_profile_idがNULLになること |

---

## テストケース詳細

### I-01: サイト作成・一覧取得

**前提**: インメモリDBにマイグレーション済み  
**操作**:
1. `site_create({ name: "サイトA", base_url: "https://a.example.com" })` 実行
2. `site_list()` 実行  

**期待結果**:
- 作成時: `Site.name == "サイトA"`, `id > 0`, `created_at` がUTC ISO8601形式
- 一覧: サイトが1件返り、内容が一致すること

---

### I-02: サイト更新

**前提**: サイトが1件存在  
**操作**: `site_update({ id, name: "サイトB", base_url: "https://b.example.com" })` 実行  
**期待結果**: 更新後の `Site.name == "サイトB"`, `updated_at` が変化していること

---

### I-03: サイト削除

**前提**: サイトが1件存在  
**操作**: `site_delete({ id })` 実行  
**期待結果**: `site_list()` が空配列を返すこと

---

### I-04: サイト削除時の作品CASCADEデリート

**前提**: サイト1件・作品2件・ページ3件が存在  
**操作**: `site_delete({ id: site_id })` 実行  
**期待結果**: `works` テーブルと `pages` テーブルの関連レコードが全て削除されること

---

### I-05: 作品作成・一覧取得

**前提**: サイトが1件存在  
**操作**:
1. `work_create({ site_id, title: "作品1", author_name: "著者A", ... })` 実行
2. `work_list_by_site({ site_id })` 実行  

**期待結果**: 作品が1件返り、各フィールドが一致すること

---

### I-06: 作品のsort_order採番

**前提**: サイトが1件存在  
**操作**: 作品を3件連続で作成  
**期待結果**: sort_orderが `10, 20, 30` の順で採番されること

---

### I-07: 作品更新

**前提**: 作品が1件存在  
**操作**: `work_update({ id, title: "新タイトル", ... })` 実行  
**期待結果**: タイトルが更新され `updated_at` が変化すること

---

### I-08: 作品削除時のページCASCADE

**前提**: 作品1件・ページ2件が存在  
**操作**: `work_delete({ id: work_id })` 実行  
**期待結果**: `pages` テーブルの関連レコードが削除されること

---

### I-09: ページ作成・一覧・1件取得

**前提**: 作品が1件存在  
**操作**:
1. `page_create({ work_id, title: "第1話", source_type: "normal", ... })` 実行
2. `page_list_by_work({ work_id })` 実行
3. `page_get({ id })` 実行  

**期待結果**: 各操作で正しいページデータが返ること。初期 `fetch_status == "pending"`

---

### I-10: ページのsort_order採番

**前提**: 作品が1件存在  
**操作**: ページを3件連続で作成  
**期待結果**: sort_orderが `10, 20, 30` の順で採番されること

---

### I-11: ページ更新のfetch_status自動判定

**前提**: ページが1件存在 (`fetch_status = pending`)  
**操作**:
1. `page_update({ ..., content_text: "本文あり" })` → `fetch_status == "success"` になること
2. `page_update({ ..., content_text: None })` → `fetch_status == "pending"` になること

---

### I-12: ページ削除

**前提**: ページが1件存在  
**操作**: `page_delete({ id })` 実行  
**期待結果**: `page_list_by_work` が空を返すこと

---

### I-13: お気に入り追加・判定

**前提**: ページが1件存在  
**操作**:
1. `favorite_add({ page_id })` 実行
2. `favorite_check({ page_id })` 実行  

**期待結果**: `isFavorite == true`, `favoriteId != null`

---

### I-14: お気に入り重複防止

**前提**: ページが1件存在、お気に入り1件登録済み  
**操作**: 同じpage_idで `favorite_add` を再実行  
**期待結果**: エラーにならず、レコードが重複しないこと（ON CONFLICT DO NOTHING）

---

### I-15: お気に入り一覧取得（グループ化）

**前提**: サイト2件・作品2件・ページ4件・お気に入り3件が存在  
**操作**: `favorite_list()` 実行  
**期待結果**:
- `groups` がサイト数分返ること
- 各グループ内に `works` が存在し、各`works`に`pages`が存在すること
- 順序が `sites.name COLLATE NOCASE, works.sort_order, pages.sort_order` 順であること

---

### I-16: お気に入り削除

**前提**: お気に入りが1件存在  
**操作**: `favorite_remove({ page_id })` 実行  
**期待結果**: `favorite_check({ page_id }).isFavorite == false`

---

### I-17: ページ削除時のお気に入りCASCADE

**前提**: ページ1件・お気に入り1件が存在  
**操作**: `page_delete({ id: page_id })` 実行  
**期待結果**: `favorites` テーブルのレコードが自動削除されること

---

### I-18: タイトル検索（横断）

**前提**: サイト・作品・ページが複数存在  
**操作**: `search_titles({ query: "キーワード" })` 実行  
**期待結果**: `sites`, `works`, `pages` に各ヒット件数が返ること

---

### I-19: タイトル検索の大文字小文字非区別

**前提**: `name = "SiteAlpha"` のサイトが存在  
**操作**: `search_titles({ query: "sitealpha" })` 実行  
**期待結果**: 該当サイトがヒットすること

---

### I-20: 空クエリのタイトル検索

**操作**: `search_titles({ query: "" })` 実行  
**期待結果**: `sites = []`, `works = []`, `pages = []` が返ること（DB検索は行わない）

---

### I-21: Waybackメタデータの保存・取得

**前提**: 作品が1件存在  
**操作**: `page_create({ source_type: "wayback", source_url: "https://web.archive.org/web/20040604075856/http://example.com/001.html", ... })` 実行  
**期待結果**: `canonical_url == "http://example.com/001.html"`, `archived_at == "20040604075856"` が保存されること

---

### I-22: バリデーション: サイト名が空

**操作**: `site_create({ name: "", base_url: "https://example.com" })` 実行  
**期待結果**: `code == "VALIDATION_ERROR"`

---

### I-23: バリデーション: 作品名が空

**前提**: サイトが1件存在  
**操作**: `work_create({ site_id, title: "  ", ... })` 実行 （空白のみ）  
**期待結果**: `code == "VALIDATION_ERROR"`

---

### I-24: バリデーション: 不正source_type

**前提**: 作品が1件存在  
**操作**: `page_create({ ..., source_type: "ftp" })` 実行  
**期待結果**: `code == "VALIDATION_ERROR"`

---

### I-25: NOT_FOUND: 存在しないサイトIDで作品作成

**操作**: `work_create({ site_id: 99999, title: "test" })` 実行  
**期待結果**: `code == "NOT_FOUND"`

---

### I-26: NOT_FOUND: 存在しないIDでsite_update

**操作**: `site_update({ id: 99999, name: "x", base_url: "https://x.example.com" })` 実行  
**期待結果**: `code == "NOT_FOUND"`

---

### I-27: サイトプロファイルCRUD

**前提**: サイトが1件存在  
**操作**:
1. `site_profile_create({ site_id, name: "標準", profile_json: "{}" })` 実行
2. `site_profile_list({ site_id })` 実行
3. `site_profile_update({ id, name: "更新済み", profile_json: "{\"encoding\":\"utf-8\"}" })` 実行
4. `site_profile_delete({ id })` 実行  

**期待結果**: 各操作が正常に完了し、削除後に一覧が空になること

---

### I-28: プロファイルJSONバリデーション

**前提**: サイトが1件存在  
**操作**: `site_profile_create({ ..., profile_json: "not-json" })` 実行  
**期待結果**: `code == "VALIDATION_ERROR"`

---

### I-29: プロファイル削除時の作品site_profile_idがNULL化

**前提**: サイト・プロファイル・作品（site_profile_idを持つ）が存在  
**操作**: `site_profile_delete({ id: profile_id })` 実行  
**期待結果**: 作品の `site_profile_id == NULL` となり、作品自体は削除されないこと

---

## 試験結果記録

実施日: 2026-04-28  
実施方法: `cargo test` (Rust 1.95.0, インメモリSQLite)  
結果サマリ: **29/29 PASS**

| ID | 結果 | テスト関数 |
|----|------|-----------|
| I-01 | PASS | `test_i01_site_create_and_list` |
| I-02 | PASS | `test_i02_site_update` |
| I-03 | PASS | `test_i03_site_delete` |
| I-04 | PASS | `test_i04_site_cascade_delete` |
| I-05 | PASS | `test_i05_work_create_and_list` |
| I-06 | PASS | `test_i06_work_sort_order_increments` |
| I-07 | PASS | `test_i07_work_update` |
| I-08 | PASS | `test_i08_work_cascade_delete` |
| I-09 | PASS | `test_i09_page_create_list_get` |
| I-10 | PASS | `test_i10_page_sort_order_increments` |
| I-11 | PASS | `test_i11_fetch_status_auto` |
| I-12 | PASS | `test_i12_page_delete` |
| I-13 | PASS | `test_i13_favorite_add_and_check` |
| I-14 | PASS | `test_i14_favorite_no_duplicate` |
| I-15 | PASS | `test_i15_favorite_list_grouped` |
| I-16 | PASS | `test_i16_favorite_remove` |
| I-17 | PASS | `test_i17_favorite_cascade_on_page_delete` |
| I-18 | PASS | `test_i18_search_titles` |
| I-19 | PASS | `test_i19_search_case_insensitive` |
| I-20 | PASS | `test_i20_search_empty_query` |
| I-21 | PASS | `test_i21_wayback_metadata_stored` |
| I-22 | PASS | `test_i22_site_name_empty_validation` |
| I-23 | PASS | `test_i23_work_title_empty_validation` |
| I-24 | PASS | `test_i24_invalid_source_type` |
| I-25 | PASS | `test_i25_not_found_site_for_work` |
| I-26 | PASS | `test_i26_not_found_site_update` |
| I-27 | PASS | `test_i27_site_profile_crud` |
| I-28 | PASS | `test_i28_profile_json_validation` |
| I-29 | PASS | `test_i29_profile_delete_nullifies_work` |
