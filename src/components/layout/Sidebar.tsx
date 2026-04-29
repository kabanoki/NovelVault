import { ChevronDown, ChevronRight, FileText, Globe2, LibraryBig, Pencil, Trash2 } from 'lucide-react';
import { MouseEvent, useEffect, useState } from 'react';
import { useSites } from '../../hooks/useSites';
import { useWorks } from '../../hooks/useWorks';
import { usePages } from '../../hooks/usePages';
import type { Page, Site, Work } from '../../types/entities';

interface SidebarProps {
  selectedSiteId: number | null;
  selectedWorkId: number | null;
  selectedPageId: number | null;
  onSelectSite: (id: number) => void;
  onSelectWork: (id: number, siteId: number) => void;
  onSelectPage: (id: number) => void;
  onEditSite: (site: Site) => void;
  onDeleteSite: (site: Site) => void;
  onEditWork: (work: Work) => void;
  onDeleteWork: (work: Work) => void;
  onEditPage: (page: Page) => void;
  onDeletePage: (page: Page) => void;
}

export function Sidebar({
  selectedSiteId,
  selectedWorkId,
  selectedPageId,
  onSelectSite,
  onSelectWork,
  onSelectPage,
  onEditSite,
  onDeleteSite,
  onEditWork,
  onDeleteWork,
  onEditPage,
  onDeletePage,
}: SidebarProps) {
  const { data: sites, isLoading, error } = useSites();
  const [expandedSiteIds, setExpandedSiteIds] = useState<Set<number>>(new Set());
  const [expandedWorkIds, setExpandedWorkIds] = useState<Set<number>>(new Set());
  const [contextMenu, setContextMenu] = useState<ContextMenuState>(null);

  useEffect(() => {
    if (selectedSiteId === null) return;
    setExpandedSiteIds((prev) => {
      if (prev.has(selectedSiteId)) return prev;
      return new Set(prev).add(selectedSiteId);
    });
  }, [selectedSiteId]);

  useEffect(() => {
    if (selectedWorkId === null) return;
    setExpandedWorkIds((prev) => {
      if (prev.has(selectedWorkId)) return prev;
      return new Set(prev).add(selectedWorkId);
    });
  }, [selectedWorkId]);

  function toggleSite(site: Site) {
    setExpandedSiteIds((prev) => toggleSet(prev, site.id));
    onSelectSite(site.id);
  }

  function toggleWork(work: Work) {
    setExpandedWorkIds((prev) => toggleSet(prev, work.id));
    onSelectWork(work.id, work.siteId);
  }

  function openContextMenu(event: MouseEvent, target: ContextTarget) {
    event.preventDefault();
    setContextMenu({ x: event.clientX, y: event.clientY, target });
  }

  return (
    <aside
      className="relative w-72 shrink-0 overflow-y-auto border-r border-zinc-200 bg-zinc-50"
      onClick={() => setContextMenu(null)}
    >
      <div className="border-b border-zinc-200 px-4 py-3 text-xs font-semibold uppercase text-zinc-500">
        ライブラリ
      </div>

      {isLoading && <p className="px-4 py-3 text-sm text-zinc-500">読み込み中...</p>}
      {error && <p className="px-4 py-3 text-sm text-red-600">サイト一覧を取得できませんでした。</p>}
      {sites?.length === 0 && (
        <p className="px-4 py-4 text-sm leading-6 text-zinc-500">上部のフォームからサイトを追加してください。</p>
      )}

      <div className="py-2">
        {sites?.map((site) => (
          <SiteNode
            key={site.id}
            site={site}
            isExpanded={expandedSiteIds.has(site.id)}
            isSelected={selectedSiteId === site.id}
            selectedWorkId={selectedWorkId}
            selectedPageId={selectedPageId}
            expandedWorkIds={expandedWorkIds}
            onToggleSite={toggleSite}
            onToggleWork={toggleWork}
            onSelectPage={onSelectPage}
            onEditSite={onEditSite}
            onDeleteSite={onDeleteSite}
            onEditWork={onEditWork}
            onDeleteWork={onDeleteWork}
            onEditPage={onEditPage}
            onDeletePage={onDeletePage}
            onContextMenu={openContextMenu}
          />
        ))}
      </div>
      <ContextMenu
        menu={contextMenu}
        onClose={() => setContextMenu(null)}
        onEdit={(target) => {
          if (target.type === 'site') onEditSite(target.site);
          if (target.type === 'work') onEditWork(target.work);
          if (target.type === 'page') onEditPage(target.page);
        }}
        onDelete={(target) => {
          if (target.type === 'site') onDeleteSite(target.site);
          if (target.type === 'work') onDeleteWork(target.work);
          if (target.type === 'page') onDeletePage(target.page);
        }}
      />
    </aside>
  );
}

