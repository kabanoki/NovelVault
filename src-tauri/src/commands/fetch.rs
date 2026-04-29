use super::{
    ensure_exists, ensure_unique_page_source_url, next_sort_order, now_utc, page_get_by_id,
    site_profile_get_by_id, validate_source_type, wayback_metadata, work_get_by_id,
};
use crate::{
    db::DbState,
    error::{CommandError, CommandResult},
    types::{BulkFetchByProfileArgs, BulkFetchResult, FetchPageByUrlArgs, Page},
};
use encoding_rs::Encoding;
use rusqlite::{params, OptionalExtension};
use scraper::{Html, Selector};
use serde::Deserialize;
use std::net::IpAddr;
use tauri::{AppHandle, Manager, State};

const FETCH_ERROR_MAX_CHARS: usize = 1024;

#[tauri::command]
pub async fn fetch_page_by_url(
    app: AppHandle,
    db: State<'_, DbState>,
    args: FetchPageByUrlArgs,
) -> CommandResult<Page> {
    validate_source_type(&args.source_type)?;
    let url = args.url.trim().to_string();
    if url.is_empty() {
        return Err(CommandError::new("VALIDATION_ERROR", "URLは必須です"));
    }
    validate_fetch_url(&url)?;

    let wayback = wayback_metadata(Some(&url), &args.source_type)?;
    let requested_encoding = normalize_requested_encoding(&args.encoding)?;

    {
        let conn = db.connection()?;
        ensure_exists(&conn, "pages", args.page_id, "ページ")?;
    }

    let fetched = match fetch_html(&url, &requested_encoding).await {
        Ok(fetched) => fetched,
        Err(error) => {
            let conn = db.connection()?;
            update_fetch_failure(&conn, args.page_id, "fetch_failed", &error.message)?;
            return Err(error);
        }
    };

    let page = {
        let conn = db.connection()?;
        page_get_by_id(&conn, args.page_id)?
    };
    {
        let conn = db.connection()?;
        ensure_unique_page_source_url(
            &conn,
            page.work_id,
            Some(args.page_id),
            Some(&url),
            &args.source_type,
        )?;
    }

    let relative_html_path = match save_original_html(&app, &db, &page, &fetched.raw_bytes) {
        Ok(relative_path) => relative_path,
        Err(error) => {
            let conn = db.connection()?;
            update_fetch_failure(&conn, args.page_id, "save_failed", &error.message)?;
            return Err(error);
        }
    };

    let extracted = match extract_page_content(
        &fetched.html,
        &args.title_selector,
        &args.content_selector,
        &args.remove_selectors,
    ) {
        Ok(extracted) => extracted,
        Err(error) => {
            let conn = db.connection()?;
            update_fetch_failure_with_html(
                &conn,
                args.page_id,
                FetchFailureUpdate {
                    status: "parse_failed",
                    error: &error.message,
                    source_url: &url,
                    source_type: &args.source_type,
                    canonical_url: wayback.canonical_url.as_deref(),
                    archived_at: wayback.archived_at.as_deref(),
                    requested_encoding: &requested_encoding,
                    detected_encoding: &fetched.detected_encoding,
                    content_html_path: &relative_html_path,
                },
            )?;
            return Err(error);
        }
    };

    let conn = db.connection()?;
    let now = now_utc();
    conn.execute(
        "UPDATE pages
         SET title=COALESCE(?1, title),
             source_url=?2,
             source_type=?3,
             canonical_url=?4,
             archived_at=?5,
             requested_encoding=?6,
             detected_encoding=?7,
             content_text=?8,
             content_html_path=?9,
             fetch_status='success',
             fetch_error=NULL,
             fetched_at=?10,
             updated_at=?11
         WHERE id=?12",
        params![
            extracted.title,
            url,
            args.source_type,
            wayback.canonical_url,
            wayback.archived_at,
            requested_encoding,
            fetched.detected_encoding,
            extracted.content_text,
            relative_html_path,
            now,
            now,
            args.page_id,
        ],
    )?;
    page_get_by_id(&conn, args.page_id)
}

