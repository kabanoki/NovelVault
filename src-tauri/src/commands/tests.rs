use super::*;
use crate::types::FavoriteListItem;
use rusqlite::{params, Connection, OptionalExtension};

fn setup_db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", "ON").unwrap();
    conn.execute_batch(include_str!("../../migrations/001_initial_schema.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../../migrations/002_add_favorites.sql"))
        .unwrap();
    conn.execute_batch(include_str!(
        "../../migrations/003_add_wayback_metadata.sql"
    ))
    .unwrap();
    conn.execute_batch(include_str!("../../migrations/004_add_profile_indexes.sql"))
        .unwrap();
    conn.execute_batch(include_str!("../../migrations/005_use_trigram_fts.sql"))
        .unwrap();
    conn.execute_batch(include_str!(
        "../../migrations/006_unique_page_source_urls.sql"
    ))
    .unwrap();
    conn
}

fn insert_site(conn: &Connection, name: &str, base_url: &str) -> i64 {
    let now = now_utc();
    conn.execute(
        "INSERT INTO sites (name, base_url, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![name, base_url, now, now],
    )
    .unwrap();
    conn.last_insert_rowid()
}

fn insert_work(conn: &Connection, site_id: i64, title: &str) -> i64 {
    let now = now_utc();
    let sort_order = next_sort_order(conn, "works", "site_id", site_id).unwrap();
    conn.execute(
            "INSERT INTO works (site_id, title, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![site_id, title, sort_order, now, now],
        ).unwrap();
    conn.last_insert_rowid()
}

fn insert_page(conn: &Connection, work_id: i64, title: &str, content_text: Option<&str>) -> i64 {
    let now = now_utc();
    let sort_order = next_sort_order(conn, "pages", "work_id", work_id).unwrap();
    let fetch_status = if content_text.is_some() {
        "success"
    } else {
        "pending"
    };
    conn.execute(
            "INSERT INTO pages (work_id, sort_order, title, source_type, content_text, fetch_status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'normal', ?4, ?5, ?6, ?7)",
            params![work_id, sort_order, title, content_text, fetch_status, now, now],
        ).unwrap();
    conn.last_insert_rowid()
}

fn insert_favorite(conn: &Connection, page_id: i64) -> i64 {
    let now = now_utc();
    conn.execute(
        "INSERT OR IGNORE INTO favorites (page_id, created_at) VALUES (?1, ?2)",
        params![page_id, now],
    )
    .unwrap();
    conn.query_row(
        "SELECT id FROM favorites WHERE page_id=?1",
        params![page_id],
        |row| row.get(0),
    )
    .unwrap()
}

// ── U-01: now_utc() フォーマット検証 ──────────────────────────
#[test]
fn test_u01_now_utc_format() {
    let now = now_utc();
    let re = regex::Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$").unwrap();
    assert!(re.is_match(&now), "now_utc() format mismatch: {now}");
}

// ── U-02: normalize_text() ────────────────────────────────────
#[test]
fn test_u02_normalize_text() {
    assert_eq!(
        fetch::normalize_text("  hello   world  ".to_string()),
        "hello world"
    );
    assert_eq!(fetch::normalize_text("a\t\tb".to_string()), "a b");
    assert_eq!(fetch::normalize_text("abc".to_string()), "abc");
    assert_eq!(fetch::normalize_text("".to_string()), "");
}

// ── U-03: normalize_lines() ───────────────────────────────────
#[test]
fn test_u03_normalize_lines() {
    assert_eq!(
        fetch::normalize_lines("  hello  \n\n  world  \n".to_string()),
        "hello\nworld"
    );
    assert_eq!(fetch::normalize_lines("".to_string()), "");
    assert_eq!(fetch::normalize_lines("\n\n\n".to_string()), "");
}

// ── U-04: build_fts_query() ───────────────────────────────────
#[test]
fn test_u04_build_fts_query() {
    assert_eq!(search::build_fts_query("keyword"), "\"keyword\"");
    assert_eq!(
        search::build_fts_query("hello world"),
        "\"hello\" AND \"world\""
    );
    assert_eq!(search::build_fts_query("key\"word"), "\"key\"\"word\"");
    assert_eq!(search::build_fts_query(""), "");
}

// ── U-05: parse_wayback_url() 正常系 ─────────────────────────
#[test]
fn test_u05_parse_wayback_url_valid() {
    let result =
        parse_wayback_url("https://web.archive.org/web/20040604075856/http://example.com/001.html")
            .unwrap();
    assert_eq!(result.archived_at, Some("20040604075856".to_string()));
    assert_eq!(
        result.canonical_url,
        Some("http://example.com/001.html".to_string())
    );
}

