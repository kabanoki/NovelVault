import { BookOpen, FilePlus2, Library, Plus, Star } from 'lucide-react';
import { ReactNode } from 'react';
import { useDuplicateSourceUrlList } from '../../hooks/useDiagnostics';
import { useFavoriteAdd, useFavoriteCheck, useFavoriteRemove, useFavorites } from '../../hooks/useFavorites';
import { useBackupDatabase, useExportPageText } from '../../hooks/useFiles';
import { usePage } from '../../hooks/usePages';
import { ReaderFontSize } from '../../hooks/useReaderSettings';
import { useFullTextSearch, useTitleSearch } from '../../hooks/useSearch';
import type { Page } from '../../types/entities';

interface MainPaneProps {
  searchQuery: string;
  searchType: 'title' | 'fulltext';
  showFavorites: boolean;
  showSettings: boolean;
  readerFontSize: ReaderFontSize;
  selectedSiteId: number | null;
  selectedWorkId: number | null;
  selectedPageId: number | null;
  onAddWork: (siteId: number) => void;
  onAddPage: (workId: number) => void;
  onBulkFetch: (workId: number, siteId: number) => void;
  onFetchPage: (page: Page) => void;
  onCloseFavorites: () => void;
  onCloseSettings: () => void;
  onReaderFontSizeChange: (value: ReaderFontSize) => void;
  onClearSearch: () => void;
  onSelectSite: (id: number | null) => void;
  onSelectWork: (id: number | null) => void;
  onSelectPage: (id: number | null) => void;
}

export function MainPane({
  searchQuery,
  searchType,
  showFavorites,
  showSettings,
  readerFontSize,
  selectedSiteId,
  selectedWorkId,
  selectedPageId,
  onAddWork,
  onAddPage,
  onBulkFetch,
  onFetchPage,
  onCloseFavorites,
  onCloseSettings,
  onReaderFontSizeChange,
  onClearSearch,
  onSelectSite,
  onSelectWork,
  onSelectPage,
}: MainPaneProps) {
  if (showFavorites) {
    return (
      <FavoritesView
        onClose={onCloseFavorites}
        onSelectPage={(pageId, workId, siteId) => {
          onSelectSite(siteId);
          onSelectWork(workId);
          onSelectPage(pageId);
          onCloseFavorites();
        }}
      />
    );
  }

  if (showSettings) {
    return (
      <SettingsView
        selectedPageId={selectedPageId}
        readerFontSize={readerFontSize}
        onFontSizeChange={onReaderFontSizeChange}
        onClose={onCloseSettings}
      />
    );
  }

  if (searchQuery.trim()) {
    return (
      <SearchResults
        query={searchQuery}
        searchType={searchType}
        onSelectSite={(id) => {
          onSelectSite(id);
          onSelectWork(null);
          onSelectPage(null);
          onClearSearch();
        }}
        onSelectWork={(workId, siteId) => {
          onSelectSite(siteId);
          onSelectWork(workId);
          onSelectPage(null);
          onClearSearch();
        }}
        onSelectPage={(pageId, workId, siteId) => {
          onSelectSite(siteId);
          onSelectWork(workId);
          onSelectPage(pageId);
          onClearSearch();
        }}
      />
    );
  }

  if (selectedPageId) {
    return <PageReader pageId={selectedPageId} readerFontSize={readerFontSize} onFetch={onFetchPage} />;
  }

  return (
    <main className="min-w-0 flex-1 overflow-y-auto bg-white">
      <div className="mx-auto grid max-w-4xl gap-6 px-8 py-8">
        <section>
          <div className="flex items-center gap-2">
            <Library className="h-5 w-5 text-zinc-500" aria-hidden="true" />
            <h1 className="text-xl font-semibold">手動登録</h1>
          </div>
          <p className="mt-2 text-sm leading-6 text-zinc-500">
            サイドバーでサイトまたは作品を選び、作品とページを追加します。
          </p>
        </section>
        <div className="flex flex-wrap gap-3">
          <button
            className="inline-flex h-9 items-center gap-1 rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300"
            disabled={selectedSiteId === null}
            onClick={() => selectedSiteId !== null && onAddWork(selectedSiteId)}
            type="button"
          >
            <Plus className="h-4 w-4" aria-hidden="true" />
            作品を追加
          </button>
          <button
            className="inline-flex h-9 items-center gap-1 rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300"
            disabled={selectedWorkId === null}
            onClick={() => selectedWorkId !== null && onAddPage(selectedWorkId)}
            type="button"
          >
            <FilePlus2 className="h-4 w-4" aria-hidden="true" />
            ページを追加
          </button>
          <button
            className="inline-flex h-9 items-center rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300"
            disabled={selectedWorkId === null || selectedSiteId === null}
            onClick={() => selectedWorkId !== null && selectedSiteId !== null && onBulkFetch(selectedWorkId, selectedSiteId)}
            type="button"
          >
            一括取得
          </button>
        </div>
      </div>
    </main>
  );
}