#[tauri::command]
pub async fn bulk_fetch_by_profile(
    app: AppHandle,
    db: State<'_, DbState>,
    args: BulkFetchByProfileArgs,
) -> CommandResult<BulkFetchResult> {
    let (work, profile) = {
        let conn = db.connection()?;
        (
            work_get_by_id(&conn, args.work_id)?,
            site_profile_get_by_id(&conn, args.site_profile_id)?,
        )
    };
    if work.site_id != profile.site_id {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "作品とプロファイルのサイトが一致しません",
        ));
    }

    let profile_config: SiteProfileConfig = serde_json::from_str(&profile.profile_json)
        .map_err(|e| CommandError::new("VALIDATION_ERROR", format!("JSONが不正です: {e}")))?;
    let index = profile_config
        .index_pattern
        .ok_or_else(|| CommandError::new("VALIDATION_ERROR", "index_pattern が必要です"))?;
    let source_type = profile_config
        .source_type
        .unwrap_or_else(|| "normal".to_string());
    validate_source_type(&source_type)?;
    let encoding = normalize_requested_encoding(
        &profile_config
            .encoding
            .unwrap_or_else(|| "auto".to_string()),
    )?;
    let page_pattern = profile_config.page_pattern;

    let links = if let Some(template) = index
        .link_url_template
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        generate_template_links(template, index.link_url_range.as_ref())?
    } else {
        let index_url = index
            .url
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| CommandError::new("VALIDATION_ERROR", "index_pattern.url が必要です"))?;
        let link_selector = index
            .link_selector
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| {
                CommandError::new("VALIDATION_ERROR", "index_pattern.link_selector が必要です")
            })?;
        let index_html = fetch_html(&index_url, &encoding).await?.html;
        extract_index_links(
            &index_html,
            &index_url,
            profile_config.base_url.as_deref(),
            &link_selector,
            index.link_url_pattern.as_deref(),
        )?
    };

    let mut created_count = 0;
    let mut success_count = 0;
    let mut failed_count = 0;

    for link in links {
        if validate_fetch_url(&link.url).is_err() {
            failed_count += 1;
            continue;
        }

        let wayback = wayback_metadata(Some(&link.url), &source_type)?;
        let page_id = {
            let conn = db.connection()?;
            let now = now_utc();
            let sort_order = next_sort_order(&conn, "pages", "work_id", args.work_id)?;
            conn.execute(
                "INSERT INTO pages (
                    work_id, sort_order, title, source_url, source_type, canonical_url, archived_at,
                    requested_encoding, fetch_status, created_at, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'pending', ?9, ?10)",
                params![
                    args.work_id,
                    sort_order,
                    link.title,
                    link.url,
                    source_type,
                    wayback.canonical_url,
                    wayback.archived_at,
                    encoding,
                    now,
                    now,
                ],
            )?;
            conn.last_insert_rowid()
        };
        created_count += 1;

        let fetch_args = FetchPageByUrlArgs {
            page_id,
            url: link.url,
            source_type: source_type.clone(),
            title_selector: page_pattern.title_selector.clone(),
            content_selector: page_pattern.content_selector.clone(),
            remove_selectors: page_pattern.remove_selectors.clone().unwrap_or_default(),
            encoding: encoding.clone(),
        };

        match fetch_page_by_url(app.clone(), db.clone(), fetch_args).await {
            Ok(_) => success_count += 1,
            Err(_) => failed_count += 1,
        }
    }

    Ok(BulkFetchResult {
        created_count,
        success_count,
        failed_count,
    })
}

pub(super) struct FetchedHtml {
    pub(super) raw_bytes: Vec<u8>,
    pub(super) html: String,
    pub(super) detected_encoding: String,
}

async fn fetch_html(url: &str, requested_encoding: &str) -> CommandResult<FetchedHtml> {
    validate_fetch_url(url)?;

    let response = reqwest::Client::builder()
        .user_agent("NovelVault/0.1")
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if validate_fetch_url(attempt.url().as_str()).is_ok() {
                attempt.follow()
            } else {
                attempt.error("redirect target is not allowed")
            }
        }))
        .build()
        .map_err(|e| CommandError::new("FETCH_ERROR", e.to_string()))?
        .get(url)
        .send()
        .await
        .map_err(|e| CommandError::new("FETCH_ERROR", e.to_string()))?;

    let status = response.status();
    if !status.is_success() {
        return Err(CommandError::new(
            "FETCH_ERROR",
            format!("HTTP取得に失敗しました: {status}"),
        ));
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let bytes = response
        .bytes()
        .await
        .map_err(|e| CommandError::new("FETCH_ERROR", e.to_string()))?;
    decode_html(bytes.as_ref(), requested_encoding, content_type.as_deref())
}