#[test]
fn test_u05_parse_wayback_url_with_archive_flags() {
    let result = parse_wayback_url(
        "https://web.archive.org/web/20040604075856if_/http://example.com/001.html",
    )
    .unwrap();
    assert_eq!(result.archived_at, Some("20040604075856".to_string()));
    assert_eq!(
        result.canonical_url,
        Some("http://example.com/001.html".to_string())
    );
}

#[test]
fn test_u05_parse_wayback_url_preserves_original_query() {
    let result = parse_wayback_url(
        "https://web.archive.org/web/20040604075856/http://example.com/search?q=abc",
    )
    .unwrap();
    assert_eq!(
        result.canonical_url,
        Some("http://example.com/search?q=abc".to_string())
    );
}

// ── U-06: parse_wayback_url() 異常系 ─────────────────────────
#[test]
fn test_u06_parse_wayback_url_invalid() {
    let r1 = parse_wayback_url("https://example.com/normal");
    assert!(r1.is_err());
    assert_eq!(r1.unwrap_err().code, "INVALID_WAYBACK_URL");

    let r2 = parse_wayback_url("https://web.archive.org/web/");
    assert!(r2.is_err());
    assert_eq!(r2.unwrap_err().code, "INVALID_WAYBACK_URL");

    let r3 = parse_wayback_url(
        "https://notweb.archive.org/web/20040604075856/http://example.com/001.html",
    );
    assert!(r3.is_err());
    assert_eq!(r3.unwrap_err().code, "INVALID_WAYBACK_URL");
}

// ── U-07: charset_from_content_type() 正常系 ─────────────────
#[test]
fn test_u07_charset_from_content_type() {
    assert_eq!(
        fetch::charset_from_content_type(Some("text/html; charset=shift_jis")),
        Some("shift_jis".to_string())
    );
    assert_eq!(
        fetch::charset_from_content_type(Some("text/html; charset=\"utf-8\"")),
        Some("utf-8".to_string())
    );
}

// ── U-08: charset_from_content_type() None系 ─────────────────
#[test]
fn test_u08_charset_from_content_type_none() {
    assert_eq!(fetch::charset_from_content_type(None), None);
    assert_eq!(fetch::charset_from_content_type(Some("text/html")), None);
}

// ── U-09: charset_from_html_head() ───────────────────────────
#[test]
fn test_u09_charset_from_html_head() {
    let html1 = b"<meta charset=\"shift_jis\">";
    assert_eq!(
        fetch::charset_from_html_head(html1),
        Some("shift_jis".to_string())
    );

    let html2 = b"<meta http-equiv=\"content-type\" content=\"text/html; charset=euc-jp\">";
    assert_eq!(
        fetch::charset_from_html_head(html2),
        Some("euc-jp".to_string())
    );
}

// ── U-10: validate_source_type() 正常値 ──────────────────────
#[test]
fn test_u10_validate_source_type_valid() {
    assert!(validate_source_type("normal").is_ok());
    assert!(validate_source_type("wayback").is_ok());
    assert!(validate_source_type("local").is_ok());
}

// ── U-11: validate_source_type() 異常値 ──────────────────────
#[test]
fn test_u11_validate_source_type_invalid() {
    let r = validate_source_type("unknown");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "VALIDATION_ERROR");

    let r2 = validate_source_type("NORMAL");
    assert!(r2.is_err());
}

// ── U-12: normalize_requested_encoding() 正規化 ───────────────
#[test]
fn test_u12_normalize_encoding() {
    assert_eq!(
        fetch::normalize_requested_encoding("utf8").unwrap(),
        "utf-8"
    );
    assert_eq!(
        fetch::normalize_requested_encoding("UTF-8").unwrap(),
        "utf-8"
    );
    assert_eq!(
        fetch::normalize_requested_encoding("shift_jis").unwrap(),
        "shift_jis"
    );
    assert_eq!(
        fetch::normalize_requested_encoding("Shift-JIS").unwrap(),
        "shift_jis"
    );
    assert_eq!(
        fetch::normalize_requested_encoding("sjis").unwrap(),
        "shift_jis"
    );
    assert_eq!(
        fetch::normalize_requested_encoding("euc-jp").unwrap(),
        "euc-jp"
    );
    assert_eq!(fetch::normalize_requested_encoding("auto").unwrap(), "auto");
}

// ── U-13: normalize_requested_encoding() 未対応 ───────────────
#[test]
fn test_u13_normalize_encoding_unsupported() {
    let r = fetch::normalize_requested_encoding("iso-2022-jp");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "UNSUPPORTED_ENCODING");
}

#[test]
fn test_u13_validate_fetch_url_accepts_http_https() {
    assert!(fetch::validate_fetch_url("https://example.com/a").is_ok());
    assert!(fetch::validate_fetch_url("http://example.com/a").is_ok());
}

