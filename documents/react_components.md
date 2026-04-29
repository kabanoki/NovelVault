# Reactコンポーネント構成設計書

対応要件定義: v4.3 / 画面設計: screen_design.md

---

## 1. ディレクトリ構成

```
src/
├── main.tsx                      # エントリポイント
├── App.tsx                       # ルートコンポーネント・ルーティング
├── api/                          # Tauri invoke ラッパー
│   ├── client.ts                 # 共通invoke関数・エラーハンドリング
│   ├── sites.ts
│   ├── siteProfiles.ts
│   ├── works.ts
│   ├── pages.ts
│   ├── fetch.ts
│   ├── search.ts
│   └── favorites.ts
├── types/                        # 型定義
│   ├── entities.ts               # Site / Work / Page / Favorite 等
│   ├── commands.ts               # コマンド引数・レスポンス型
│   └── common.ts                 # SourceType / Encoding / FetchStatus
├── stores/                       # 状態管理（Zustand）
│   ├── sidebarStore.ts           # サイドバー展開状態
│   ├── selectionStore.ts         # 選択中エンティティ
│   └── settingsStore.ts          # フォントサイズ等のローカル設定
├── hooks/                        # カスタムフック
│   ├── useSites.ts
│   ├── useWorks.ts
│   ├── usePages.ts
│   ├── useFavorites.ts
│   ├── useSearch.ts
│   └── useKeyboardShortcuts.ts
├── components/
│   ├── layout/                   # レイアウト
│   │   ├── AppLayout.tsx
│   │   ├── TopBar.tsx
│   │   └── Sidebar/
│   │       ├── Sidebar.tsx
│   │       ├── SiteNode.tsx
│   │       ├── WorkNode.tsx
│   │       └── PageNode.tsx
│   ├── reader/                   # 本文ビューワー
│   │   ├── PageReader.tsx
│   │   ├── PageHeader.tsx        # タイトル・お気に入りボタン
│   │   ├── PageNavigation.tsx    # 前後ページ
│   │   └── FavoriteToggle.tsx
│   ├── search/                   # 検索
│   │   ├── SearchBar.tsx
│   │   ├── SearchResultsView.tsx
│   │   ├── TitleSearchResults.tsx
│   │   └── FullTextSearchResults.tsx
│   ├── favorites/                # お気に入り
│   │   ├── FavoritesView.tsx
│   │   ├── FavoriteSiteGroup.tsx
│   │   ├── FavoriteWorkGroup.tsx
│   │   └── FavoriteItem.tsx
│   ├── modals/                   # モーダル
│   │   ├── ModalContainer.tsx
│   │   ├── SiteFormModal.tsx
│   │   ├── WorkFormModal.tsx
│   │   ├── PageFormModal.tsx
│   │   ├── FetchUrlModal.tsx
│   │   └── ConfirmDialog.tsx
│   ├── profiles/                 # サイトプロファイル
│   │   ├── ProfileManagerView.tsx
│   │   ├── ProfileList.tsx
│   │   └── ProfileEditor.tsx     # JSON+フォームの併用エディタ
│   ├── progress/                 # 取得進捗
│   │   └── FetchProgressView.tsx
│   ├── settings/                 # 設定
│   │   └── SettingsView.tsx
│   └── ui/                       # 汎用UIパーツ
│       ├── Button.tsx
│       ├── Input.tsx
│       ├── Select.tsx
│       ├── Textarea.tsx
│       ├── Modal.tsx
│       ├── Toast.tsx
│       └── EmptyState.tsx
└── utils/
    ├── format.ts                 # 日付フォーマット等
    └── error.ts                  # エラー整形
```

---

## 2. コンポーネント階層図

```
App
└── AppLayout
    ├── TopBar
    │   ├── AddSiteButton
    │   ├── SearchBar
    │   ├── FavoritesButton
    │   └── SettingsButton
    │
    ├── Sidebar
    │   └── SiteNode (×n)
    │       └── WorkNode (×n)
    │           └── PageNode (×n)
    │
    └── MainPane (条件分岐で切り替え)
        ├── PageReader              # 通常時
        │   ├── PageHeader
        │   │   └── FavoriteToggle
        │   ├── ContentBody
        │   └── PageNavigation
        │
        ├── SearchResultsView       # 検索時
        │   ├── TitleSearchResults
        │   └── FullTextSearchResults
        │
        ├── FavoritesView           # お気に入り表示時
        │   └── FavoriteSiteGroup (×n)
        │       └── FavoriteWorkGroup (×n)
        │           └── FavoriteItem (×n)
        │
        ├── ProfileManagerView      # プロファイル管理時
        ├── FetchProgressView       # 一括取得時
        └── SettingsView            # 設定時

ModalContainer (Portal)
├── SiteFormModal
├── WorkFormModal
├── PageFormModal
├── FetchUrlModal
└── ConfirmDialog
```