function PageReader({
  pageId,
  readerFontSize,
  onFetch,
}: {
  pageId: number;
  readerFontSize: ReaderFontSize;
  onFetch: (page: Page) => void;
}) {
  const { data: page, isLoading } = usePage(pageId);
  const favoriteCheck = useFavoriteCheck(pageId);
  const favoriteAdd = useFavoriteAdd(pageId);
  const favoriteRemove = useFavoriteRemove(pageId);
  const readerClass = {
    small: 'text-sm leading-7',
    medium: 'text-base leading-8',
    large: 'text-lg leading-9',
  }[readerFontSize];

  if (isLoading) {
    return <main className="min-w-0 flex-1 bg-white p-8 text-sm text-zinc-500">読み込み中...</main>;
  }

  return (
    <main className="min-w-0 flex-1 overflow-y-auto bg-white">
      <article className="mx-auto max-w-3xl px-8 py-8">
        <div className="flex items-start gap-3">
          <BookOpen className="mt-1 h-5 w-5 text-zinc-500" aria-hidden="true" />
          <div className="min-w-0 flex-1">
            <h1 className="text-2xl font-semibold">{page?.title || '無題のページ'}</h1>
            <p className="mt-1 text-sm text-zinc-500">状態: {page?.fetchStatus}</p>
            {page?.sourceType === 'wayback' && (
              <p className="mt-1 text-xs text-zinc-500">
                Wayback: {page.archivedAt ?? '-'} / {page.canonicalUrl ?? '-'}
              </p>
            )}
          </div>
          {page && (
            <div className="flex items-center gap-2">
              <button
                className="inline-flex h-9 w-9 items-center justify-center rounded-md border border-zinc-300 text-zinc-600 hover:bg-zinc-50"
                onClick={() => {
                  if (favoriteCheck.data?.isFavorite) {
                    favoriteRemove.mutate();
                  } else {
                    favoriteAdd.mutate();
                  }
                }}
                title="お気に入り"
                type="button"
              >
                <Star
                  className={`h-4 w-4 ${favoriteCheck.data?.isFavorite ? 'fill-yellow-400 text-yellow-500' : ''}`}
                  aria-hidden="true"
                />
              </button>
              <button
                className="inline-flex h-9 items-center rounded-md bg-blue-600 px-3 text-sm font-medium text-white"
                onClick={() => onFetch(page)}
                type="button"
              >
                URL取得
              </button>
            </div>
          )}
        </div>
        <div className={`mt-8 whitespace-pre-wrap text-zinc-800 ${readerClass}`}>
          {page?.contentText || '本文はまだ登録されていません。'}
        </div>
      </article>
    </main>
  );
}