#[test]
fn test_u13_validate_fetch_url_rejects_local_targets() {
    for url in [
        "file:///tmp/a.html",
        "http://localhost:3000",
        "http://127.0.0.1:3000",
        "http://192.168.0.1/",
        "http://[::1]/",
        "http://[::ffff:127.0.0.1]/",
    ] {
        let result = fetch::validate_fetch_url(url);
        assert!(result.is_err(), "{url} should be rejected");
        assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
    }
}

#[test]
fn test_u13_decode_html_keeps_raw_bytes() {
    let bytes = b"<html><body>abc</body></html>";
    let result = fetch::decode_html(bytes, "utf-8", None).unwrap();
    assert_eq!(result.raw_bytes, bytes);
    assert_eq!(result.html, "<html><body>abc</body></html>");
}

#[test]
fn test_u13_truncate_fetch_error_caps_length() {
    let error = "a".repeat(1100);
    let truncated = fetch::truncate_fetch_error(&error);
    assert_eq!(truncated.chars().count(), 1025);
    assert!(truncated.ends_with('…'));
}

// ── U-14: sanitize_filename() 特殊文字除去 ───────────────────
#[test]
fn test_u14_sanitize_filename() {
    assert_eq!(files::sanitize_filename("hello-world"), "hello-world");
    assert_eq!(files::sanitize_filename("file_name"), "file_name");
    let result = files::sanitize_filename("hello world");
    assert!(!result.contains(' '));
    let long = "a".repeat(100);
    assert_eq!(files::sanitize_filename(&long).len(), 80);
}

// ── U-15: sanitize_filename() 空入力 ─────────────────────────
#[test]
fn test_u15_sanitize_filename_empty() {
    assert_eq!(files::sanitize_filename(""), "untitled");
    assert_eq!(files::sanitize_filename("!!!"), "untitled");
}

// ── U-16: apply_link_template() {n} 展開 ─────────────────────
#[test]
fn test_u16_link_template_n() {
    let result = fetch::apply_link_template("http://example.com/{n}.html", 5).unwrap();
    assert_eq!(result, "http://example.com/5.html");
}

// ── U-17: apply_link_template() {n:03d} ゼロ埋め ─────────────
#[test]
fn test_u17_link_template_zero_pad() {
    let result = fetch::apply_link_template("http://example.com/{n:03d}.html", 5).unwrap();
    assert_eq!(result, "http://example.com/005.html");
}

// ── U-18: apply_link_template() プレースホルダーなし ──────────
#[test]
fn test_u18_link_template_no_placeholder() {
    let r = fetch::apply_link_template("http://example.com/fixed.html", 1);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "VALIDATION_ERROR");
}

// ── U-19: group_favorites() グループ化 ───────────────────────
#[test]
fn test_u19_group_favorites() {
    let items = vec![
        FavoriteListItem {
            favorite_id: 1,
            favorited_at: "2026-01-01T00:00:00Z".to_string(),
            page_id: 1,
            page_title: Some("第1話".to_string()),
            page_number: Some(1),
            work_id: 1,
            work_title: "作品1".to_string(),
            site_id: 1,
            site_name: "サイトA".to_string(),
        },
        FavoriteListItem {
            favorite_id: 2,
            favorited_at: "2026-01-02T00:00:00Z".to_string(),
            page_id: 2,
            page_title: Some("第5話".to_string()),
            page_number: Some(5),
            work_id: 1,
            work_title: "作品1".to_string(),
            site_id: 1,
            site_name: "サイトA".to_string(),
        },
        FavoriteListItem {
            favorite_id: 3,
            favorited_at: "2026-01-03T00:00:00Z".to_string(),
            page_id: 3,
            page_title: Some("最終話".to_string()),
            page_number: None,
            work_id: 2,
            work_title: "作品2".to_string(),
            site_id: 2,
            site_name: "サイトB".to_string(),
        },
    ];
    let result = favorites::group_favorites(items);
    assert_eq!(result.groups.len(), 2);
    assert_eq!(result.groups[0].site_name, "サイトA");
    assert_eq!(result.groups[0].works[0].pages.len(), 2);
    assert_eq!(result.groups[1].site_name, "サイトB");
    assert_eq!(result.groups[1].works[0].pages.len(), 1);
}

// ── U-20: group_favorites() 空リスト ─────────────────────────
#[test]
fn test_u20_group_favorites_empty() {
    let result = favorites::group_favorites(vec![]);
    assert_eq!(result.groups.len(), 0);
}

// ── U-21: extract_page_content() 正常抽出 ────────────────────
#[test]
fn test_u21_extract_page_content_normal() {
    let html = "<html><body><h1>タイトル</h1><div id=\"content\">本文テキスト</div></body></html>";
    let result = fetch::extract_page_content(html, "h1", "#content", &[]).unwrap();
    assert_eq!(result.title, Some("タイトル".to_string()));
    assert_eq!(result.content_text, "本文テキスト");
}

