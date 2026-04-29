# 単体試験書

対応要件定義: v4.3  
対応設計書: fetch_storage_flow.md / screen_design.md

---

## 概要

Rustバックエンドの純関数（入出力のみ、DB・HTTP・ファイルI/Oなし）を対象とした単体試験。  
テストツール: Rust標準テスト (`cargo test`)

---

## テスト対象一覧

| ID | 対象関数 | テスト観点 |
|----|---------|-----------|
| U-01 | `now_utc()` | UTC ISO8601形式の文字列が返ること |
| U-02 | `normalize_text()` | 連続空白・タブが単一スペースに整形されること |
| U-03 | `normalize_lines()` | 空行除去・各行トリム・改行で結合されること |
| U-04 | `build_fts_query()` | FTS5クエリが正しく構築されること |
| U-05 | `parse_wayback_url()` | 正常なWayback URLからタイムスタンプと元URLを抽出できること |
| U-06 | `parse_wayback_url()` | 不正なURLでINVALID_WAYBACK_URLエラーが返ること |
| U-07 | `charset_from_content_type()` | Content-Typeヘッダからcharsetを抽出できること |
| U-08 | `charset_from_content_type()` | charsetなし・None入力でNoneが返ること |
| U-09 | `charset_from_html_head()` | HTMLのmetaタグからcharsetを抽出できること |
| U-10 | `validate_source_type()` | 正常値 (normal/wayback/local) でOkが返ること |
| U-11 | `validate_source_type()` | 不正値でVALIDATION_ERRORが返ること |
| U-12 | `normalize_requested_encoding()` | 各エンコーディング表記を正規化できること |
| U-13 | `normalize_requested_encoding()` | 未対応エンコーディングでUNSUPPORTED_ENCODINGが返ること |
| U-14 | `sanitize_filename()` | ファイル名に使えない文字が除去・置換されること |
| U-15 | `sanitize_filename()` | 空文字入力で"untitled"が返ること |
| U-16 | `apply_link_template()` | `{n}` テンプレートが正しく展開されること |
| U-17 | `apply_link_template()` | `{n:03d}` テンプレートがゼロ埋めで展開されること |
| U-18 | `apply_link_template()` | プレースホルダーなしテンプレートでVALIDATION_ERRORが返ること |
| U-19 | `group_favorites()` | 平坦なリストがサイト・作品でグループ化されること |
| U-20 | `group_favorites()` | 空リストで空のgroupsが返ること |
| U-21 | `extract_page_content()` | 正常HTMLからタイトルと本文が抽出されること |
| U-22 | `extract_page_content()` | 不正なCSSセレクタでINVALID_SELECTORエラーが返ること |
| U-23 | `extract_page_content()` | 本文セレクタ不一致でPARSE_ERRORが返ること |
| U-24 | `extract_page_content()` | remove_selectorsで指定した要素のテキストが除外されること |

---

## テストケース詳細

### U-01: now_utc() 形式検証

**前提条件**: なし  
**操作**: `now_utc()` を呼び出す  
**期待結果**: `\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z` にマッチすること

---

### U-02: normalize_text() 空白整形

**前提条件**: なし  
**入力**: `"  hello   world  "`, `"a\t\tb"`, `"abc"`  
**期待結果**: それぞれ `"hello world"`, `"a b"`, `"abc"`

---

### U-03: normalize_lines() 行整形

**前提条件**: なし  
**入力**: `"  hello  \n\n  world  \n"`, 空文字列  
**期待結果**: `"hello\nworld"`, `""`

---

### U-04: build_fts_query() FTSクエリ構築

**前提条件**: なし  
**入力**:
- 単語1つ: `"keyword"`
- 複数語: `"hello world"`
- ダブルクォート含む: `"key\"word"`
- 空文字: `""`

**期待結果**:
- `"\"keyword\""`
- `"\"hello\" AND \"world\""`
- `"\"key\"\"word\""`
- `""`

---

### U-05: parse_wayback_url() 正常系

**前提条件**: なし  
**入力**: `"https://web.archive.org/web/20040604075856/http://example.com/001.html"`  
**期待結果**: `archived_at = Some("20040604075856")`, `canonical_url = Some("http://example.com/001.html")`

---

### U-06: parse_wayback_url() 異常系

**前提条件**: なし  
**入力**: `"https://example.com/normal-url"`, `"https://web.archive.org/web/"`  
**期待結果**: `code = "INVALID_WAYBACK_URL"` のエラー

---

### U-07: charset_from_content_type() 正常系

**前提条件**: なし  
**入力**: `Some("text/html; charset=shift_jis")`, `Some("text/html; charset=\"utf-8\"")`  
**期待結果**: `Some("shift_jis")`, `Some("utf-8")`

---

### U-08: charset_from_content_type() Noneケース

**前提条件**: なし  
**入力**: `None`, `Some("text/html")`  
**期待結果**: `None`

---

### U-09: charset_from_html_head() メタタグ検出

**前提条件**: なし  
**入力**: `<meta charset="shift_jis">`, `<meta http-equiv="content-type" content="text/html; charset=euc-jp">`  
**期待結果**: `Some("shift_jis")`, `Some("euc-jp")`

---

### U-10: validate_source_type() 正常値