pub(super) fn decode_html(
    bytes: &[u8],
    requested_encoding: &str,
    content_type: Option<&str>,
) -> CommandResult<FetchedHtml> {
    let encoding = if requested_encoding == "auto" {
        detect_encoding(bytes, content_type)
    } else {
        Encoding::for_label(requested_encoding.as_bytes()).ok_or_else(|| {
            CommandError::new(
                "UNSUPPORTED_ENCODING",
                format!("未対応の文字コードです: {requested_encoding}"),
            )
        })?
    };

    let (decoded, _, _) = encoding.decode(bytes);
    Ok(FetchedHtml {
        raw_bytes: bytes.to_vec(),
        html: decoded.into_owned(),
        detected_encoding: encoding.name().to_lowercase(),
    })
}

pub(super) fn validate_fetch_url(url: &str) -> CommandResult<()> {
    let parsed = reqwest::Url::parse(url)
        .map_err(|e| CommandError::new("VALIDATION_ERROR", format!("URLが不正です: {e}")))?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => {
            return Err(CommandError::new(
                "VALIDATION_ERROR",
                "取得URLは http または https のみ対応しています",
            ));
        }
    }

    let host = parsed
        .host_str()
        .ok_or_else(|| CommandError::new("VALIDATION_ERROR", "取得URLにホストがありません"))?;
    let lower_host = host.to_ascii_lowercase();
    if lower_host == "localhost" || lower_host.ends_with(".localhost") {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "localhost への取得は許可されていません",
        ));
    }
    let ip_host = host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host);
    if let Ok(ip) = ip_host.parse::<IpAddr>() {
        if is_disallowed_fetch_ip(ip) {
            return Err(CommandError::new(
                "VALIDATION_ERROR",
                "ローカルまたはプライベートIPへの取得は許可されていません",
            ));
        }
    }

    Ok(())
}

fn is_disallowed_fetch_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || ip.is_broadcast()
                || ip.is_unspecified()
                || ip.is_multicast()
        }
        IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return is_disallowed_fetch_ip(IpAddr::V4(mapped));
            }
            ip.is_loopback()
                || ip.is_unspecified()
                || ip.is_unique_local()
                || ip.is_unicast_link_local()
                || ip.is_multicast()
        }
    }
}

pub(super) fn detect_encoding(bytes: &[u8], content_type: Option<&str>) -> &'static Encoding {
    if let Some(charset) = charset_from_content_type(content_type) {
        if let Some(encoding) = Encoding::for_label(charset.as_bytes()) {
            return encoding;
        }
    }

    if let Some(charset) = charset_from_html_head(bytes) {
        if let Some(encoding) = Encoding::for_label(charset.as_bytes()) {
            return encoding;
        }
    }

    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    detector.guess(None, true)
}

pub(super) fn charset_from_content_type(content_type: Option<&str>) -> Option<String> {
    let content_type = content_type?;
    content_type.split(';').find_map(|part| {
        let trimmed = part.trim();
        trimmed
            .strip_prefix("charset=")
            .or_else(|| trimmed.strip_prefix("Charset="))
            .map(|value| value.trim_matches('"').to_string())
    })
}

pub(super) fn charset_from_html_head(bytes: &[u8]) -> Option<String> {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(2048)]).to_lowercase();
    let marker = "charset=";
    let start = head.find(marker)? + marker.len();
    let rest = &head[start..];
    let charset = rest
        .trim_start_matches(['"', '\'', ' '])
        .split(|c: char| c == '"' || c == '\'' || c == '>' || c.is_whitespace() || c == ';')
        .next()?;
    if charset.is_empty() {
        None
    } else {
        Some(charset.to_string())
    }
}

pub(super) fn normalize_requested_encoding(encoding: &str) -> CommandResult<String> {
    let normalized = encoding.trim().to_lowercase().replace('_', "-");
    match normalized.as_str() {
        "auto" => Ok("auto".to_string()),
        "utf8" | "utf-8" => Ok("utf-8".to_string()),
        "shift-jis" | "shift_jis" | "sjis" | "windows-31j" => Ok("shift_jis".to_string()),
        "euc-jp" | "eucjp" => Ok("euc-jp".to_string()),
        _ => Err(CommandError::new(
            "UNSUPPORTED_ENCODING",
            "文字コードは auto / utf-8 / shift_jis / euc-jp のいずれかです",
        )),
    }
}

#[derive(Debug)]
pub(super) struct ExtractedPage {
    pub(super) title: Option<String>,
    pub(super) content_text: String,
}