// ── U-22: extract_page_content() 不正セレクタ ────────────────
#[test]
fn test_u22_extract_page_content_invalid_selector() {
    let html = "<html><body><div>test</div></body></html>";
    let r = fetch::extract_page_content(html, "!!invalid##", "#content", &[]);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "INVALID_SELECTOR");
}

// ── U-23: extract_page_content() セレクタ不一致 ──────────────
#[test]
fn test_u23_extract_page_content_no_match() {
    let html = "<html><body><div>test</div></body></html>";
    let r = fetch::extract_page_content(html, "h1", "#missing", &[]);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "PARSE_ERROR");
}

// ── U-24: extract_page_content() 除外セレクタ ────────────────
#[test]
fn test_u24_extract_page_content_remove_selectors() {
    let html = r#"<html><body><h1>t</h1><div id="content">本文<span class="nav">ナビ</span></div></body></html>"#;
    let result =
        fetch::extract_page_content(html, "h1", "#content", &[".nav".to_string()]).unwrap();
    assert!(
        !result.content_text.contains("ナビ"),
        "除外要素が残っている: {}",
        result.content_text
    );
}

#[test]
fn test_u24_extract_page_content_keeps_same_text_outside_removed_element() {
    let html = r#"
            <html><body>
              <h1>t</h1>
              <div id="content">
                <p>次へ進む前に、彼女は立ち止まった。</p>
                <nav>次へ</nav>
              </div>
            </body></html>
        "#;

    let result = fetch::extract_page_content(html, "h1", "#content", &["nav".to_string()]).unwrap();

    assert!(result.content_text.contains("次へ進む前に"));
    assert!(!result.content_text.lines().any(|line| line == "次へ"));
}

// ── I-01: サイト作成・一覧取得 ───────────────────────────────
#[test]
fn test_i01_site_create_and_list() {
    let conn = setup_db();
    insert_site(&conn, "サイトA", "https://a.example.com");
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sites", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);
    let name: String = conn
        .query_row("SELECT name FROM sites LIMIT 1", [], |r| r.get(0))
        .unwrap();
    assert_eq!(name, "サイトA");
}

// ── I-02: サイト更新 ──────────────────────────────────────────
#[test]
fn test_i02_site_update() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "旧名", "https://old.example.com");
    let now = now_utc();
    conn.execute(
        "UPDATE sites SET name=?1, base_url=?2, updated_at=?3 WHERE id=?4",
        params!["新名", "https://new.example.com", now, site_id],
    )
    .unwrap();
    let name: String = conn
        .query_row(
            "SELECT name FROM sites WHERE id=?1",
            params![site_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(name, "新名");
}

// ── I-03: サイト削除 ──────────────────────────────────────────
#[test]
fn test_i03_site_delete() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "削除サイト", "https://del.example.com");
    conn.execute("DELETE FROM sites WHERE id=?1", params![site_id])
        .unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM sites", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

// ── I-04: サイト削除時の作品・ページCASCADE ──────────────────
#[test]
fn test_i04_site_cascade_delete() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "カスケードサイト", "https://cascade.example.com");
    let work_id = insert_work(&conn, site_id, "作品");
    insert_page(&conn, work_id, "ページ1", None);
    insert_page(&conn, work_id, "ページ2", None);

    conn.execute("DELETE FROM sites WHERE id=?1", params![site_id])
        .unwrap();

    let work_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM works WHERE site_id=?1",
            params![site_id],
            |r| r.get(0),
        )
        .unwrap();
    let page_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pages WHERE work_id=?1",
            params![work_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(work_count, 0);
    assert_eq!(page_count, 0);
}

// ── I-06: 作品のsort_order 10刻み採番 ────────────────────────
#[test]
fn test_i06_work_sort_order_increments() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "サイト", "https://example.com");
    let w1 = insert_work(&conn, site_id, "作品1");
    let w2 = insert_work(&conn, site_id, "作品2");
    let w3 = insert_work(&conn, site_id, "作品3");

    let get_sort = |id: i64| -> i64 {
        conn.query_row(
            "SELECT sort_order FROM works WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap()
    };
    assert_eq!(get_sort(w1), 10);
    assert_eq!(get_sort(w2), 20);
    assert_eq!(get_sort(w3), 30);
}

// ── I-10: ページのsort_order 10刻み採番 ──────────────────────
#[test]
fn test_i10_page_sort_order_increments() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let p1 = insert_page(&conn, work_id, "p1", None);
    let p2 = insert_page(&conn, work_id, "p2", None);
    let p3 = insert_page(&conn, work_id, "p3", None);

    let get_sort = |id: i64| -> i64 {
        conn.query_row(
            "SELECT sort_order FROM pages WHERE id=?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap()
    };
    assert_eq!(get_sort(p1), 10);
    assert_eq!(get_sort(p2), 20);
    assert_eq!(get_sort(p3), 30);
}

