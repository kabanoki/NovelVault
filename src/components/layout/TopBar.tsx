import { Plus, Search, Star, Settings } from 'lucide-react';

interface TopBarProps {
  searchQuery: string;
  onSearchChange: (query: string) => void;
  searchType: 'title' | 'fulltext';
  onSearchTypeChange: (type: 'title' | 'fulltext') => void;
  onAddSite: () => void;
  onOpenProfiles: () => void;
  onOpenFavorites: () => void;
  onOpenSettings: () => void;
}

export function TopBar({ searchQuery, onSearchChange, searchType, onSearchTypeChange, onAddSite, onOpenProfiles, onOpenFavorites, onOpenSettings }: TopBarProps) {
  return (
    <header className="flex h-14 items-center gap-3 border-b border-zinc-200 bg-zinc-50 px-3">
      <button
        className="inline-flex h-9 items-center gap-1 rounded-md bg-blue-600 px-3 text-sm font-medium text-white"
        onClick={onAddSite}
        type="button"
      >
        <Plus className="h-4 w-4" aria-hidden="true" />
        サイト追加
      </button>

      <div className="ml-auto flex items-center gap-2">
        <label className="flex h-9 items-center gap-2 rounded-md border border-zinc-300 bg-white px-3 text-sm">
          <Search className="h-4 w-4 text-zinc-400" aria-hidden="true" />
          <input
            className="w-56 border-0 bg-transparent outline-none"
            value={searchQuery}
            onChange={(event) => onSearchChange(event.target.value)}
            placeholder={searchType === 'title' ? 'タイトル検索' : '全文検索'}
          />
        </label>
        <div className="flex h-9 overflow-hidden rounded-md border border-zinc-300 bg-white text-sm">
          <button
            className={`px-3 ${searchType === 'title' ? 'bg-zinc-200 text-zinc-950' : 'text-zinc-500 hover:bg-zinc-100'}`}
            onClick={() => onSearchTypeChange('title')}
            type="button"
          >
            タイトル
          </button>
          <button
            className={`border-l border-zinc-300 px-3 ${searchType === 'fulltext' ? 'bg-zinc-200 text-zinc-950' : 'text-zinc-500 hover:bg-zinc-100'}`}
            onClick={() => onSearchTypeChange('fulltext')}
            type="button"
          >
            全文
          </button>
        </div>
        <button
          className="inline-flex h-9 w-9 items-center justify-center rounded-md text-zinc-600 hover:bg-zinc-200"
          onClick={onOpenFavorites}
          title="お気に入り"
          type="button"
        >
          <Star className="h-4 w-4" aria-hidden="true" />
        </button>
        <button
          className="inline-flex h-9 items-center rounded-md px-3 text-sm text-zinc-600 hover:bg-zinc-200"
          onClick={onOpenProfiles}
          type="button"
        >
          プロファイル
        </button>
        <button
          className="inline-flex h-9 w-9 items-center justify-center rounded-md text-zinc-600 hover:bg-zinc-200"
          onClick={onOpenSettings}
          title="設定"
          type="button"
        >
          <Settings className="h-4 w-4" aria-hidden="true" />
        </button>
      </div>
    </header>
  );
}