pub(super) fn extract_page_content(
    html: &str,
    title_selector: &str,
    content_selector: &str,
    remove_selectors: &[String],
) -> CommandResult<ExtractedPage> {
    let document = Html::parse_document(html);
    let title_sel = parse_selector(title_selector, "タイトル用セレクタ")?;
    let content_sel = parse_selector(content_selector, "本文用セレクタ")?;
    let remove_sels = remove_selectors
        .iter()
        .filter(|value| !value.trim().is_empty())
        .map(|value| parse_selector(value, "除外セレクタ"))
        .collect::<CommandResult<Vec<_>>>()?;

    let title = document
        .select(&title_sel)
        .next()
        .map(|element| normalize_text(element.text().collect::<Vec<_>>().join(" ")))
        .filter(|value| !value.is_empty());

    let content_element = document.select(&content_sel).next().ok_or_else(|| {
        CommandError::new(
            "PARSE_ERROR",
            "本文用セレクタに一致する要素が見つかりません",
        )
    })?;

    let content_text = extract_text_excluding(&content_element, &remove_sels);

    if content_text.is_empty() {
        return Err(CommandError::new(
            "PARSE_ERROR",
            "本文テキストを抽出できませんでした",
        ));
    }

    Ok(ExtractedPage {
        title,
        content_text,
    })
}

fn extract_text_excluding(
    content_element: &scraper::ElementRef<'_>,
    remove_sels: &[Selector],
) -> String {
    let ignored_node_ids = remove_sels
        .iter()
        .flat_map(|selector| content_element.select(selector).map(|element| element.id()))
        .collect::<Vec<_>>();

    let text = content_element
        .descendants()
        .filter_map(|node| {
            let text = node.value().as_text()?;
            let is_ignored = node
                .ancestors()
                .any(|ancestor| ignored_node_ids.contains(&ancestor.id()));
            (!is_ignored).then_some(text.to_string())
        })
        .collect::<Vec<_>>()
        .join("\n");

    normalize_lines(text)
}

fn parse_selector(selector: &str, label: &str) -> CommandResult<Selector> {
    Selector::parse(selector.trim()).map_err(|_| {
        CommandError::new(
            "INVALID_SELECTOR",
            format!("{label}がCSSセレクタとして不正です"),
        )
    })
}

pub(super) fn normalize_text(value: String) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(super) fn normalize_lines(value: String) -> String {
    value
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn update_fetch_failure(
    conn: &rusqlite::Connection,
    page_id: i64,
    status: &str,
    error: &str,
) -> CommandResult<()> {
    let now = now_utc();
    let error = truncate_fetch_error(error);
    conn.execute(
        "UPDATE pages
         SET fetch_status=?1, fetch_error=?2, updated_at=?3
         WHERE id=?4",
        params![status, error, now, page_id],
    )?;
    Ok(())
}

struct FetchFailureUpdate<'a> {
    status: &'a str,
    error: &'a str,
    source_url: &'a str,
    source_type: &'a str,
    canonical_url: Option<&'a str>,
    archived_at: Option<&'a str>,
    requested_encoding: &'a str,
    detected_encoding: &'a str,
    content_html_path: &'a str,
}

fn update_fetch_failure_with_html(
    conn: &rusqlite::Connection,
    page_id: i64,
    update: FetchFailureUpdate<'_>,
) -> CommandResult<()> {
    let now = now_utc();
    let error = truncate_fetch_error(update.error);
    conn.execute(
        "UPDATE pages
         SET source_url=?1,
             source_type=?2,
             canonical_url=?3,
             archived_at=?4,
             requested_encoding=?5,
             detected_encoding=?6,
             content_html_path=?7,
             fetch_status=?8,
             fetch_error=?9,
             fetched_at=?10,
             updated_at=?11
         WHERE id=?12",
        params![
            update.source_url,
            update.source_type,
            update.canonical_url,
            update.archived_at,
            update.requested_encoding,
            update.detected_encoding,
            update.content_html_path,
            update.status,
            error,
            now,
            now,
            page_id,
        ],
    )?;
    Ok(())
}

fn save_original_html(
    app: &AppHandle,
    db: &State<'_, DbState>,
    page: &Page,
    html: &[u8],
) -> CommandResult<String> {
    let (site_id, work_id) = {
        let conn = db.connection()?;
        page_storage_ids(&conn, page.id)?
    };
    let relative_path = format!(
        "originals/sites/{site_id}/works/{work_id}/pages/{}.html",
        page.id
    );
    let app_data_dir = app.path().app_data_dir()?;
    let full_path = app_data_dir.join(&relative_path);
    if let Some(parent) = full_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(full_path, html)?;
    Ok(relative_path)
}

pub(super) fn truncate_fetch_error(error: &str) -> String {
    let mut truncated = error
        .chars()
        .take(FETCH_ERROR_MAX_CHARS)
        .collect::<String>();
    if error.chars().count() > FETCH_ERROR_MAX_CHARS {
        truncated.push('…');
    }
    truncated
}