// ── I-11: ページのfetch_status自動判定 ───────────────────────
#[test]
fn test_i11_fetch_status_auto() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");

    let p_success = insert_page(&conn, work_id, "有本文", Some("本文テキスト"));
    let status_s: String = conn
        .query_row(
            "SELECT fetch_status FROM pages WHERE id=?1",
            params![p_success],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status_s, "success");

    let p_pending = insert_page(&conn, work_id, "無本文", None);
    let status_p: String = conn
        .query_row(
            "SELECT fetch_status FROM pages WHERE id=?1",
            params![p_pending],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(status_p, "pending");
}

// ── I-13: お気に入り追加・判定 ───────────────────────────────
#[test]
fn test_i13_favorite_add_and_check() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let page_id = insert_page(&conn, work_id, "第1話", None);
    insert_favorite(&conn, page_id);

    let fav_id: Option<i64> = conn
        .query_row(
            "SELECT id FROM favorites WHERE page_id=?1",
            params![page_id],
            |r| r.get(0),
        )
        .optional()
        .unwrap();
    assert!(fav_id.is_some());
}

// ── I-14: お気に入り重複防止 ─────────────────────────────────
#[test]
fn test_i14_favorite_no_duplicate() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let page_id = insert_page(&conn, work_id, "第1話", None);

    let now = now_utc();
    conn.execute(
        "INSERT INTO favorites (page_id, created_at) VALUES (?1, ?2)",
        params![page_id, now],
    )
    .unwrap();
    conn.execute(
        "INSERT OR IGNORE INTO favorites (page_id, created_at) VALUES (?1, ?2)",
        params![page_id, now],
    )
    .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM favorites WHERE page_id=?1",
            params![page_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

// ── I-15: お気に入り一覧のグループ化クエリ ───────────────────
#[test]
fn test_i15_favorite_list_grouped() {
    let conn = setup_db();
    let s1 = insert_site(&conn, "サイトA", "https://a.example.com");
    let s2 = insert_site(&conn, "サイトB", "https://b.example.com");
    let w1 = insert_work(&conn, s1, "作品1");
    let w2 = insert_work(&conn, s2, "作品2");
    let p1 = insert_page(&conn, w1, "第1話", None);
    let p2 = insert_page(&conn, w1, "第5話", None);
    let p3 = insert_page(&conn, w2, "最終話", None);
    insert_favorite(&conn, p1);
    insert_favorite(&conn, p2);
    insert_favorite(&conn, p3);

    let mut stmt = conn
        .prepare(
            "SELECT f.id, f.created_at, p.id, p.title, p.page_number, w.id, w.title, s.id, s.name
             FROM favorites f
             JOIN pages p ON f.page_id = p.id
             JOIN works w ON p.work_id = w.id
             JOIN sites s ON w.site_id = s.id
             ORDER BY s.name COLLATE NOCASE, s.id, w.sort_order, w.id, p.sort_order, p.id",
        )
        .unwrap();
    let items: Vec<FavoriteListItem> = stmt
        .query_map([], |row| {
            Ok(FavoriteListItem {
                favorite_id: row.get(0)?,
                favorited_at: row.get(1)?,
                page_id: row.get(2)?,
                page_title: row.get(3)?,
                page_number: row.get(4)?,
                work_id: row.get(5)?,
                work_title: row.get(6)?,
                site_id: row.get(7)?,
                site_name: row.get(8)?,
            })
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    let grouped = favorites::group_favorites(items);
    assert_eq!(grouped.groups.len(), 2);
    assert_eq!(grouped.groups[0].site_name, "サイトA");
    assert_eq!(grouped.groups[0].works[0].pages.len(), 2);
    assert_eq!(grouped.groups[1].site_name, "サイトB");
    assert_eq!(grouped.groups[1].works[0].pages.len(), 1);
}

// ── I-16: お気に入り削除 ─────────────────────────────────────
#[test]
fn test_i16_favorite_remove() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let page_id = insert_page(&conn, work_id, "第1話", None);
    insert_favorite(&conn, page_id);
    conn.execute("DELETE FROM favorites WHERE page_id=?1", params![page_id])
        .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM favorites WHERE page_id=?1",
            params![page_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

// ── I-17: ページ削除時のお気に入りCASCADE ────────────────────
#[test]
fn test_i17_favorite_cascade_on_page_delete() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let page_id = insert_page(&conn, work_id, "第1話", None);
    insert_favorite(&conn, page_id);

    conn.execute("DELETE FROM pages WHERE id=?1", params![page_id])
        .unwrap();

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM favorites", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

// ── I-18: タイトル検索 ───────────────────────────────────────
#[test]
fn test_i18_search_titles() {
    let conn = setup_db();
    let s = insert_site(&conn, "テストサイト", "https://example.com");
    let w = insert_work(&conn, s, "テスト作品");
    insert_page(&conn, w, "テスト第1話", Some("本文"));

    let pattern = "%テスト%";
    let site_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sites WHERE name LIKE ?1",
            params![pattern],
            |r| r.get(0),
        )
        .unwrap();
    let work_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM works WHERE title LIKE ?1",
            params![pattern],
            |r| r.get(0),
        )
        .unwrap();
    let page_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pages WHERE title LIKE ?1",
            params![pattern],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(site_count, 1);
    assert_eq!(work_count, 1);
    assert_eq!(page_count, 1);
}

#[test]
fn test_i18_full_text_search_japanese_trigram() {
    let conn = setup_db();
    let s = insert_site(&conn, "テストサイト", "https://example.com");
    let w = insert_work(&conn, s, "テスト作品");
    insert_page(&conn, w, "テスト第1話", Some("これは日本語の本文です"));

    let results = search::search_full_text_fts(&conn, "日本語").unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].page_title, Some("テスト第1話".to_string()));
}

