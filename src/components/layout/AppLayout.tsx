import { useState } from 'react';
import { ConfirmDialog, ConfirmState, EditorModal, EditorState } from './AppModals';
import { MainPane } from './MainPane';
import { Sidebar } from './Sidebar';
import { TopBar } from './TopBar';
import { useReaderSettings } from '../../hooks/useReaderSettings';

export function AppLayout() {
  const [selectedSiteId, setSelectedSiteId] = useState<number | null>(null);
  const [selectedWorkId, setSelectedWorkId] = useState<number | null>(null);
  const [selectedPageId, setSelectedPageId] = useState<number | null>(null);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchType, setSearchType] = useState<'title' | 'fulltext'>('title');
  const [editor, setEditor] = useState<EditorState>(null);
  const [confirm, setConfirm] = useState<ConfirmState>(null);
  const [showFavorites, setShowFavorites] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const { readerFontSize, setReaderFontSize } = useReaderSettings();

  return (
    <div className="flex h-screen flex-col bg-zinc-100 text-zinc-950">
      <TopBar
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        searchType={searchType}
        onSearchTypeChange={setSearchType}
        onAddSite={() => setEditor({ type: 'site-create' })}
        onOpenProfiles={() => setEditor({ type: 'profiles', siteId: selectedSiteId })}
        onOpenFavorites={() => {
          setShowFavorites(true);
          setShowSettings(false);
          setSearchQuery('');
        }}
        onOpenSettings={() => {
          setShowSettings(true);
          setShowFavorites(false);
          setSearchQuery('');
        }}
      />
      <div className="flex min-h-0 flex-1">
        <Sidebar
          selectedPageId={selectedPageId}
          selectedSiteId={selectedSiteId}
          selectedWorkId={selectedWorkId}
          onSelectPage={setSelectedPageId}
          onSelectSite={(id) => {
            setSelectedSiteId(id);
            setSelectedWorkId(null);
            setSelectedPageId(null);
          }}
          onSelectWork={(id, siteId) => {
            setSelectedSiteId(siteId);
            setSelectedWorkId(id);
            setSelectedPageId(null);
          }}
          onEditSite={(site) => setEditor({ type: 'site', site })}
          onDeleteSite={(site) => setConfirm({ type: 'site', site })}
          onEditWork={(work) => setEditor({ type: 'work', work })}
          onDeleteWork={(work) => setConfirm({ type: 'work', work })}
          onEditPage={(page) => setEditor({ type: 'page', page })}
          onDeletePage={(page) => setConfirm({ type: 'page', page })}
        />
        <MainPane
          searchQuery={searchQuery}
          searchType={searchType}
          selectedPageId={selectedPageId}
          selectedSiteId={selectedSiteId}
          selectedWorkId={selectedWorkId}
          showFavorites={showFavorites}
          showSettings={showSettings}
          readerFontSize={readerFontSize}
          onAddPage={(workId) => setEditor({ type: 'page-create', workId })}
          onAddWork={(siteId) => setEditor({ type: 'work-create', siteId })}
          onBulkFetch={(workId, siteId) => setEditor({ type: 'bulk-fetch', workId, siteId })}
          onFetchPage={(page) => setEditor({ type: 'fetch', page })}
          onCloseFavorites={() => setShowFavorites(false)}
          onCloseSettings={() => setShowSettings(false)}
          onReaderFontSizeChange={setReaderFontSize}
          onClearSearch={() => setSearchQuery('')}
          onSelectPage={setSelectedPageId}
          onSelectSite={setSelectedSiteId}
          onSelectWork={setSelectedWorkId}
        />
      </div>
      <EditorModal editor={editor} onClose={() => setEditor(null)} />
      <ConfirmDialog
        confirm={confirm}
        onClose={() => setConfirm(null)}
        onDeleted={(deleted) => {
          if (deleted.type === 'site' && selectedSiteId === deleted.id) {
            setSelectedSiteId(null);
            setSelectedWorkId(null);
            setSelectedPageId(null);
          }
          if (deleted.type === 'work' && selectedWorkId === deleted.id) {
            setSelectedWorkId(null);
            setSelectedPageId(null);
          }
          if (deleted.type === 'page' && selectedPageId === deleted.id) {
            setSelectedPageId(null);
          }
        }}
      />
    </div>
  );
}