---

## 3. 状態管理方針

### 3.1 ライブラリ選定

- **サーバ状態（DB由来）**：TanStack Query (React Query)
- **クライアント状態（UI由来）**：Zustand
- **フォーム**：React Hook Form

### 3.2 役割分担

| 状態の種類 | 例 | 管理方法 |
|----------|-----|---------|
| サーバ状態 | サイト一覧、作品一覧、お気に入り一覧 | TanStack Query（キャッシュ + 自動再取得） |
| UI状態 | サイドバー展開、選択中ページ、現在の表示モード | Zustand |
| フォーム状態 | モーダル内の入力値 | React Hook Form |
| 一時的UI | モーダル開閉、トースト | Zustand or ローカルstate |

### 3.3 Zustand ストア定義例

```typescript
// stores/selectionStore.ts
interface SelectionState {
  // 表示モード
  viewMode: 'reader' | 'search' | 'favorites' | 'profiles' | 'progress' | 'settings';
  setViewMode: (mode: ViewMode) => void;

  // 選択中エンティティ
  selectedPageId: number | null;
  selectPage: (id: number) => void;

  // 検索クエリ（検索モード時のみ）
  searchQuery: string;
  searchType: 'title' | 'fulltext';
  setSearch: (query: string, type: 'title' | 'fulltext') => void;
}

// stores/sidebarStore.ts
interface SidebarState {
  expandedSiteIds: Set<number>;
  expandedWorkIds: Set<number>;
  toggleSite: (id: number) => void;
  toggleWork: (id: number) => void;
  expandPath: (siteId: number, workId: number) => void;  // ジャンプ時に使う
}

// stores/modalStore.ts
interface ModalState {
  openModal:
    | null
    | { type: 'site'; siteId?: number }
    | { type: 'work'; workId?: number; siteId?: number }
    | { type: 'page'; pageId?: number; workId?: number }
    | { type: 'fetch'; pageId: number }
    | { type: 'confirm'; title: string; message: string; onConfirm: () => void };
  show: (modal: NonNullable<ModalState['openModal']>) => void;
  close: () => void;
}
```

### 3.4 TanStack Query キー設計

```typescript
// クエリキーは配列で階層的に管理し、無効化を効率化する
const queryKeys = {
  sites: ['sites'] as const,
  worksBySite: (siteId: number) => ['works', siteId] as const,
  pagesByWork: (workId: number) => ['pages', workId] as const,
  page: (id: number) => ['page', id] as const,
  favorites: ['favorites'] as const,
  favoriteCheck: (pageId: number) => ['favorite', pageId] as const,
  searchTitles: (query: string) => ['search', 'titles', query] as const,
  searchFullText: (query: string) => ['search', 'fulltext', query] as const,
};

// 例：作品作成後にサイト配下の作品一覧を無効化
queryClient.invalidateQueries({ queryKey: queryKeys.worksBySite(siteId) });
```

---

## 4. 主要コンポーネント詳細

### 4.1 AppLayout

**責務**：全体レイアウト・トップバー・サイドバー・メインペイン配置

```tsx
// 擬似コード
function AppLayout() {
  const viewMode = useSelectionStore(s => s.viewMode);

  return (
    <div className="flex flex-col h-screen">
      <TopBar />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar />
        <MainPane viewMode={viewMode} />
      </div>
      <ModalContainer />
      <ToastContainer />
    </div>
  );
}

function MainPane({ viewMode }: { viewMode: ViewMode }) {
  switch (viewMode) {
    case 'reader':    return <PageReader />;
    case 'search':    return <SearchResultsView />;
    case 'favorites': return <FavoritesView />;
    case 'profiles':  return <ProfileManagerView />;
    case 'progress':  return <FetchProgressView />;
    case 'settings':  return <SettingsView />;
  }
}
```

### 4.2 Sidebar

**責務**：サイト → 作品 → ページの階層ツリー表示

**設計のポイント**
- サイト一覧は `useSites()` で取得
- サイト展開時のみ作品一覧を取得（遅延ロード）
- 作品展開時のみページ一覧を取得（遅延ロード）
- 各ノードはメモ化（React.memo）して再レンダリング抑制