function SettingsView({
  selectedPageId,
  readerFontSize,
  onFontSizeChange,
  onClose,
}: {
  selectedPageId: number | null;
  readerFontSize: ReaderFontSize;
  onFontSizeChange: (value: ReaderFontSize) => void;
  onClose: () => void;
}) {
  const exportPage = useExportPageText();
  const backup = useBackupDatabase();
  const duplicateUrls = useDuplicateSourceUrlList();

  return (
    <main className="min-w-0 flex-1 overflow-y-auto bg-white px-8 py-8">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">設定</h1>
        <button className="h-9 rounded-md border border-zinc-300 px-3 text-sm" onClick={onClose} type="button">
          閉じる
        </button>
      </div>

      <div className="mt-6 grid max-w-2xl gap-6">
        <section className="grid gap-3 border-b border-zinc-200 pb-6">
          <h2 className="text-sm font-semibold text-zinc-700">リーダー</h2>
          <div className="flex h-9 w-fit overflow-hidden rounded-md border border-zinc-300 bg-white text-sm">
            {(['small', 'medium', 'large'] as ReaderFontSize[]).map((value) => (
              <button
                key={value}
                className={`border-r border-zinc-300 px-4 last:border-r-0 ${
                  readerFontSize === value ? 'bg-zinc-200 text-zinc-950' : 'text-zinc-500 hover:bg-zinc-100'
                }`}
                onClick={() => onFontSizeChange(value)}
                type="button"
              >
                {value === 'small' ? '小' : value === 'medium' ? '中' : '大'}
              </button>
            ))}
          </div>
        </section>

        <section className="grid gap-3 border-b border-zinc-200 pb-6">
          <h2 className="text-sm font-semibold text-zinc-700">エクスポート</h2>
          <div className="flex items-center gap-3">
            <button
              className="h-9 rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300"
              disabled={selectedPageId === null || exportPage.isPending}
              onClick={() => selectedPageId !== null && exportPage.mutate(selectedPageId)}
              type="button"
            >
              選択ページを書き出す
            </button>
          </div>
          {exportPage.data && <p className="break-all text-xs text-zinc-500">出力先: {exportPage.data.path}</p>}
          {exportPage.error && (
            <p className="rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
              {exportPage.error instanceof Error ? exportPage.error.message : 'エクスポートに失敗しました'}
            </p>
          )}
        </section>

        <section className="grid gap-3 border-b border-zinc-200 pb-6">
          <h2 className="text-sm font-semibold text-zinc-700">バックアップ</h2>
          <div>
            <button
              className="h-9 rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300"
              disabled={backup.isPending}
              onClick={() => backup.mutate()}
              type="button"
            >
              DBバックアップを作成
            </button>
          </div>
          {backup.data && <p className="break-all text-xs text-zinc-500">出力先: {backup.data.path}</p>}
          {backup.error && (
            <p className="rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
              {backup.error instanceof Error ? backup.error.message : 'バックアップに失敗しました'}
            </p>
          )}
        </section>

        <section className="grid gap-3">
          <h2 className="text-sm font-semibold text-zinc-700">診断</h2>
          <div>
            <button
              className="h-9 rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300"
              disabled={duplicateUrls.isPending}
              onClick={() => duplicateUrls.mutate()}
              type="button"
            >
              重複URLを確認
            </button>
          </div>
          {duplicateUrls.data?.length === 0 && (
            <p className="rounded-md bg-green-50 px-3 py-2 text-sm text-green-700">重複URLは見つかりませんでした。</p>
          )}
          {duplicateUrls.data && duplicateUrls.data.length > 0 && (
            <div className="grid gap-2">
              {duplicateUrls.data.map((group) => (
                <div
                  className="rounded-md border border-amber-200 bg-amber-50 px-3 py-2 text-sm text-amber-900"
                  key={`${group.workId}-${group.sourceType}-${group.sourceUrl}`}
                >
                  <p className="font-medium">
                    {group.siteName} / {group.workTitle} / {group.sourceType}
                  </p>
                  <p className="break-all text-xs">{group.sourceUrl}</p>
                  <p className="mt-1 text-xs">
                    対象ページ:{' '}
                    {group.pages.map((page) => `${page.pageTitle || '無題'}(#${page.pageId})`).join(', ')}
                  </p>
                </div>
              ))}
            </div>
          )}
          {duplicateUrls.error && (
            <p className="rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
              {duplicateUrls.error instanceof Error ? duplicateUrls.error.message : '重複URL診断に失敗しました'}
            </p>
          )}
        </section>
      </div>
    </main>
  );
}