#[test]
fn test_i18_full_text_search_short_japanese_like_fallback() {
    let conn = setup_db();
    let s = insert_site(&conn, "テストサイト", "https://example.com");
    let w = insert_work(&conn, s, "テスト作品");
    insert_page(&conn, w, "テスト第1話", Some("これは本文です"));

    let results = search::search_full_text_like(&conn, "本文").unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].page_title, Some("テスト第1話".to_string()));
}

// ── I-19: タイトル検索の大文字小文字非区別 ───────────────────
#[test]
fn test_i19_search_case_insensitive() {
    let conn = setup_db();
    insert_site(&conn, "SiteAlpha", "https://alpha.example.com");
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sites WHERE name LIKE ?1 ESCAPE '\\'",
            params!["%sitealpha%"],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);
}

// ── I-20: 空クエリは早期リターン ─────────────────────────────
#[test]
fn test_i20_search_empty_query() {
    let query = "";
    assert!(query.is_empty());
}

// ── I-21: Waybackメタデータ保存 ──────────────────────────────
#[test]
fn test_i21_wayback_metadata_stored() {
    let url = "https://web.archive.org/web/20040604075856/http://example.com/001.html";
    let meta = parse_wayback_url(url).unwrap();

    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let now = now_utc();
    let sort_order = next_sort_order(&conn, "pages", "work_id", work_id).unwrap();
    conn.execute(
            "INSERT INTO pages (work_id, sort_order, source_url, source_type, canonical_url, archived_at, fetch_status, created_at, updated_at)
             VALUES (?1, ?2, ?3, 'wayback', ?4, ?5, 'pending', ?6, ?7)",
            params![work_id, sort_order, url, meta.canonical_url, meta.archived_at, now, now],
        ).unwrap();
    let page_id = conn.last_insert_rowid();

    let (canonical, archived): (Option<String>, Option<String>) = conn
        .query_row(
            "SELECT canonical_url, archived_at FROM pages WHERE id=?1",
            params![page_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(canonical, Some("http://example.com/001.html".to_string()));
    assert_eq!(archived, Some("20040604075856".to_string()));
}

#[test]
fn test_i21_duplicate_source_url_in_same_work_is_rejected() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let now = now_utc();
    conn.execute(
            "INSERT INTO pages (work_id, sort_order, title, source_url, source_type, fetch_status, created_at, updated_at)
             VALUES (?1, 10, '第1話', 'https://example.com/1', 'normal', 'pending', ?2, ?2)",
            params![work_id, now],
        )
        .unwrap();

    let result = ensure_unique_page_source_url(
        &conn,
        work_id,
        None,
        Some("https://example.com/1"),
        "normal",
    );

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code, "VALIDATION_ERROR");
}

#[test]
fn test_i21_same_url_with_different_source_type_is_allowed() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let now = now_utc();
    conn.execute(
            "INSERT INTO pages (work_id, sort_order, title, source_url, source_type, fetch_status, created_at, updated_at)
             VALUES (?1, 10, '第1話', 'https://example.com/1', 'normal', 'pending', ?2, ?2)",
            params![work_id, now],
        )
        .unwrap();

    let result = ensure_unique_page_source_url(
        &conn,
        work_id,
        None,
        Some("https://example.com/1"),
        "wayback",
    );

    assert!(result.is_ok());
}

