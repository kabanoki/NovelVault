# サイトプロファイル JSON 仕様書

対応要件定義: v4.3

---

## 1. 概要

サイトプロファイルは、サイトごとに異なるHTML構造を吸収するための設定JSON。
1サイトに複数のプロファイルを持つことができ、作品単位で使い分けられる。

DBの `site_profiles.profile_json` に文字列として保存される。

---

## 2. ルート構造

```json
{
  "schema_version": 1,
  "name": "プロファイル名",
  "base_url": "https://example.com/",
  "source_type": "normal",
  "encoding": "auto",
  "index_pattern": { ... },
  "page_pattern":  { ... },
  "fetch_options": { ... }
}
```

| フィールド | 型 | 必須 | 説明 |
|-----------|----|----|------|
| `schema_version` | number | 必須 | スキーマバージョン。現在は `1` |
| `name` | string | 必須 | プロファイル表示名 |
| `base_url` | string | 必須 | サイトのベースURL（相対リンク解決に使用） |
| `source_type` | enum | 必須 | `normal` / `wayback` / `local` |
| `encoding` | enum | 必須 | `auto` / `utf-8` / `shift_jis` / `euc-jp` |
| `index_pattern` | object | 任意 | 目次ページの設定（一括取得時に使用） |
| `page_pattern` | object | 必須 | 個別ページの本文抽出設定 |
| `fetch_options` | object | 任意 | 取得時のオプション |

---

## 3. index_pattern（目次パターン）

目次ページから各話リンクを抽出するための設定。
Phase 7 の一括取得機能で使用する。MVPでは未使用。

```json
{
  "index_pattern": {
    "url": "https://example.com/index.html",
    "link_selector": "a.chapter-link",
    "link_url_pattern": null,
    "link_url_template": null
  }
}
```

| フィールド | 型 | 必須 | 説明 |
|-----------|----|----|------|
| `url` | string | 任意 | 目次ページのURL |
| `link_selector` | string | 任意 | 各話リンクのCSSセレクタ |
| `link_url_pattern` | string \| null | 任意 | URLが連番の場合の正規表現パターン |
| `link_url_template` | string \| null | 任意 | 連番URL生成のテンプレート |

### 3.1 目次ページ型（link_selector）

目次にリンクが並ぶサイト用。

```json
{
  "url": "http://rinrin.saiin.net/~yokubounotou/ss/slmbst/index01.html",
  "link_selector": "a[href$='.html']",
  "link_url_pattern": null,
  "link_url_template": null
}
```

各 `<a>` タグの `href` を取得し、`base_url` で相対URL解決して各話URLとする。

### 3.2 連番URL型（link_url_template）

`001.html`, `002.html` のような連番URLのサイト用。

```json
{
  "url": null,
  "link_selector": null,
  "link_url_pattern": null,
  "link_url_template": "http://example.com/{n:03d}.html",
  "link_url_range": { "start": 1, "end": 50 }
}
```

| 書式 | 意味 | 例 |
|------|-----|-----|
| `{n}` | 連番（パディングなし） | 1, 2, ..., 10 |
| `{n:03d}` | 3桁ゼロ埋め | 001, 002, ..., 010 |
| `{n:04d}` | 4桁ゼロ埋め | 0001, 0002, ..., 0010 |

`link_url_range.start` から `link_url_range.end` まで生成する。

### 3.3 link_url_pattern の用途

目次ページのHTMLから取得したリンクのうち、特定のパターンに合致するものだけを各話リンクとして採用する正規表現。

例：目次にナビゲーションリンクと各話リンクが混在する場合

```json
{
  "url": "http://example.com/index.html",
  "link_selector": "a",
  "link_url_pattern": "^chapter\\d+\\.html$",
  "link_url_template": null
}
```

`link_selector` で取得した全 `<a>` のうち、`href` が `chapter数字.html` 形式のものだけを採用する。

---

## 4. page_pattern（ページパターン）

個別ページから本文を抽出するための設定。**MVP では必須**。

```json
{
  "page_pattern": {
    "title_selector": "h1",
    "content_selector": "div#honbun",
    "remove_selectors": [
      "script",
      "style",
      ".footer",
      ".nav"
    ]
  }
}
```

