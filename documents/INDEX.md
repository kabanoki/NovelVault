# 小説アーカイブ・ビューアー 設計ドキュメント一覧

すべての設計ドキュメントへの索引。実装フェーズに進む前の確認用。

---

## ドキュメント構成

### 要件定義

| ファイル | 内容 | 状態 |
|---------|------|------|
| `requirements_v4.3.md` | 最新の要件定義書（お気に入り機能追加版） | 確定 |

### 詳細設計

| ファイル | 内容 | 状態 |
|---------|------|------|
| `001_initial_schema.sql` | 初期DBスキーマ（sites/works/pages/FTS5/トリガー） | 確定 |
| `002_add_favorites.sql` | お気に入りテーブル追加マイグレーション | 確定 |
| `commands.ts` | Tauri コマンド型定義（TypeScript） | 確定 |
| `types.rs` | Tauri コマンド型定義（Rust） | 確定 |
| `commands.rs` | Tauri コマンド実装骨格（Rust） | 確定 |
| `favorites.ts` | お気に入り型定義（TypeScript） | 確定 |
| `favorites.rs` | お気に入りコマンド実装（Rust） | 確定 |
| `screen_design.md` | 画面一覧・画面遷移図 | 確定 |
| `react_components.md` | React コンポーネント構成 | 確定 |
| `fetch_storage_flow.md` | Rust 側 取得・保存処理フロー | 確定 |
| `site_profile_spec.md` | サイトプロファイル JSON 仕様 | 確定 |

### 試験

| ファイル | 内容 | 状態 |
|---------|------|------|
| `test_unit.md` | 単体試験書（Rust純関数、24ケース） | 実施済み 24/24 PASS |
| `test_integration.md` | 結合試験書（インメモリSQLite、29ケース） | 実施済み 29/29 PASS（`cargo test` 53テスト全PASS） |
| `test_system.md` | 総合試験書（GUIアプリ手動確認、30ケース） | 未実施 |

---

## 開発フェーズ

| Phase | 内容 | 関連ドキュメント |
|------|------|----------------|
| 1 | Tauri + React + SQLite 接続・基本UI・マイグレーション | requirements §11, react §9, fetch_storage §5 |
| 2 | サイト・作品・ページ手動登録、タイトル検索 | screen S-02/03/04/06, react §9 |
| 3 | 手動URL取得（utf-8） | screen S-05, fetch_storage §3-4 |
| 4 | 元HTML保存・auto/shift_jis 文字コード対応 | fetch_storage §4.3 / §4.6 |
| 5 | FTS5 全文検索 | screen S-07, schema §FTS5 |
| 6 | サイトプロファイル保存・切替 | screen S-08, site_profile_spec |
| 6.5 | お気に入り機能 | screen S-09, favorites.* |
| 7 | 一括取得 | screen S-10, fetch_storage §8, site_profile §3 |
| 8 | Wayback特有URL対応 | fetch_storage §4.1, requirements §3.1 |
| 9 | 残りの文字コード対応 | fetch_storage §4.3 |
| 10 | エクスポート・バックアップ・設定画面・重複URL診断 | screen S-11 |

---

## 実装に進む前のチェックリスト

### 環境

- [ ] Rust（cargo）インストール済み
- [ ] Node.js（pnpm or npm）インストール済み
- [ ] Tauri CLI インストール済み（`cargo install tauri-cli`）
- [ ] Xcode Command Line Tools（macOSビルド用）

### プロジェクト初期化

- [ ] `cargo create-tauri-app` でひな形作成
- [ ] `001_initial_schema.sql` を `src-tauri/migrations/` に配置
- [ ] `002_add_favorites.sql` を `src-tauri/migrations/` に配置
- [ ] `Cargo.toml` に依存クレート追加（fetch_storage_flow §7参照）
- [ ] `package.json` に依存パッケージ追加（react_components §10参照）

### 最初に書くコード

1. `src-tauri/src/db/connection.rs` → DB接続初期化
2. `src-tauri/src/db/migrations.rs` → マイグレーションランナー
3. `src-tauri/src/main.rs` → アプリ起動時にマイグレーション実行
4. `src-tauri/src/commands.rs` → site_create / site_list から実装
5. `src/App.tsx` → AppLayout + Sidebar の最小構成

---

## ドキュメント間の参照関係

```
requirements_v4.3.md           (頂点・全ドキュメントの根拠)
        │
        ├──► 001_initial_schema.sql      (データモデルの実装)
        ├──► 002_add_favorites.sql       (お気に入り機能の実装)
        │       │
        │       ▼
        │   types.rs / commands.ts       (型定義)
        │       │
        │       ▼
        │   commands.rs / favorites.rs   (Rust コマンド実装)
        │
        ├──► screen_design.md            (UI設計)
        │       │
        │       ▼
        │   react_components.md          (フロントエンド実装)
        │
        ├──► fetch_storage_flow.md       (取得処理の詳細)
        │
        └──► site_profile_spec.md        (プロファイル仕様)
```

---

## 未決事項（要件定義§13より）

実装中に判断すべき項目。

- [ ] サイトプロファイル設定UIの形（GUIで要素ピッカー or JSON直接編集）
- [ ] `link_url_pattern` の具体的な書式 → site_profile_spec で確定
- [ ] 画像の扱い（小説中の挿絵など）
- [ ] 誤削除対策（`deleted_at` による論理削除を将来検討）
- [x] エクスポート機能（テキスト出力など）の要否 → Phase 10 MVPでは選択中ページの本文を `.txt` 出力
- [x] アプリ全体のグローバル設定保存先（`app_settings` テーブル or 設定ファイル） → Phase 10 MVPではリーダー表示設定を `localStorage` に保存
- [ ] マイグレーションツールの選定（refinery / sqlx-migrate / 自前実装）
- [ ] お気に入り機能の将来拡張（複数リスト・メモ・タグ）の要否