#[derive(Debug, Deserialize)]
struct SiteProfileConfig {
    base_url: Option<String>,
    source_type: Option<String>,
    encoding: Option<String>,
    index_pattern: Option<IndexPatternConfig>,
    page_pattern: PagePatternConfig,
}

#[derive(Debug, Deserialize)]
struct IndexPatternConfig {
    url: Option<String>,
    link_selector: Option<String>,
    link_url_pattern: Option<String>,
    link_url_template: Option<String>,
    link_url_range: Option<LinkUrlRange>,
}

#[derive(Debug, Deserialize)]
struct LinkUrlRange {
    start: i64,
    end: i64,
}

#[derive(Debug, Deserialize)]
struct PagePatternConfig {
    title_selector: String,
    content_selector: String,
    remove_selectors: Option<Vec<String>>,
}

struct IndexLink {
    url: String,
    title: Option<String>,
}

fn generate_template_links(
    template: &str,
    range: Option<&LinkUrlRange>,
) -> CommandResult<Vec<IndexLink>> {
    let range = range.ok_or_else(|| {
        CommandError::new(
            "VALIDATION_ERROR",
            "link_url_template には link_url_range が必要です",
        )
    })?;
    if range.end < range.start {
        return Err(CommandError::new(
            "VALIDATION_ERROR",
            "link_url_range.end は start 以上である必要があります",
        ));
    }
    let mut links = Vec::new();
    for n in range.start..=range.end {
        let url = apply_link_template(template, n)?;
        links.push(IndexLink {
            title: Some(n.to_string()),
            url,
        });
    }
    Ok(links)
}

pub(super) fn apply_link_template(template: &str, n: i64) -> CommandResult<String> {
    if template.contains("{n}") {
        return Ok(template.replace("{n}", &n.to_string()));
    }
    let re = regex::Regex::new(r"\{n:(\d+)d\}")
        .map_err(|e| CommandError::new("REGEX_ERROR", e.to_string()))?;
    if let Some(captures) = re.captures(template) {
        let width = captures
            .get(1)
            .and_then(|value| value.as_str().parse::<usize>().ok())
            .ok_or_else(|| CommandError::new("VALIDATION_ERROR", "link_url_template が不正です"))?;
        let formatted = format!("{n:0width$}");
        return Ok(re.replace(template, formatted.as_str()).to_string());
    }
    Err(CommandError::new(
        "VALIDATION_ERROR",
        "link_url_template は {n} または {n:03d} を含む必要があります",
    ))
}

fn extract_index_links(
    html: &str,
    index_url: &str,
    base_url: Option<&str>,
    link_selector: &str,
    link_url_pattern: Option<&str>,
) -> CommandResult<Vec<IndexLink>> {
    let document = Html::parse_document(html);
    let selector = parse_selector(link_selector, "リンク用セレクタ")?;
    let base = reqwest::Url::parse(base_url.unwrap_or(index_url))
        .map_err(|e| CommandError::new("VALIDATION_ERROR", format!("base_urlが不正です: {e}")))?;
    let pattern = link_url_pattern
        .filter(|value| !value.trim().is_empty())
        .map(regex::Regex::new)
        .transpose()
        .map_err(|e| {
            CommandError::new(
                "VALIDATION_ERROR",
                format!("link_url_pattern が不正です: {e}"),
            )
        })?;

    let mut links = Vec::new();
    for element in document.select(&selector) {
        let Some(href) = element.value().attr("href") else {
            continue;
        };
        if let Some(pattern) = &pattern {
            if !pattern.is_match(href) {
                continue;
            }
        }
        let url = base.join(href).map_err(|e| {
            CommandError::new("VALIDATION_ERROR", format!("リンクURLが不正です: {e}"))
        })?;
        let title = normalize_text(element.text().collect::<Vec<_>>().join(" "));
        links.push(IndexLink {
            url: url.to_string(),
            title: if title.is_empty() { None } else { Some(title) },
        });
    }

    if links.is_empty() {
        return Err(CommandError::new(
            "PARSE_ERROR",
            "目次ページからリンクを抽出できませんでした",
        ));
    }
    Ok(links)
}

fn page_storage_ids(conn: &rusqlite::Connection, page_id: i64) -> CommandResult<(i64, i64)> {
    conn.query_row(
        "SELECT w.site_id, p.work_id
         FROM pages p
         JOIN works w ON p.work_id = w.id
         WHERE p.id=?1",
        params![page_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )
    .optional()?
    .ok_or_else(|| CommandError::new("NOT_FOUND", "ページが見つかりません"))
}