function SiteNode({
  site,
  isExpanded,
  isSelected,
  selectedWorkId,
  selectedPageId,
  expandedWorkIds,
  onToggleSite,
  onToggleWork,
  onSelectPage,
  onEditSite,
  onDeleteSite,
  onEditWork,
  onDeleteWork,
  onEditPage,
  onDeletePage,
  onContextMenu,
}: {
  site: Site;
  isExpanded: boolean;
  isSelected: boolean;
  selectedWorkId: number | null;
  selectedPageId: number | null;
  expandedWorkIds: Set<number>;
  onToggleSite: (site: Site) => void;
  onToggleWork: (work: Work) => void;
  onSelectPage: (id: number) => void;
  onEditSite: (site: Site) => void;
  onDeleteSite: (site: Site) => void;
  onEditWork: (work: Work) => void;
  onDeleteWork: (work: Work) => void;
  onEditPage: (page: Page) => void;
  onDeletePage: (page: Page) => void;
  onContextMenu: (event: MouseEvent, target: ContextTarget) => void;
}) {
  const { data: works } = useWorks(isExpanded ? site.id : null);

  return (
    <div>
      <div
        className={`group flex items-center hover:bg-zinc-200 ${isSelected ? 'bg-blue-50 text-blue-700' : ''}`}
        onContextMenu={(event) => onContextMenu(event, { type: 'site', site })}
      >
        <button className="flex min-w-0 flex-1 items-center gap-2 px-3 py-2 text-left text-sm" onClick={() => onToggleSite(site)} type="button">
          {isExpanded ? <ChevronDown className="h-4 w-4 shrink-0 text-zinc-400" /> : <ChevronRight className="h-4 w-4 shrink-0 text-zinc-400" />}
          <Globe2 className="h-4 w-4 shrink-0 text-zinc-500" aria-hidden="true" />
          <span className="min-w-0 truncate">{site.name}</span>
        </button>
        <RowActions onEdit={() => onEditSite(site)} onDelete={() => onDeleteSite(site)} />
      </div>
      {isExpanded && (
        <div>
          {works?.length === 0 && <p className="px-8 py-2 text-xs text-zinc-500">作品未登録</p>}
          {works?.map((work) => (
            <WorkNode
              key={work.id}
              work={work}
              isExpanded={expandedWorkIds.has(work.id)}
              isSelected={selectedWorkId === work.id}
              selectedPageId={selectedPageId}
              onToggleWork={onToggleWork}
              onSelectPage={onSelectPage}
              onEditWork={onEditWork}
              onDeleteWork={onDeleteWork}
              onEditPage={onEditPage}
              onDeletePage={onDeletePage}
              onContextMenu={onContextMenu}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function WorkNode({
  work,
  isExpanded,
  isSelected,
  selectedPageId,
  onToggleWork,
  onSelectPage,
  onEditWork,
  onDeleteWork,
  onEditPage,
  onDeletePage,
  onContextMenu,
}: {
  work: Work;
  isExpanded: boolean;
  isSelected: boolean;
  selectedPageId: number | null;
  onToggleWork: (work: Work) => void;
  onSelectPage: (id: number) => void;
  onEditWork: (work: Work) => void;
  onDeleteWork: (work: Work) => void;
  onEditPage: (page: Page) => void;
  onDeletePage: (page: Page) => void;
  onContextMenu: (event: MouseEvent, target: ContextTarget) => void;
}) {
  const { data: pages } = usePages(isExpanded ? work.id : null);

  return (
    <div>
      <div
        className={`group flex items-center hover:bg-zinc-200 ${isSelected ? 'bg-blue-50 text-blue-700' : ''}`}
        onContextMenu={(event) => onContextMenu(event, { type: 'work', work })}
      >
        <button className="flex min-w-0 flex-1 items-center gap-2 py-2 pl-8 pr-3 text-left text-sm" onClick={() => onToggleWork(work)} type="button">
          {isExpanded ? <ChevronDown className="h-4 w-4 shrink-0 text-zinc-400" /> : <ChevronRight className="h-4 w-4 shrink-0 text-zinc-400" />}
          <LibraryBig className="h-4 w-4 shrink-0 text-zinc-500" aria-hidden="true" />
          <span className="min-w-0 truncate">{work.title}</span>
        </button>
        <RowActions onEdit={() => onEditWork(work)} onDelete={() => onDeleteWork(work)} />
      </div>
      {isExpanded && (
        <div>
          {pages?.length === 0 && <p className="px-12 py-2 text-xs text-zinc-500">ページ未登録</p>}
          {pages?.map((page) => (
            <div
              className={`group flex items-center hover:bg-zinc-200 ${selectedPageId === page.id ? 'bg-blue-100 text-blue-800' : ''}`}
              key={page.id}
              onContextMenu={(event) => onContextMenu(event, { type: 'page', page })}
            >
              <button className="flex min-w-0 flex-1 items-center gap-2 py-2 pl-14 pr-3 text-left text-sm" onClick={() => onSelectPage(page.id)} type="button">
                <FileText className="h-4 w-4 shrink-0 text-zinc-500" aria-hidden="true" />
                <span className="min-w-0 truncate">{page.title || '無題のページ'}</span>
              </button>
              <RowActions onEdit={() => onEditPage(page)} onDelete={() => onDeletePage(page)} />
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

type ContextTarget =
  | { type: 'site'; site: Site }
  | { type: 'work'; work: Work }
  | { type: 'page'; page: Page };

type ContextMenuState = {
  x: number;
  y: number;
  target: ContextTarget;
} | null;

function ContextMenu({
  menu,
  onClose,
  onEdit,
  onDelete,
}: {
  menu: ContextMenuState;
  onClose: () => void;
  onEdit: (target: ContextTarget) => void;
  onDelete: (target: ContextTarget) => void;
}) {
  if (!menu) return null;

  return (
    <div
      className="fixed z-50 w-36 rounded-md border border-zinc-200 bg-white py-1 text-sm shadow-lg"
      style={{ left: menu.x, top: menu.y }}
      onClick={(event) => event.stopPropagation()}
    >
      <button
        className="flex w-full items-center gap-2 px-3 py-2 text-left hover:bg-zinc-100"
        onClick={() => {
          onEdit(menu.target);
          onClose();
        }}
        type="button"
      >
        <Pencil className="h-4 w-4 text-zinc-500" aria-hidden="true" />
        編集
      </button>
      <button
        className="flex w-full items-center gap-2 px-3 py-2 text-left text-red-600 hover:bg-red-50"
        onClick={() => {
          onDelete(menu.target);
          onClose();
        }}
        type="button"
      >
        <Trash2 className="h-4 w-4" aria-hidden="true" />
        削除
      </button>
    </div>
  );
}

function RowActions({ onEdit, onDelete }: { onEdit: () => void; onDelete: () => void }) {
  return (
    <div className="flex shrink-0 items-center gap-1 pr-2 opacity-0 group-hover:opacity-100">
      <button
        className="inline-flex h-7 w-7 items-center justify-center rounded-md text-zinc-500 hover:bg-white hover:text-zinc-800"
        onClick={(event) => {
          event.stopPropagation();
          onEdit();
        }}
        title="編集"
        type="button"
      >
        <Pencil className="h-3.5 w-3.5" aria-hidden="true" />
      </button>
      <button
        className="inline-flex h-7 w-7 items-center justify-center rounded-md text-zinc-500 hover:bg-white hover:text-red-600"
        onClick={(event) => {
          event.stopPropagation();
          onDelete();
        }}
        title="削除"
        type="button"
      >
        <Trash2 className="h-3.5 w-3.5" aria-hidden="true" />
      </button>
    </div>
  );
}

function toggleSet(values: Set<number>, id: number) {
  const next = new Set(values);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  return next;
}
