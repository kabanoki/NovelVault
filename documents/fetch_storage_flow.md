# Rust側 取得・保存処理フロー設計書

対応要件定義: v4.3 / スキーマ: 001_initial_schema.sql + 002_add_favorites.sql

---

## 1. 概要

`fetchPageByUrl` コマンドの内部処理フローを定義する。
HTTPリクエストからDB保存、HTMLファイル保存までを担当する中核処理。

---

## 2. モジュール構成

```
src-tauri/src/
├── main.rs                    # エントリポイント
├── lib.rs                     # Tauri コマンド登録
├── commands.rs                # コマンド共通ヘルパー・テスト親モジュール
├── commands/
│   ├── diagnostics.rs         # DB診断
│   ├── favorites.rs           # お気に入り
│   ├── fetch.rs               # URL取得・一括取得・HTML抽出
│   ├── files.rs               # エクスポート・バックアップ
│   ├── pages.rs               # ページCRUD
│   ├── search.rs              # タイトル/全文検索
│   ├── site_profiles.rs       # サイトプロファイルCRUD
│   ├── sites.rs               # サイトCRUD
│   ├── tests.rs               # commands 系ユニット/統合テスト
│   └── works.rs               # 作品CRUD
├── types.rs                   # 共通型
├── db/
│   ├── mod.rs                 # DBパス保持・コマンドごとの接続生成
│   ├── migrations.rs          # マイグレーションランナー
│   └── connection.rs          # 接続初期化（PRAGMA設定含む）
└── error.rs                   # エラー型定義
```

---

## 3. 全体処理フロー

```
fetchPageByUrl(args)
    │
    │  事前条件: pages レコードは pageCreate で作成済み
    │
    ▼
┌─────────────────────────────────────┐
│ 1. URL正規化・種別判定                  │
│    url_handler::normalize_url()      │
└──────────────┬──────────────────────┘
               │ URL ok
               ▼
┌─────────────────────────────────────┐
│ 2. HTTP取得                           │
│    http::fetch(url)                  │
│    → bytes + Content-Type            │
└──────────────┬──────────────────────┘
               │ 成功
               │ 失敗 → fetch_status=fetch_failed → 終了
               ▼
┌─────────────────────────────────────┐
│ 3. 文字コード判定・UTF-8変換             │
│    encoding::decode(bytes, hint)     │
│    → utf8_html + detected_encoding   │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│ 4. 元HTML保存                          │
│    originals/.../{page_id}.html      │
└──────────────┬──────────────────────┘
               │ 成功
               │ 失敗 → fetch_status=save_failed → 終了
               ▼
┌─────────────────────────────────────┐
│ 5. HTML解析・本文抽出                   │
│    extract_page_content(html, args)  │
│    → title + content_text            │
└──────────────┬──────────────────────┘
               │ 成功
               │ 失敗 → fetch_status=parse_failed │
               │        content_html_pathは保存済み │
               ▼
┌─────────────────────────────────────┐
│ 6. DB更新（success として）             │
│    content_text + content_html_path  │
└─────────────────────────────────────┘
```

---

## 4. 各ステップの詳細

### 4.1 URL正規化・種別判定