| フィールド | 型 | 必須 | 説明 |
|-----------|----|----|------|
| `title_selector` | string | 必須 | タイトルのCSSセレクタ |
| `content_selector` | string | 必須 | 本文のCSSセレクタ |
| `remove_selectors` | string[] | 任意 | 本文から除外する要素のCSSセレクタ |

### 4.1 セレクタの書き方

CSS セレクタは scraper クレートが対応する範囲（CSS Selector Level 4 の主要部分）。

| 用途 | セレクタ例 |
|------|-----------|
| ID指定 | `#honbun`, `div#main` |
| クラス指定 | `.story-body`, `td.text` |
| 属性指定 | `td[width="600"]`, `a[href$=".html"]` |
| 子孫 | `div#main p` |
| 直接の子 | `div > p` |
| 最初の子 | `p:first-child` |

### 4.2 古いサイト向けのコツ

`<table>` レイアウトの古いサイトでは以下のパターンが多い。

```json
{
  "title_selector": "td > b",
  "content_selector": "td[width='600']",
  "remove_selectors": [
    "table table",
    "a[href*='index']"
  ]
}
```

`<font>` タグや `<center>` タグが本文を囲んでいる場合：

```json
{
  "content_selector": "body > center > font",
  "remove_selectors": ["a"]
}
```

### 4.3 remove_selectors の動作

`content_selector` でマッチした本文ノードの内側で、`remove_selectors` にマッチする子孫ノードを除外してテキスト抽出する。

例：

```html
<div id="honbun">
  <p>本文です。</p>
  <div class="footer">
    <a href="/back">戻る</a>
  </div>
</div>
```

```json
{
  "content_selector": "#honbun",
  "remove_selectors": [".footer"]
}
```

抽出結果：`本文です。`（`.footer` は除外される）

---

## 5. fetch_options（取得オプション）

```json
{
  "fetch_options": {
    "interval_ms": 1000,
    "user_agent": "NovelArchiveViewer/0.1",
    "timeout_sec": 30,
    "max_retries": 2
  }
}
```

| フィールド | 型 | 必須 | デフォルト | 説明 |
|-----------|----|----|----------|------|
| `interval_ms` | number | 任意 | 1000 | 連続リクエスト間隔（ms） |
| `user_agent` | string | 任意 | アプリ既定値 | User-Agent ヘッダ |
| `timeout_sec` | number | 任意 | 30 | リクエストタイムアウト（秒） |
| `max_retries` | number | 任意 | 0 | リトライ回数（MVPでは0、Phase 7以降で活用） |

### 5.1 サイトへの配慮

- `interval_ms` は **最低 1000ms**（1秒）を推奨。Wayback Machineへのアクセスは特に注意
- `user_agent` には連絡先を含めることが望ましい（例：`NovelArchiveViewer/0.1 (https://github.com/...)`）

---

## 6. 完全なプロファイル例

### 6.1 目次ページ型（個人サイト）

```json
{
  "schema_version": 1,
  "name": "rinrin.saiin.net 標準",
  "base_url": "http://rinrin.saiin.net/~yokubounotou/ss/slmbst/",
  "source_type": "normal",
  "encoding": "auto",
  "index_pattern": {
    "url": "http://rinrin.saiin.net/~yokubounotou/ss/slmbst/index01.html",
    "link_selector": "a[href$='.html']",
    "link_url_pattern": "^\\d+\\.html$|^番外編.*\\.htm$",
    "link_url_template": null
  },
  "page_pattern": {
    "title_selector": "title",
    "content_selector": "body",
    "remove_selectors": ["a", "hr"]
  },
  "fetch_options": {
    "interval_ms": 1500,
    "user_agent": "NovelArchiveViewer/0.1"
  }
}
```

### 6.2 Wayback Machine 連番型