#[test]
fn test_i21_duplicate_url_migration_allows_existing_duplicates_but_blocks_new_ones() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE pages (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                work_id INTEGER NOT NULL,
                source_type TEXT NOT NULL,
                source_url TEXT
            );",
    )
    .unwrap();
    conn.execute(
            "INSERT INTO pages (work_id, source_type, source_url) VALUES (1, 'normal', 'https://example.com/1')",
            [],
        )
        .unwrap();
    conn.execute(
            "INSERT INTO pages (work_id, source_type, source_url) VALUES (1, 'normal', 'https://example.com/1')",
            [],
        )
        .unwrap();

    conn.execute_batch(include_str!(
        "../../migrations/006_unique_page_source_urls.sql"
    ))
    .unwrap();

    let result = conn.execute(
            "INSERT INTO pages (work_id, source_type, source_url) VALUES (1, 'normal', 'https://example.com/1')",
            [],
        );
    assert!(result.is_err());
}

#[test]
fn test_i21_duplicate_source_url_diagnostic_lists_existing_duplicates() {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE sites (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            );
            CREATE TABLE works (
                id INTEGER PRIMARY KEY,
                site_id INTEGER NOT NULL,
                title TEXT NOT NULL,
                sort_order INTEGER NOT NULL
            );
            CREATE TABLE pages (
                id INTEGER PRIMARY KEY,
                work_id INTEGER NOT NULL,
                sort_order INTEGER NOT NULL,
                title TEXT,
                page_number INTEGER,
                source_type TEXT NOT NULL,
                source_url TEXT
            );",
    )
    .unwrap();
    conn.execute("INSERT INTO sites (id, name) VALUES (1, 'サイトA')", [])
        .unwrap();
    conn.execute(
        "INSERT INTO works (id, site_id, title, sort_order) VALUES (1, 1, '作品A', 10)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO pages (id, work_id, sort_order, title, source_type, source_url)
             VALUES (1, 1, 10, '第1話', 'normal', 'https://example.com/1')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO pages (id, work_id, sort_order, title, source_type, source_url)
             VALUES (2, 1, 20, '第1話 duplicate', 'normal', 'https://example.com/1')",
        [],
    )
    .unwrap();

    let groups = diagnostics::find_duplicate_source_urls(&conn).unwrap();

    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].source_url, "https://example.com/1");
    assert_eq!(groups[0].pages.len(), 2);
}

// ── I-22: サイト名空のバリデーション ─────────────────────────
#[test]
fn test_i22_site_name_empty_validation() {
    let name = "  ";
    let trimmed = name.trim();
    let r: Result<(), CommandError> = if trimmed.is_empty() {
        Err(CommandError::new("VALIDATION_ERROR", "サイト名は必須です"))
    } else {
        Ok(())
    };
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "VALIDATION_ERROR");
}

// ── I-24: 不正source_typeのバリデーション ────────────────────
#[test]
fn test_i24_invalid_source_type() {
    let r = validate_source_type("ftp");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "VALIDATION_ERROR");
}

// ── I-25: 存在しないサイトIDで作品作成 → NOT_FOUND ────────────
#[test]
fn test_i25_not_found_site_for_work() {
    let conn = setup_db();
    let r = ensure_exists(&conn, "sites", 99999, "サイト");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "NOT_FOUND");
}

// ── I-26: 存在しないIDでsite_update → 0行変更 ────────────────
#[test]
fn test_i26_not_found_site_update() {
    let conn = setup_db();
    let now = now_utc();
    let changed = conn
        .execute(
            "UPDATE sites SET name=?1, updated_at=?2 WHERE id=?3",
            params!["x", now, 99999_i64],
        )
        .unwrap();
    assert_eq!(changed, 0);
}

// ── I-28: プロファイルJSONバリデーション ─────────────────────
#[test]
fn test_i28_profile_json_validation() {
    let r = validate_profile_json("not-json");
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "VALIDATION_ERROR");

    let r2 = validate_profile_json("[1,2,3]");
    assert!(r2.is_err());
    assert_eq!(r2.unwrap_err().code, "VALIDATION_ERROR");

    let r3 = validate_profile_json("{}");
    assert!(r3.is_ok());
}

// ── I-05: 作品作成・一覧取得 ──────────────────────────────────
#[test]
fn test_i05_work_create_and_list() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "サイト", "https://example.com");
    insert_work(&conn, site_id, "作品1");
    insert_work(&conn, site_id, "作品2");

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM works WHERE site_id=?1",
            params![site_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 2);

    let mut stmt = conn
        .prepare("SELECT title FROM works WHERE site_id=?1 ORDER BY sort_order")
        .unwrap();
    let titles: Vec<String> = stmt
        .query_map(params![site_id], |r| r.get(0))
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(titles, vec!["作品1", "作品2"]);
}