```rust
// commands/fetch.rs

pub struct NormalizedUrl {
    pub url: String,
    pub source_type: SourceType,
    pub original_url: Option<String>,    // Wayback時の元URL
    pub archived_at: Option<String>,     // Waybackタイムスタンプ
}

pub fn normalize_url(url: &str, hint: SourceType) -> Result<NormalizedUrl, Error> {
    validate_fetch_url(url)?;

    match hint {
        SourceType::Wayback => parse_wayback_url(url),
        SourceType::Normal  => Ok(NormalizedUrl {
            url: url.to_string(),
            source_type: SourceType::Normal,
            original_url: None,
            archived_at: None,
        }),
        SourceType::Local   => Err(Error::NotImplemented),
    }
}

/// Wayback URLを分解する
/// 例: https://web.archive.org/web/20040604075856/http://example.com/001.html
///   → タイムスタンプ: 20040604075856
///   → 元URL:        http://example.com/001.html
fn parse_wayback_url(url: &str) -> Result<NormalizedUrl, Error> {
    let parsed = Url::parse(url).map_err(|_| Error::InvalidWaybackUrl)?;
    if !matches!(parsed.scheme(), "http" | "https")
        || parsed.host_str() != Some("web.archive.org")
    {
        return Err(Error::InvalidWaybackUrl);
    }

    // /web/{timestamp}{flags}/{original_url} を構造化して抽出する。
    // 例: 20040604075856if_ のような Wayback 表示フラグも許容する。
    let rest = parsed.path().strip_prefix("/web/").ok_or(Error::InvalidWaybackUrl)?;
    let (timestamp_with_flags, original_url) =
        rest.split_once('/').ok_or(Error::InvalidWaybackUrl)?;
    if timestamp_with_flags.len() < 14 {
        return Err(Error::InvalidWaybackUrl);
    }
    let archived_at = &timestamp_with_flags[..14];
    let original_url = match parsed.query() {
        Some(query) => format!("{original_url}?{query}"),
        None => original_url.to_string(),
    };

    Ok(NormalizedUrl {
        url: url.to_string(),
        source_type: SourceType::Wayback,
        original_url: Some(original_url),
        archived_at:  Some(archived_at.to_string()),
    })
}
```

`validate_fetch_url` は `http` / `https` のみを許可し、`localhost`、ループバック、プライベートIP、リンクローカルなどローカルネットワーク向けの取得を拒否する。

### 4.2 HTTP取得

```rust
// commands/fetch.rs

pub struct FetchResult {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
    pub charset_hint: Option<String>,    // Content-Type charset=...
    pub final_url: String,                // リダイレクト後のURL
}

pub async fn fetch(url: &str, options: &FetchOptions) -> Result<FetchResult, Error> {
    let client = reqwest::Client::builder()
        .user_agent(&options.user_agent)
        .timeout(std::time::Duration::from_secs(30))
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            if validate_fetch_url(attempt.url().as_str()).is_ok() {
                attempt.follow()
            } else {
                attempt.error("redirect target is not allowed")
            }
        }))
        .build()?;

    // リクエスト間隔を確保（前回リクエストからのウェイト）
    options.rate_limiter.wait().await;

    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(Error::HttpError {
            status: response.status().as_u16(),
            url: url.to_string(),
        });
    }

    let final_url = response.url().to_string();
    let content_type = response.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(String::from);
    let charset_hint = parse_charset_from_content_type(&content_type);
    let bytes = response.bytes().await?.to_vec();

    Ok(FetchResult { bytes, content_type, charset_hint, final_url })
}
```

**リクエストレート制限**

複数ページの一括取得時に同一サイトへ短時間で連続リクエストしないよう、サイト単位のレート制限を設ける。

```rust
pub struct RateLimiter {
    last_request: Mutex<Option<Instant>>,
    interval_ms: u64,
}

impl RateLimiter {
    pub async fn wait(&self) {
        let mut last = self.last_request.lock().await;
        if let Some(prev) = *last {
            let elapsed = prev.elapsed();
            let interval = Duration::from_millis(self.interval_ms);
            if elapsed < interval {
                tokio::time::sleep(interval - elapsed).await;
            }
        }
        *last = Some(Instant::now());
    }
}
```

### 4.3 文字コード判定・UTF-8変換