```json
{
  "schema_version": 1,
  "name": "onyx Wayback アーカイブ",
  "base_url": "https://web.archive.org/web/20040604075856/http://www.onyx.dti.ne.jp/~sultan/",
  "source_type": "wayback",
  "encoding": "shift_jis",
  "index_pattern": {
    "url": null,
    "link_selector": null,
    "link_url_pattern": null,
    "link_url_template": "https://web.archive.org/web/20040604075856/http://www.onyx.dti.ne.jp/~sultan/Dcup{n:03d}.html",
    "link_url_range": { "start": 0, "end": 50 }
  },
  "page_pattern": {
    "title_selector": "title",
    "content_selector": "body",
    "remove_selectors": ["table", "a"]
  },
  "fetch_options": {
    "interval_ms": 2000,
    "user_agent": "NovelArchiveViewer/0.1"
  }
}
```

### 6.3 ブログ型（fc2）

```json
{
  "schema_version": 1,
  "name": "fc2blog 標準",
  "base_url": "https://web.archive.org/web/20120124192301/http://sakaimaroudo.blog118.fc2.com/",
  "source_type": "wayback",
  "encoding": "auto",
  "index_pattern": {
    "url": null,
    "link_selector": "a.entry-title",
    "link_url_pattern": "blog-entry-\\d+\\.html",
    "link_url_template": null
  },
  "page_pattern": {
    "title_selector": "h2.title a",
    "content_selector": ".entry-body",
    "remove_selectors": [
      ".entry-tags",
      ".comment-area",
      "script"
    ]
  },
  "fetch_options": {
    "interval_ms": 2000,
    "user_agent": "NovelArchiveViewer/0.1"
  }
}
```

---

## 7. バリデーション

プロファイル保存時に以下を検証する。

| 項目 | 検証内容 |
|------|---------|
| `schema_version` | 1 のみ許容（将来のため） |
| `name` | 1〜100文字 |
| `base_url` | 有効なURL（http/https/file） |
| `source_type` | enum 値であること |
| `encoding` | enum 値であること |
| `page_pattern.title_selector` | 有効なCSSセレクタ |
| `page_pattern.content_selector` | 有効なCSSセレクタ |
| `page_pattern.remove_selectors[*]` | 各要素が有効なCSSセレクタ |
| `index_pattern.link_url_template` | `{n}` または `{n:Nd}` を含むこと |
| `index_pattern.link_url_range.end` | `start` 以上であること |
| `fetch_options.interval_ms` | 100以上 |

実装は `serde_json` でパース後、構造体に対して個別バリデーションを行う。

---

## 8. プロファイル作成のワークフロー

### 8.1 GUIでの作成（Phase 6 以降）

1. サイトを登録する（`siteCreate`）
2. プロファイル管理画面を開く（S-08）
3. 「+ 新規プロファイル」をクリック
4. プロファイルエディタで以下を入力：
   - 名前
   - JSON直接編集 or フォーム入力
5. テスト用URLを入れて「テスト取得」ボタンで動作確認
6. 保存

### 8.2 セレクタの探し方（ユーザー向けTips）

1. ブラウザで対象サイトを開く
2. 本文部分を右クリック → 「検証」（Inspect）
3. 開発者ツールで本文を囲む要素を探す
4. その要素のID・クラス・タグからセレクタを組み立てる

例：本文が `<div class="story-body">` の場合 → `content_selector: ".story-body"`

---

## 9. 将来の拡張案

| 拡張項目 | 説明 |
|---------|------|
| `page_pattern.next_link_selector` | 次のページへのリンクセレクタ。クリック型のページネーション対応 |
| `page_pattern.images.save` | 挿絵画像も保存するか（要件外） |
| `index_pattern.pagination` | 目次が複数ページに分かれる場合の対応 |
| `transform_rules` | 取得後のテキスト整形ルール（ルビ展開、注記処理等） |

これらは `schema_version` を上げて対応する。

---

## 10. デフォルトプロファイル（フォールバック）

ユーザーが何も設定せずにURL取得を試みた場合のデフォルト。

```json
{
  "schema_version": 1,
  "name": "デフォルト",
  "base_url": "",
  "source_type": "normal",
  "encoding": "auto",
  "page_pattern": {
    "title_selector": "title",
    "content_selector": "body",
    "remove_selectors": ["script", "style", "nav", "header", "footer"]
  },
  "fetch_options": {
    "interval_ms": 1000,
    "user_agent": "NovelArchiveViewer/0.1"
  }
}
```

これでとりあえず取得は試せる。本文抽出の精度はサイトに合わせて個別調整が必要。