```tsx
function Sidebar() {
  const { data: sites } = useSites();

  if (!sites || sites.length === 0) {
    return <EmptyState message="+ サイト追加 からはじめましょう" />;
  }

  return (
    <aside className="w-64 border-r overflow-y-auto">
      {sites.map(site => (
        <SiteNode key={site.id} site={site} />
      ))}
    </aside>
  );
}

function SiteNode({ site }: { site: Site }) {
  const isExpanded = useSidebarStore(s => s.expandedSiteIds.has(site.id));
  const toggle = useSidebarStore(s => s.toggleSite);
  const { data: works } = useWorks(site.id, { enabled: isExpanded });

  return (
    <div>
      <button onClick={() => toggle(site.id)}>
        {isExpanded ? '▼' : '▶'} {site.name}
      </button>
      {isExpanded && works?.map(w => <WorkNode key={w.id} work={w} />)}
    </div>
  );
}
```

### 4.3 PageReader

**責務**：本文表示・お気に入りトグル・前後ナビ

```tsx
function PageReader() {
  const pageId = useSelectionStore(s => s.selectedPageId);
  const { data: page, isLoading } = usePage(pageId);

  if (!pageId) return <EmptyState message="サイドバーからページを選択してください" />;
  if (isLoading) return <Spinner />;
  if (!page) return <ErrorState />;

  return (
    <main className="flex-1 overflow-y-auto p-8">
      <PageHeader page={page} />
      <article className="prose max-w-none mt-6">
        {page.contentText?.split('\n').map((line, i) => (
          <p key={i}>{line}</p>
        ))}
      </article>
      <PageNavigation currentPageId={pageId} workId={page.workId} />
    </main>
  );
}
```

### 4.4 FavoriteToggle

**責務**：お気に入り登録/解除（楽観的更新）

```tsx
function FavoriteToggle({ pageId }: { pageId: number }) {
  const { data: status } = useFavoriteCheck(pageId);
  const add = useFavoriteAdd();
  const remove = useFavoriteRemove();

  const handleToggle = () => {
    if (status?.isFavorite) {
      remove.mutate({ pageId });
    } else {
      add.mutate({ pageId });
    }
  };

  return (
    <button onClick={handleToggle} aria-label="お気に入り">
      {status?.isFavorite ? '★' : '☆'}
    </button>
  );
}
```

useFavoriteAdd / Remove 内部では `onMutate` で楽観的更新、`onError` でロールバック、`onSettled` で `favorites` と `favoriteCheck(pageId)` を invalidate する。

### 4.5 FavoritesView

**責務**：お気に入り一覧をサイト・作品でグループ化表示

```tsx
function FavoritesView() {
  const { data: grouped, isLoading } = useFavorites();
  const selectPage = useSelectionStore(s => s.selectPage);
  const setViewMode = useSelectionStore(s => s.setViewMode);
  const expandPath = useSidebarStore(s => s.expandPath);

  const handleJump = (siteId: number, workId: number, pageId: number) => {
    expandPath(siteId, workId);
    selectPage(pageId);
    setViewMode('reader');
  };

  if (isLoading) return <Spinner />;
  if (!grouped?.groups.length) {
    return <EmptyState message="お気に入りに登録したページがここに表示されます" />;
  }

  return (
    <main className="p-6">
      <h2>★ お気に入り</h2>
      {grouped.groups.map(siteGroup => (
        <FavoriteSiteGroup
          key={siteGroup.siteId}
          group={siteGroup}
          onJump={handleJump}
        />
      ))}
    </main>
  );
}
```

### 4.6 SearchBar

**責務**：検索クエリ入力・送信・タブ切替

**設計のポイント**
- デバウンス（300ms）で TanStack Query を呼び出し
- 入力中は前回結果を保持して画面のチラつきを防ぐ
- `⌘+F` フォーカスを `useKeyboardShortcuts` で実装

### 4.7 FetchUrlModal

**責務**：URL取得ダイアログ

**画面状態**

```typescript
type FetchState =
  | { phase: 'input' }                              // 入力中
  | { phase: 'fetching' }                           // 取得中
  | { phase: 'success'; result: FetchPageResult }   // 成功・プレビュー表示
  | { phase: 'error'; error: string };              // 失敗
```

**設計のポイント**
- 取得中はキャンセル不可（バックエンドの中断機構が無いため）
- 成功時は本文プレビューを表示してから保存確定
- 失敗時はエラー詳細を表示し、セレクタ修正→再試行を促す

### 4.8 ProfileEditor