```rust
// commands/fetch.rs

pub struct DecodeResult {
    pub raw_bytes: Vec<u8>,   // 保存用。デコード前のレスポンス本文をそのまま保持
    pub html:      String,
    pub detected:  String,    // "utf-8" / "shift_jis" / "euc-jp"
}

pub fn decode(
    bytes: &[u8],
    requested: Encoding,
    charset_hint: Option<&str>,
) -> Result<DecodeResult, Error> {
    let encoding_name = match requested {
        Encoding::Auto => detect_encoding(bytes, charset_hint),
        Encoding::Utf8     => "utf-8",
        Encoding::ShiftJis => "shift_jis",
        Encoding::EucJp    => "euc-jp",
    };

    let encoding = encoding_rs::Encoding::for_label(encoding_name.as_bytes())
        .ok_or_else(|| Error::UnknownEncoding(encoding_name.to_string()))?;

    let (cow, _used, had_errors) = encoding.decode(bytes);

    if had_errors {
        // 警告ログだけ出して続行（古いサイトはBOM不正等が多いため）
        log::warn!("Decoding had errors for encoding {}", encoding.name());
    }

    Ok(DecodeResult {
        raw_bytes: bytes.to_vec(),
        html:      cow.into_owned(),
        detected:  encoding.name().to_lowercase(),
    })
}

/// 検出ロジック（優先順位）:
///   1. Content-Type の charset パラメータ
///   2. HTML <meta charset="..."> または <meta http-equiv="content-type">
///   3. chardetng などのバイト分析ライブラリ
fn detect_encoding(bytes: &[u8], charset_hint: Option<&str>) -> &'static str {
    // 1. Content-Type のヒント
    if let Some(hint) = charset_hint {
        if !hint.is_empty() {
            return normalize_charset_name(hint);
        }
    }

    // 2. HTMLのmetaタグから検出
    //    先頭1024バイトをasciiとして読み、metaタグを探す
    let head = &bytes[..bytes.len().min(1024)];
    if let Some(charset) = extract_meta_charset(head) {
        return normalize_charset_name(&charset);
    }

    // 3. chardetng で推定
    use chardetng::EncodingDetector;
    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    detector.guess(None, true).name()
}
```

### 4.4 HTML解析・本文抽出

```rust
// commands/fetch.rs

pub struct ExtractResult {
    pub title:        Option<String>,
    pub content_text: String,
}

pub struct ExtractOptions<'a> {
    pub title_selector:   &'a str,
    pub content_selector: &'a str,
    pub remove_selectors: &'a [String],
}

pub fn extract(html: &str, opts: &ExtractOptions) -> Result<ExtractResult, Error> {
    let document = scraper::Html::parse_document(html);

    // タイトル抽出
    let title_sel = scraper::Selector::parse(opts.title_selector)
        .map_err(|_| Error::InvalidSelector(opts.title_selector.to_string()))?;
    let title = document.select(&title_sel)
        .next()
        .map(|el| normalize_whitespace(&el.text().collect::<String>()));

    // 本文抽出
    let content_sel = scraper::Selector::parse(opts.content_selector)
        .map_err(|_| Error::InvalidSelector(opts.content_selector.to_string()))?;
    let content_node = document.select(&content_sel)
        .next()
        .ok_or(Error::ContentNotFound)?;

    // 不要要素を除外してテキスト抽出
    // scraper は tree mutation を直接サポートしないため、
    // 除外対象のノードIDを集めて再帰的にスキップする
    let exclude_ids = collect_excluded_node_ids(&document, opts.remove_selectors);
    let content_text = extract_text_excluding(&content_node, &exclude_ids);

    if content_text.trim().is_empty() {
        return Err(Error::EmptyContent);
    }

    Ok(ExtractResult {
        title,
        content_text: normalize_whitespace(&content_text),
    })
}

/// 連続する空白・改行を整形
fn normalize_whitespace(s: &str) -> String {
    s.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}
```

**抽出失敗の判定基準**

| 状況 | 結果 |
|------|------|
| `title_selector` が無効なCSSセレクタ | parse_failed |
| `content_selector` が無効なCSSセレクタ | parse_failed |
| `content_selector` がマッチしない | parse_failed |
| 抽出した本文が空 | parse_failed |
| タイトルがマッチしない | 警告のみ（本文があれば success） |

### 4.5 DB更新

現行実装では、取得できた元HTMLを先に保存してから本文抽出を行う。
これにより、CSSセレクタ不一致などで `parse_failed` になった場合でも、
ユーザーが保存済みHTMLを見て原因を確認できる。

| 状況 | DB状態 |
|------|--------|
| HTTP取得失敗 | `fetch_status='fetch_failed'`, `content_html_path` は更新しない |
| HTML保存失敗 | `fetch_status='save_failed'`, 新しい `content_text` は書き込まない |
| 本文抽出失敗 | `fetch_status='parse_failed'`, `content_html_path` と取得メタデータは保存する |
| 取得・保存・抽出すべて成功 | `fetch_status='success'`, `content_text` と `content_html_path` を保存する |

`fetch_error` はDB肥大化を避けるため、保存前に1024文字へ切り詰める。