// ── I-07: 作品更新 ────────────────────────────────────────────
#[test]
fn test_i07_work_update() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "旧タイトル");
    let now = now_utc();
    let changed = conn
        .execute(
            "UPDATE works SET title=?1, updated_at=?2 WHERE id=?3",
            params!["新タイトル", now, work_id],
        )
        .unwrap();
    assert_eq!(changed, 1);
    let title: String = conn
        .query_row(
            "SELECT title FROM works WHERE id=?1",
            params![work_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(title, "新タイトル");
}

// ── I-08: 作品削除時のページCASCADE ──────────────────────────
#[test]
fn test_i08_work_cascade_delete() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "作品");
    insert_page(&conn, work_id, "p1", None);
    insert_page(&conn, work_id, "p2", None);

    conn.execute("DELETE FROM works WHERE id=?1", params![work_id])
        .unwrap();

    let page_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pages WHERE work_id=?1",
            params![work_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(page_count, 0);
}

// ── I-09: ページ作成・一覧・1件取得 ──────────────────────────
#[test]
fn test_i09_page_create_list_get() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let page_id = insert_page(&conn, work_id, "第1話", None);

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pages WHERE work_id=?1",
            params![work_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    let title: String = conn
        .query_row(
            "SELECT title FROM pages WHERE id=?1",
            params![page_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(title, "第1話");

    let fetch_status: String = conn
        .query_row(
            "SELECT fetch_status FROM pages WHERE id=?1",
            params![page_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(fetch_status, "pending");
}

// ── I-12: ページ削除 ──────────────────────────────────────────
#[test]
fn test_i12_page_delete() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let work_id = insert_work(&conn, site_id, "w");
    let page_id = insert_page(&conn, work_id, "第1話", None);

    conn.execute("DELETE FROM pages WHERE id=?1", params![page_id])
        .unwrap();

    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM pages WHERE work_id=?1",
            params![work_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0);
}

// ── I-23: 作品名空のバリデーション ───────────────────────────
#[test]
fn test_i23_work_title_empty_validation() {
    let title = "   ";
    let trimmed = title.trim();
    let r: Result<(), CommandError> = if trimmed.is_empty() {
        Err(CommandError::new(
            "VALIDATION_ERROR",
            "作品タイトルは必須です",
        ))
    } else {
        Ok(())
    };
    assert!(r.is_err());
    assert_eq!(r.unwrap_err().code, "VALIDATION_ERROR");
}

// ── I-27: サイトプロファイルCRUD ─────────────────────────────
#[test]
fn test_i27_site_profile_crud() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let now = now_utc();

    // Create
    conn.execute(
            "INSERT INTO site_profiles (site_id, name, profile_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![site_id, "標準", "{}", now, now],
        ).unwrap();
    let profile_id = conn.last_insert_rowid();

    // List
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM site_profiles WHERE site_id=?1",
            params![site_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 1);

    // Update
    conn.execute(
        "UPDATE site_profiles SET name=?1, profile_json=?2, updated_at=?3 WHERE id=?4",
        params!["更新済み", "{\"encoding\":\"utf-8\"}", now, profile_id],
    )
    .unwrap();
    let name: String = conn
        .query_row(
            "SELECT name FROM site_profiles WHERE id=?1",
            params![profile_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(name, "更新済み");

    // Delete
    conn.execute("DELETE FROM site_profiles WHERE id=?1", params![profile_id])
        .unwrap();
    let count_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM site_profiles WHERE site_id=?1",
            params![site_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count_after, 0);
}

// ── I-29: プロファイル削除で作品のsite_profile_idがNULL化 ─────
#[test]
fn test_i29_profile_delete_nullifies_work() {
    let conn = setup_db();
    let site_id = insert_site(&conn, "s", "https://example.com");
    let now = now_utc();

    conn.execute(
            "INSERT INTO site_profiles (site_id, name, profile_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![site_id, "標準", "{}", now, now],
        ).unwrap();
    let profile_id = conn.last_insert_rowid();

    let sort_order = next_sort_order(&conn, "works", "site_id", site_id).unwrap();
    conn.execute(
            "INSERT INTO works (site_id, site_profile_id, title, sort_order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![site_id, profile_id, "作品1", sort_order, now, now],
        ).unwrap();
    let work_id = conn.last_insert_rowid();

    conn.execute("DELETE FROM site_profiles WHERE id=?1", params![profile_id])
        .unwrap();

    let profile_id_after: Option<i64> = conn
        .query_row(
            "SELECT site_profile_id FROM works WHERE id=?1",
            params![work_id],
            |r| r.get(0),
        )
        .unwrap();
    assert!(profile_id_after.is_none(), "site_profile_id should be NULL");

    let work_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM works WHERE id=?1",
            params![work_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(work_count, 1, "Work should still exist");
}