**入力**: `"normal"`, `"wayback"`, `"local"`  
**期待結果**: `Ok(())`

---

### U-11: validate_source_type() 異常値

**入力**: `"unknown"`, `""`, `"NORMAL"`  
**期待結果**: `Err` with `code = "VALIDATION_ERROR"`

---

### U-12: normalize_requested_encoding() 正規化

**入力**: `"utf8"`, `"UTF-8"`, `"shift_jis"`, `"Shift-JIS"`, `"sjis"`, `"euc-jp"`, `"auto"`  
**期待結果**: それぞれ `"utf-8"`, `"utf-8"`, `"shift_jis"`, `"shift_jis"`, `"shift_jis"`, `"euc-jp"`, `"auto"`

---

### U-13: normalize_requested_encoding() 未対応

**入力**: `"iso-2022-jp"`, `"ascii"`, `""`  
**期待結果**: `Err` with `code = "UNSUPPORTED_ENCODING"`

---

### U-14: sanitize_filename() 特殊文字除去

**入力**: `"サイトA/作品1"`, `"hello world"`, `"a!b@c#d"`  
**期待結果**: `"_A__1"` の形（ASCII英数字・`-`・`_` 以外は `_` に置換）, `"hello_world"`, `"a_b_c_d"`

---

### U-15: sanitize_filename() 空入力

**入力**: `""`, `"!!!"` （特殊文字のみ）  
**期待結果**: `"untitled"`

---

### U-16: apply_link_template() {n} 展開

**入力**: テンプレート `"http://example.com/{n}.html"`, n=5  
**期待結果**: `"http://example.com/5.html"`

---

### U-17: apply_link_template() {n:03d} ゼロ埋め展開

**入力**: テンプレート `"http://example.com/{n:03d}.html"`, n=5  
**期待結果**: `"http://example.com/005.html"`

---

### U-18: apply_link_template() プレースホルダーなし

**入力**: テンプレート `"http://example.com/fixed.html"`, n=1  
**期待結果**: `Err` with `code = "VALIDATION_ERROR"`

---

### U-19: group_favorites() グループ化

**前提条件**: 同サイト・同作品のFavoriteListItem 2件、別サイト1件を用意  
**期待結果**: `groups.len() == 2`、第1グループに `works[0].pages.len() == 2`

---

### U-20: group_favorites() 空リスト

**入力**: 空Vec  
**期待結果**: `groups.len() == 0`

---

### U-21: extract_page_content() 正常抽出

**入力**: HTML `<h1>タイトル</h1><div id="content">本文テキスト</div>`, セレクタ `h1`, `#content`  
**期待結果**: `title == Some("タイトル")`, `content_text == "本文テキスト"`

---

### U-22: extract_page_content() 不正セレクタ

**入力**: セレクタ `"!!invalid##"` (タイトル用)  
**期待結果**: `Err` with `code = "INVALID_SELECTOR"`

---

### U-23: extract_page_content() セレクタ不一致

**入力**: HTML に `#missing` セレクタが存在しない場合  
**期待結果**: `Err` with `code = "PARSE_ERROR"`

---

### U-24: extract_page_content() 除外セレクタ

**入力**: HTML `<div id="content">本文<span class="nav">ナビ</span></div>`, remove_selectors `[".nav"]`  
**期待結果**: `content_text` に "ナビ" が含まれないこと

---

## 試験結果記録

実施日: 2026-04-28  
実施方法: `cargo test` (Rust 1.95.0)  
結果サマリ: **24/24 PASS**

| ID | 結果 | テスト関数 |
|----|------|-----------|
| U-01 | PASS | `test_u01_now_utc_format` |
| U-02 | PASS | `test_u02_normalize_text` |
| U-03 | PASS | `test_u03_normalize_lines` |
| U-04 | PASS | `test_u04_build_fts_query` |
| U-05 | PASS | `test_u05_parse_wayback_url_valid` |
| U-06 | PASS | `test_u06_parse_wayback_url_invalid` |
| U-07 | PASS | `test_u07_charset_from_content_type` |
| U-08 | PASS | `test_u08_charset_from_content_type_none` |
| U-09 | PASS | `test_u09_charset_from_html_head` |
| U-10 | PASS | `test_u10_validate_source_type_valid` |
| U-11 | PASS | `test_u11_validate_source_type_invalid` |
| U-12 | PASS | `test_u12_normalize_encoding` |
| U-13 | PASS | `test_u13_normalize_encoding_unsupported` |
| U-14 | PASS | `test_u14_sanitize_filename` |
| U-15 | PASS | `test_u15_sanitize_filename_empty` |
| U-16 | PASS | `test_u16_link_template_n` |
| U-17 | PASS | `test_u17_link_template_zero_pad` |
| U-18 | PASS | `test_u18_link_template_no_placeholder` |
| U-19 | PASS | `test_u19_group_favorites` |
| U-20 | PASS | `test_u20_group_favorites_empty` |
| U-21 | PASS | `test_u21_extract_page_content_normal` |
| U-22 | PASS | `test_u22_extract_page_content_invalid_selector` |
| U-23 | PASS | `test_u23_extract_page_content_no_match` |
| U-24 | PASS | `test_u24_extract_page_content_remove_selectors` |