**責務**：サイトプロファイルJSONの編集

**設計のポイント**
- 上部はフォーム（name / source_type / encoding / 主要セレクタ）
- 下部はJSON直接編集（エキスパート向け）
- 双方向バインディング：フォーム変更 → JSON更新、JSON変更 → 検証してフォーム反映
- スキーマバリデーションは zod を使用

---

## 5. ルーティング方針

**SPAだがブラウザのURLは使わず、内部状態（Zustand）で画面切替**する。

理由：
- Tauri デスクトップアプリのため、URL共有等の必要がない
- 状態管理が単純化する

ただし以下の操作で「戻る」が発生する場面があるため、簡易的な履歴スタックを `selectionStore` に持つ：
- 検索結果から本文へジャンプ → 戻る
- お気に入りから本文へジャンプ → 戻る

```typescript
// 履歴スタック
interface HistoryState {
  history: ViewSnapshot[];
  push: (snapshot: ViewSnapshot) => void;
  back: () => void;
}

interface ViewSnapshot {
  viewMode: ViewMode;
  selectedPageId?: number;
  searchQuery?: string;
}
```

---

## 6. スタイリング方針

### 6.1 Tailwind CSS

- 基本はTailwindのユーティリティクラスで構築
- 共通スタイルは `components/ui/` のコンポーネントに集約
- `prose` クラスを本文表示に活用

### 6.2 macOSらしい見栄え

| 要素 | スタイル方針 |
|------|------------|
| 背景 | `bg-zinc-50` / ダークモード `bg-zinc-900` |
| 区切り線 | `border-zinc-200` / `border-zinc-800` |
| 角丸 | `rounded-lg`（少し大きめ） |
| 影 | `shadow-sm`（控えめ） |
| フォント | システムフォント（`font-sans`） |
| アクセントカラー | macOSブルー風 `text-blue-600` |

### 6.3 ダークモード対応

`prefers-color-scheme` に追従。Tailwind の `dark:` プレフィックスを使用。

---

## 7. パフォーマンス方針

### 7.1 レンダリング最適化

- サイドバーノードは `React.memo` で必要時のみ再描画
- 大きなページ本文は仮想スクロール検討（react-window）
- お気に入り一覧は階層構造のため仮想化不要（数百件まで想定）

### 7.2 データ取得の最適化

- サイドバーは遅延ロード（展開時に取得）
- ページ一覧は `PageSummary`（本文除外）で取得し、本文は選択時のみ取得
- TanStack Query のキャッシュで重複取得を回避

### 7.3 検索の応答性

- 検索クエリは300msデバウンス
- 全文検索結果は最大50件に制限（要件レベルで定義済み）

---

## 8. テスト方針

| レイヤ | ツール | 対象 |
|-------|-------|------|
| ユニット | Vitest | 純関数（フォーマッタ、グルーピング等） |
| コンポーネント | Vitest + Testing Library | UIコンポーネント |
| E2E | Playwright（Tauri経由） | 主要ユーザーフロー |

MVPでは最低限、`api/` 層のラッパーと `utils/` 内の関数にユニットテストを書く。

---

## 9. フェーズ別 実装順序

| フェーズ | 実装するコンポーネント |
|---------|--------------------|
| Phase 1 | AppLayout / TopBar / Sidebar（空状態） / api/client / Zustand基盤 |
| Phase 2 | SiteFormModal / WorkFormModal / PageFormModal / SearchBar / TitleSearchResults / ConfirmDialog / row action buttons |
| Phase 3 | FetchUrlModal / PageReader |
| Phase 4 | （PageReader拡張・文字コード対応） |
| Phase 5 | FullTextSearchResults |
| Phase 6 | ProfileManagerView / ProfileEditor |
| Phase 6.5 | FavoritesView / FavoriteToggle |
| Phase 7 | FetchProgressView |
| Phase 10 | SettingsView / readerSettings(localStorage) / export・backup API hooks |

---

## 10. 依存パッケージ一覧

```json
{
  "dependencies": {
    "react": "^18",
    "react-dom": "^18",
    "@tauri-apps/api": "^2",
    "@tanstack/react-query": "^5",
    "zustand": "^4",
    "react-hook-form": "^7",
    "zod": "^3",
    "tailwindcss": "^3",
    "lucide-react": "^0.3"
  },
  "devDependencies": {
    "typescript": "^5",
    "vite": "^5",
    "@vitejs/plugin-react": "^4",
    "vitest": "^1",
    "@testing-library/react": "^14"
  }
}
```