```rust
// commands/fetch.rs（概略）

let fetched = fetch_html(&url, &requested_encoding).await?;
let relative_html_path = save_original_html(&app, &db, &page, &fetched.raw_bytes)?;

let extracted = match extract_page_content(
    &fetched.html,
    &args.title_selector,
    &args.content_selector,
    &args.remove_selectors,
) {
    Ok(extracted) => extracted,
    Err(error) => {
        update_fetch_failure_with_html(
            conn,
            page_id,
            FetchFailureUpdate {
                status: "parse_failed",
                content_html_path: &relative_html_path,
                // source_url / source_type / encoding / wayback metadata も保存
            },
        )?;
        return Err(error);
    }
};

conn.execute(
    "UPDATE pages
     SET content_text=?1,
         content_html_path=?2,
         fetch_status='success',
         fetch_error=NULL,
         fetched_at=?3,
         updated_at=?3
     WHERE id=?4",
    params![extracted.content_text, relative_html_path, now_utc(), page_id],
)?;
```

### 4.6 HTMLファイル保存

```rust
// commands/fetch.rs

/// 保存先: {app_data_dir}/originals/sites/{site_id}/works/{work_id}/pages/{page_id}.html
/// 戻り値: アプリデータディレクトリからの相対パス（例: "originals/sites/1/works/2/pages/3.html"）
pub fn save(
    page_id: i64,
    site_id: i64,
    work_id: i64,
    html: &[u8],
) -> Result<String, Error> {
    let app_dir = paths::app_data_dir()?;
    let relative = format!(
        "originals/sites/{}/works/{}/pages/{}.html",
        site_id, work_id, page_id
    );
    let absolute = app_dir.join(&relative);

    // 親ディレクトリを作成
    if let Some(parent) = absolute.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // アトミック書き込み（一時ファイル → リネーム）
    let tmp_path = absolute.with_extension("html.tmp");
    std::fs::write(&tmp_path, html)?;
    std::fs::rename(&tmp_path, &absolute)?;

    Ok(relative)
}

pub fn load(relative_path: &str) -> Result<Vec<u8>, Error> {
    let app_dir = paths::app_data_dir()?;
    let absolute = app_dir.join(relative_path);
    Ok(std::fs::read(absolute)?)
}

pub fn delete(relative_path: &str) -> Result<(), Error> {
    let app_dir = paths::app_data_dir()?;
    let absolute = app_dir.join(relative_path);
    if absolute.exists() {
        std::fs::remove_file(absolute)?;
    }
    Ok(())
}
```

```rust
// lib.rs / commands/fetch.rs

use std::path::PathBuf;

pub fn app_data_dir() -> Result<PathBuf, Error> {
    // macOS: ~/Library/Application Support/NovelArchiveViewer
    let base = dirs::data_dir().ok_or(Error::AppDirNotFound)?;
    let dir = base.join("NovelArchiveViewer");
    if !dir.exists() {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

pub fn database_path() -> Result<PathBuf, Error> {
    Ok(app_data_dir()?.join("app.db"))
}
```

---

## 5. DB接続管理

```rust
// db/connection.rs

pub fn open(path: &Path) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    conn.busy_timeout(Duration::from_secs(5))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    Ok(conn)
}
```

`DbState` は接続そのものではなくDBファイルパスを保持する。
各コマンドは `db.connection()?` で接続を開くため、Rust側の共有 `Mutex<Connection>` で全DB操作を直列化しない。

```rust
// db/migrations.rs

const MIGRATIONS: &[(&str, &str)] = &[
    ("001", include_str!("../../migrations/001_initial_schema.sql")),
    ("002", include_str!("../../migrations/002_add_favorites.sql")),
];

pub fn run_migrations(conn: &mut rusqlite::Connection) -> Result<(), Error> {
    // schema_migrations テーブルが存在しなければ作成
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version TEXT NOT NULL PRIMARY KEY,
            applied_at TEXT NOT NULL
        );
    ")?;

    let applied: HashSet<String> = conn
        .prepare("SELECT version FROM schema_migrations")?
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<_, _>>()?;

    for (version, sql) in MIGRATIONS {
        if applied.contains(*version) {
            continue;
        }

        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (?1, ?2)",
            rusqlite::params![version, now_utc()],
        )?;
        tx.commit()?;

        log::info!("Applied migration: {}", version);
    }

    Ok(())
}
```