function FavoritesView({
  onClose,
  onSelectPage,
}: {
  onClose: () => void;
  onSelectPage: (pageId: number, workId: number, siteId: number) => void;
}) {
  const { data, isLoading } = useFavorites();

  return (
    <main className="min-w-0 flex-1 overflow-y-auto bg-white px-8 py-8">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">お気に入り</h1>
        <button className="h-9 rounded-md border border-zinc-300 px-3 text-sm" onClick={onClose} type="button">
          閉じる
        </button>
      </div>
      {isLoading && <p className="mt-6 text-sm text-zinc-500">読み込み中...</p>}
      {data?.groups.length === 0 && <p className="mt-6 text-sm text-zinc-500">お気に入りは未登録です。</p>}
      <div className="mt-6 grid gap-6">
        {data?.groups.map((siteGroup) => (
          <section key={siteGroup.siteId}>
            <h2 className="text-base font-semibold">{siteGroup.siteName}</h2>
            <div className="mt-3 grid gap-4">
              {siteGroup.works.map((workGroup) => (
                <div key={workGroup.workId}>
                  <h3 className="text-sm font-medium text-zinc-700">{workGroup.workTitle}</h3>
                  <div className="mt-2 grid gap-2">
                    {workGroup.pages.map((page) => (
                      <button
                        className="rounded-md border border-zinc-200 px-3 py-2 text-left text-sm hover:bg-zinc-50"
                        key={page.favoriteId}
                        onClick={() => onSelectPage(page.pageId, page.workId, page.siteId)}
                        type="button"
                      >
                        {page.pageTitle || '無題のページ'}
                      </button>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          </section>
        ))}
      </div>
    </main>
  );
}

function SearchResults({
  query,
  searchType,
  onSelectSite,
  onSelectWork,
  onSelectPage,
}: {
  query: string;
  searchType: 'title' | 'fulltext';
  onSelectSite: (siteId: number) => void;
  onSelectWork: (workId: number, siteId: number) => void;
  onSelectPage: (pageId: number, workId: number, siteId: number) => void;
}) {
  const titleSearch = useTitleSearch(query, searchType === 'title');
  const fullTextSearch = useFullTextSearch(query, searchType === 'fulltext');
  const isLoading = titleSearch.isLoading || fullTextSearch.isLoading;

  return (
    <main className="min-w-0 flex-1 overflow-y-auto bg-white px-8 py-8">
      <h1 className="text-xl font-semibold">検索結果</h1>
      <p className="mt-1 text-sm text-zinc-500">検索語: {query}</p>
      {isLoading && <p className="mt-6 text-sm text-zinc-500">検索中...</p>}
      {searchType === 'title' && titleSearch.data && (
        <div className="mt-6 grid gap-6">
          <ResultSection title={`サイト (${titleSearch.data.sites.length})`}>
            {titleSearch.data.sites.map((site) => (
              <ResultButton key={site.id} onClick={() => onSelectSite(site.id)} title={site.name} detail={site.baseUrl} />
            ))}
          </ResultSection>
          <ResultSection title={`作品 (${titleSearch.data.works.length})`}>
            {titleSearch.data.works.map((work) => (
              <ResultButton key={work.id} onClick={() => onSelectWork(work.id, work.siteId)} title={work.title} detail={work.siteName} />
            ))}
          </ResultSection>
          <ResultSection title={`ページ (${titleSearch.data.pages.length})`}>
            {titleSearch.data.pages.map((page) => (
              <ResultButton
                key={page.id}
                onClick={() => onSelectPage(page.id, page.workId, page.siteId)}
                title={page.title || '無題のページ'}
                detail={`${page.workTitle} / ${page.siteName}`}
              />
            ))}
          </ResultSection>
        </div>
      )}
      {searchType === 'fulltext' && fullTextSearch.data && (
        <div className="mt-6 grid gap-3">
          {fullTextSearch.data.map((item) => (
            <button
              className="rounded-md border border-zinc-200 px-4 py-3 text-left hover:bg-zinc-50"
              key={item.pageId}
              onClick={() => onSelectPage(item.pageId, item.workId, item.siteId)}
              type="button"
            >
              <span className="block text-sm font-medium">{item.pageTitle || '無題のページ'}</span>
              <span className="mt-1 block text-xs text-zinc-500">
                {item.workTitle} / {item.siteName}
              </span>
              <span
                className="mt-3 block text-sm leading-6 text-zinc-700 [&_mark]:rounded-sm [&_mark]:bg-yellow-200 [&_mark]:px-1"
                dangerouslySetInnerHTML={{ __html: item.snippet }}
              />
            </button>
          ))}
          {fullTextSearch.data.length === 0 && <p className="text-sm text-zinc-500">全文検索の結果はありません。</p>}
        </div>
      )}
    </main>
  );
}

function ResultSection({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section>
      <h2 className="text-sm font-semibold text-zinc-700">{title}</h2>
      <div className="mt-2 grid gap-2">{children}</div>
    </section>
  );
}

function ResultButton({ title, detail, onClick }: { title: string; detail: string; onClick: () => void }) {
  return (
    <button className="rounded-md border border-zinc-200 px-3 py-2 text-left hover:bg-zinc-50" onClick={onClick} type="button">
      <span className="block text-sm font-medium">{title}</span>
      <span className="mt-1 block text-xs text-zinc-500">{detail}</span>
    </button>
  );
}
