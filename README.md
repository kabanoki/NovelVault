# NovelVault

NovelVault は、閉鎖されたWebサイトや個人サイト、ブログなどに掲載されていた小説を、Wayback Machine などのWebアーカイブや現存ページから取得し、ローカル環境に保存・閲覧・検索できるMac向けデスクトップアプリです。

## 主な機能

- サイト、作品、ページを階層管理
- ページ本文の手動登録、編集、削除
- URL指定による本文取得とCSSセレクタ抽出
- サイトプロファイルJSONによる取得設定の保存
- 目次ページまたは連番URLテンプレートからの一括取得
- Wayback Machine URLのメタデータ保存
- 元HTMLのローカル保存
- タイトル検索、SQLite FTS5による全文検索
- ページ単位のお気に入り登録と一覧表示
- 文字コード `auto` / `utf-8` / `shift_jis` / `euc-jp` 対応
- リーダーフォントサイズ設定
- 選択ページのテキストエクスポート
- SQLite DBバックアップ作成

## 技術スタック

- Tauri v2
- React + TypeScript
- Tailwind CSS
- Rust
- SQLite
- reqwest / scraper

## 想定開発環境

このリポジトリでは、以下のバージョンで動作確認しています。

| ツール | 確認済みバージョン |
|--------|--------------------|
| Node.js | v25.9.0 |
| npm | 11.12.1 |
| Rust | 1.95.0 |
| Cargo | 1.95.0 |

## 開発環境の起動

依存関係をインストールします。

```bash
npm install
```

Tauriアプリを開発モードで起動します。

```bash
npm run tauri -- dev
```

Vite の開発サーバーは通常 `http://localhost:1420/` で起動します。Tauri のデスクトップウィンドウから利用する想定です。

## 検証コマンド

フロントエンドの型チェックとビルド:

```bash
npm run build
```

Rust 側のチェック:

```bash
cd src-tauri
cargo check
cargo fmt --check
```

## データ保存先

アプリデータは macOS の Application Support 配下に保存されます。

```text
~/Library/Application Support/NovelVault/
  app.db
  originals/
  exports/
  backups/
```

- `app.db`: SQLiteデータベース
- `originals/`: 取得した元HTML
- `exports/`: ページ本文のテキスト書き出し
- `backups/`: DBバックアップ

## ドキュメント

要件定義と設計資料は `documents/` 配下にあります。

- `documents/requirements_v4.3.md`: 要件定義
- `documents/screen_design.md`: 画面設計
- `documents/react_components.md`: React構成
- `documents/site_profile_spec.md`: サイトプロファイル仕様
- `documents/fetch_storage_flow.md`: 取得・保存フロー

## 今後の検討項目

- ローカルHTMLファイル取り込み
- 画像や挿絵の保存方針
- データ復元機能
- 既定User-Agentとリクエストウェイトのグローバル設定
- 誤削除対策としての論理削除