---

## 6. エラー型

```rust
// error.rs

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("HTTP error {status}: {url}")]
    HttpError { status: u16, url: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid CSS selector: {0}")]
    InvalidSelector(String),

    #[error("Content not found")]
    ContentNotFound,

    #[error("Empty content extracted")]
    EmptyContent,

    #[error("Invalid Wayback URL")]
    InvalidWaybackUrl,

    #[error("Unknown encoding: {0}")]
    UnknownEncoding(String),

    #[error("Database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("App directory not found")]
    AppDirNotFound,

    #[error("Not implemented")]
    NotImplemented,
}

impl From<Error> for CommandError {
    fn from(e: Error) -> Self {
        let code = match &e {
            Error::HttpError { .. }     => "HTTP_ERROR",
            Error::Network(_)           => "NETWORK_ERROR",
            Error::InvalidSelector(_)   => "INVALID_SELECTOR",
            Error::ContentNotFound      => "CONTENT_NOT_FOUND",
            Error::EmptyContent         => "EMPTY_CONTENT",
            Error::InvalidWaybackUrl    => "INVALID_WAYBACK_URL",
            Error::UnknownEncoding(_)   => "UNKNOWN_ENCODING",
            Error::Db(_)                => "DB_ERROR",
            Error::Io(_)                => "IO_ERROR",
            Error::AppDirNotFound       => "APP_DIR_NOT_FOUND",
            Error::NotImplemented       => "NOT_IMPLEMENTED",
        };
        CommandError {
            code: code.to_string(),
            message: e.to_string(),
        }
    }
}
```

---

## 7. 必要クレート一覧

```toml
[dependencies]
tauri = { version = "2", features = [] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }

# DB
rusqlite = { version = "0.31", features = ["bundled"] }

# HTTP
reqwest = { version = "0.12", features = ["rustls-tls"] }
tokio = { version = "1", features = ["full"] }

# HTMLパース
scraper = "0.19"
regex = "1"

# 文字コード
encoding_rs = "0.8"
chardetng = "0.1"

# パス
dirs = "5"

# エラー処理
thiserror = "1"

# ログ
log = "0.4"
env_logger = "0.11"
```

---

## 8. 一括取得（Phase 7）の処理フロー

要件レベルで決まっている範囲を先取りして記載しておく。

```
batchFetch(work_id, profile)
    │
    ▼
1. profile.index_pattern.url を取得
    │
    ▼
2. profile.index_pattern.link_selector で各話URLを抽出
    │
    ▼
3. 各URLについて pages レコードを pending で作成
    │
    ▼
4. profile.fetch_options.interval_ms 間隔で順次 fetch_page を実行
    │
    ▼
5. 進捗を Tauri Event で React 側に通知（FetchProgressView で表示）
```

進捗通知は `tauri::Manager::emit` を使ってフロントエンドにイベントを送る。

```rust
window.emit("fetch_progress", &ProgressPayload {
    work_id,
    completed,
    total,
    current_page_id,
    current_status,
})?;
```

---

## 9. テスト方針

| レイヤ | テスト対象 | ツール |
|-------|----------|------|
| 単体 | encoding::decode（Shift_JIS/UTF-8の各種パターン） | Rust標準 |
| 単体 | url_handler::parse_wayback_url | Rust標準 |
| 単体 | extractor::extract（モックHTML） | Rust標準 |
| 結合 | fetch_page（テスト用HTTPサーバ + tempdir DB） | tokio-test + wiremock |

---

## 10. 注意事項

### 10.1 既知のリスク

| リスク | 対策 |
|------|------|
| Waybackのレート制限 | 本アプリ側で1秒以上の間隔を強制 |
| 古いサイトの不正なHTML | scraper は寛容にパース、エラー時は parse_failed |
| 巨大ページ（>10MB） | 上限を設けてエラー扱い（要件は数千ページ × 数十KB想定） |
| 文字化け | 警告ログのみ出して保存。検出文字コードを `detected_encoding` に記録してデバッグ可能に |

### 10.2 セキュリティ

- HTTPSをデフォルト推奨、HTTPは警告ログ
- Tauriのallowlistで`http`プラグインを必要分だけ有効化
- ローカルファイルアクセスはアプリデータディレクトリ配下のみ
