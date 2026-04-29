import { ReactNode, useState } from 'react';
import { useBulkFetch } from '../../hooks/useBulkFetch';
import { useFetchPage } from '../../hooks/useFetchPage';
import { useCreatePage, useDeletePage, useUpdatePage } from '../../hooks/usePages';
import {
  useCreateSiteProfile,
  useDeleteSiteProfile,
  useSiteProfiles,
  useUpdateSiteProfile,
} from '../../hooks/useSiteProfiles';
import { useCreateSite, useDeleteSite, useUpdateSite } from '../../hooks/useSites';
import { useCreateWork, useDeleteWork, useUpdateWork } from '../../hooks/useWorks';
import type { Page, Site, SiteProfile, Work } from '../../types/entities';

export type EditorState =
  | { type: 'site-create' }
  | { type: 'work-create'; siteId: number }
  | { type: 'page-create'; workId: number }
  | { type: 'fetch'; page: Page }
  | { type: 'bulk-fetch'; workId: number; siteId: number }
  | { type: 'profiles'; siteId: number | null }
  | { type: 'site'; site: Site }
  | { type: 'work'; work: Work }
  | { type: 'page'; page: Page }
  | null;

export type ConfirmState =
  | { type: 'site'; site: Site }
  | { type: 'work'; work: Work }
  | { type: 'page'; page: Page }
  | null;

export function EditorModal({ editor, onClose }: { editor: EditorState; onClose: () => void }) {
  if (!editor) return null;

  if (editor.type === 'site-create') {
    return <SiteCreateModal onClose={onClose} />;
  }
  if (editor.type === 'work-create') {
    return <WorkCreateModal siteId={editor.siteId} onClose={onClose} />;
  }
  if (editor.type === 'page-create') {
    return <PageCreateModal workId={editor.workId} onClose={onClose} />;
  }
  if (editor.type === 'fetch') {
    return <FetchUrlModal page={editor.page} onClose={onClose} />;
  }
  if (editor.type === 'bulk-fetch') {
    return <BulkFetchModal workId={editor.workId} siteId={editor.siteId} onClose={onClose} />;
  }
  if (editor.type === 'profiles') {
    return <ProfileManagerModal siteId={editor.siteId} onClose={onClose} />;
  }
  if (editor.type === 'site') {
    return <SiteEditor site={editor.site} onClose={onClose} />;
  }
  if (editor.type === 'work') {
    return <WorkEditor work={editor.work} onClose={onClose} />;
  }
  return <PageEditor page={editor.page} onClose={onClose} />;
}

function SiteCreateModal({ onClose }: { onClose: () => void }) {
  const [name, setName] = useState('');
  const [baseUrl, setBaseUrl] = useState('');
  const createSite = useCreateSite();
  const canSave = name.trim().length > 0 && baseUrl.trim().length > 0;

  return (
    <Modal title="サイト追加" onClose={onClose}>
      <form
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (!canSave) return;
          createSite.mutate({ name, baseUrl }, { onSuccess: onClose });
        }}
      >
        <TextInput label="サイト名" value={name} onChange={setName} />
        <TextInput label="base_url" value={baseUrl} onChange={setBaseUrl} />
        <ModalActions disabled={!canSave || createSite.isPending} onCancel={onClose} />
      </form>
    </Modal>
  );
}

function WorkCreateModal({ siteId, onClose }: { siteId: number; onClose: () => void }) {
  const [title, setTitle] = useState('');
  const [authorName, setAuthorName] = useState('');
  const [sourceUrl, setSourceUrl] = useState('');
  const [description, setDescription] = useState('');
  const [siteProfileId, setSiteProfileId] = useState<number | null>(null);
  const { data: profiles } = useSiteProfiles(siteId);
  const createWork = useCreateWork(siteId);
  const canSave = title.trim().length > 0;

  return (
    <Modal title="作品追加" onClose={onClose}>
      <form
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (!canSave) return;
          createWork.mutate(
            { siteId, siteProfileId, title, authorName, sourceUrl, description },
            { onSuccess: onClose },
          );
        }}
      >
        <TextInput label="作品タイトル" value={title} onChange={setTitle} />
        <ProfileSelect profiles={profiles ?? []} value={siteProfileId} onChange={setSiteProfileId} />
        <TextInput label="著者名" value={authorName} onChange={setAuthorName} />
        <TextInput label="取得元URL" value={sourceUrl} onChange={setSourceUrl} />
        <TextArea label="説明" value={description} onChange={setDescription} />
        <ModalActions disabled={!canSave || createWork.isPending} onCancel={onClose} />
      </form>
    </Modal>
  );
}

function PageCreateModal({ workId, onClose }: { workId: number; onClose: () => void }) {
  const [title, setTitle] = useState('');
  const [pageNumber, setPageNumber] = useState('');
  const [sourceUrl, setSourceUrl] = useState('');
  const [sourceType, setSourceType] = useState<Page['sourceType']>('normal');
  const [requestedEncoding, setRequestedEncoding] = useState('utf-8');
  const [contentText, setContentText] = useState('');
  const createPage = useCreatePage(workId);

  return (
    <Modal title="ページ追加" onClose={onClose}>
      <form
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          createPage.mutate(
            {
              workId,
              title,
              pageNumber: pageNumber ? Number(pageNumber) : null,
              sourceUrl,
              sourceType,
              requestedEncoding,
              contentText,
            },
            { onSuccess: onClose },
          );
        }}
      >
        <TextInput label="ページタイトル" value={title} onChange={setTitle} />
        <TextInput label="章番号" value={pageNumber} onChange={setPageNumber} type="number" />
        <TextInput label="取得元URL" value={sourceUrl} onChange={setSourceUrl} />
        <SourceTypeSelect value={sourceType} onChange={setSourceType} />
        <EncodingSelect value={requestedEncoding} onChange={setRequestedEncoding} />
        <TextArea label="本文" value={contentText} onChange={setContentText} rows={8} />
        <ModalActions disabled={createPage.isPending} onCancel={onClose} />
      </form>
    </Modal>
  );
}

function FetchUrlModal({ page, onClose }: { page: Page; onClose: () => void }) {
  const [url, setUrl] = useState(page.sourceUrl ?? '');
  const [sourceType, setSourceType] = useState<Page['sourceType']>(page.sourceType);
  const [encoding, setEncoding] = useState<'auto' | 'utf-8' | 'shift_jis' | 'euc-jp'>(
    page.requestedEncoding === 'auto' || page.requestedEncoding === 'shift_jis' || page.requestedEncoding === 'euc-jp'
      ? page.requestedEncoding
      : 'utf-8',
  );
  const [titleSelector, setTitleSelector] = useState('h1');
  const [contentSelector, setContentSelector] = useState('body');
  const [removeSelectors, setRemoveSelectors] = useState('script\nstyle\nnav\nfooter');
  const fetchPage = useFetchPage(page.workId);
  const canFetch = url.trim().length > 0 && titleSelector.trim().length > 0 && contentSelector.trim().length > 0;

  return (
    <Modal title="URL取得" onClose={onClose}>
      <form
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (!canFetch) return;
          fetchPage.mutate(
            {
              pageId: page.id,
              url,
              sourceType,
              titleSelector,
              contentSelector,
              removeSelectors: removeSelectors.split('\n').map((value) => value.trim()).filter(Boolean),
              encoding,
            },
            { onSuccess: onClose },
          );
        }}
      >
        <TextInput label="URL" value={url} onChange={setUrl} />
        <SourceTypeSelect value={sourceType} onChange={setSourceType} />
        <EncodingSelect value={encoding} onChange={setEncoding} />
        <TextInput label="タイトル用セレクタ" value={titleSelector} onChange={setTitleSelector} />
        <TextInput label="本文用セレクタ" value={contentSelector} onChange={setContentSelector} />
        <TextArea label="除外セレクタ" value={removeSelectors} onChange={setRemoveSelectors} rows={4} />
        {fetchPage.error && (
          <p className="rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
            {fetchPage.error instanceof Error ? fetchPage.error.message : '取得に失敗しました'}
          </p>
        )}
        <ModalActions disabled={!canFetch || fetchPage.isPending} onCancel={onClose} submitLabel="取得実行" />
      </form>
    </Modal>
  );
}

function BulkFetchModal({ workId, siteId, onClose }: { workId: number; siteId: number; onClose: () => void }) {
  const [siteProfileId, setSiteProfileId] = useState<number | null>(null);
  const { data: profiles } = useSiteProfiles(siteId);
  const bulkFetch = useBulkFetch(workId);
  const result = bulkFetch.data;

  return (
    <Modal title="一括取得" onClose={onClose}>
      <form
        className="grid gap-4"
        onSubmit={(event) => {
          event.preventDefault();
          if (siteProfileId === null) return;
          bulkFetch.mutate(siteProfileId);
        }}
      >
        <ProfileSelect profiles={profiles ?? []} value={siteProfileId} onChange={setSiteProfileId} />
        {bulkFetch.error && (
          <p className="rounded-md bg-red-50 px-3 py-2 text-sm text-red-700">
            {bulkFetch.error instanceof Error ? bulkFetch.error.message : '一括取得に失敗しました'}
          </p>
        )}
        {result && (
          <p className="rounded-md bg-zinc-50 px-3 py-2 text-sm text-zinc-700">
            作成 {result.createdCount} 件 / 成功 {result.successCount} 件 / 失敗 {result.failedCount} 件
          </p>
        )}
        <ModalActions disabled={siteProfileId === null || bulkFetch.isPending} onCancel={onClose} submitLabel="取得実行" />
      </form>
    </Modal>
  );
}

function ProfileManagerModal({ siteId, onClose }: { siteId: number | null; onClose: () => void }) {
  const [editing, setEditing] = useState<SiteProfile | null>(null);
  const [name, setName] = useState('');
  const [profileJson, setProfileJson] = useState(defaultProfileJson());
  const { data: profiles } = useSiteProfiles(siteId);
  const createProfile = useCreateSiteProfile(siteId);
  const updateProfile = useUpdateSiteProfile(siteId);
  const deleteProfile = useDeleteSiteProfile(siteId);
  const canSave = siteId !== null && name.trim().length > 0 && profileJson.trim().length > 0;

  function startEdit(profile: SiteProfile) {
    setEditing(profile);
    setName(profile.name);
    setProfileJson(profile.profileJson);
  }

  function resetForm() {
    setEditing(null);
    setName('');
    setProfileJson(defaultProfileJson());
  }

  return (
    <Modal title="サイトプロファイル" onClose={onClose}>
      {siteId === null ? (
        <p className="text-sm text-zinc-600">サイドバーでサイトを選択してください。</p>
      ) : (
        <div className="grid gap-4">
          <div className="grid gap-2">
            {(profiles ?? []).map((profile) => (
              <div className="flex items-center gap-2 rounded-md border border-zinc-200 px-3 py-2" key={profile.id}>
                <button className="min-w-0 flex-1 truncate text-left text-sm" onClick={() => startEdit(profile)} type="button">
                  {profile.name}
                </button>
                <button className="text-sm text-zinc-500 hover:text-zinc-900" onClick={() => startEdit(profile)} type="button">
                  編集
                </button>
                <button className="text-sm text-red-600 hover:text-red-700" onClick={() => deleteProfile.mutate(profile.id)} type="button">
                  削除
                </button>
              </div>
            ))}
            {profiles?.length === 0 && <p className="text-sm text-zinc-500">プロファイルは未登録です。</p>}
          </div>
          <form
            className="grid gap-3 border-t border-zinc-200 pt-4"
            onSubmit={(event) => {
              event.preventDefault();
              if (!canSave || siteId === null) return;
              if (editing) {
                updateProfile.mutate({ id: editing.id, name, profileJson }, { onSuccess: resetForm });
              } else {
                createProfile.mutate({ siteId, name, profileJson }, { onSuccess: resetForm });
              }
            }}
          >
            <TextInput label="プロファイル名" value={name} onChange={setName} />
            <TextArea label="プロファイルJSON" value={profileJson} onChange={setProfileJson} rows={12} />
            <div className="flex justify-end gap-2">
              {editing && (
                <button className="h-9 rounded-md border border-zinc-300 px-3 text-sm" onClick={resetForm} type="button">
                  新規作成に戻る
                </button>
              )}
              <button
                className="h-9 rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300"
                disabled={!canSave || createProfile.isPending || updateProfile.isPending}
                type="submit"
              >
                {editing ? '更新' : '追加'}
              </button>
            </div>
          </form>
        </div>
      )}
    </Modal>
  );
}

function SiteEditor({ site, onClose }: { site: Site; onClose: () => void }) {
  const [name, setName] = useState(site.name);
  const [baseUrl, setBaseUrl] = useState(site.baseUrl);
  const updateSite = useUpdateSite();
  const canSave = name.trim().length > 0 && baseUrl.trim().length > 0;

  return (
    <Modal title="サイト編集" onClose={onClose}>
      <form
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (!canSave) return;
          updateSite.mutate({ id: site.id, name, baseUrl }, { onSuccess: onClose });
        }}
      >
        <TextInput label="サイト名" value={name} onChange={setName} />
        <TextInput label="base_url" value={baseUrl} onChange={setBaseUrl} />
        <ModalActions disabled={!canSave || updateSite.isPending} onCancel={onClose} />
      </form>
    </Modal>
  );
}

function WorkEditor({ work, onClose }: { work: Work; onClose: () => void }) {
  const [title, setTitle] = useState(work.title);
  const [siteProfileId, setSiteProfileId] = useState<number | null>(work.siteProfileId);
  const [authorName, setAuthorName] = useState(work.authorName ?? '');
  const [sourceUrl, setSourceUrl] = useState(work.sourceUrl ?? '');
  const [description, setDescription] = useState(work.description ?? '');
  const updateWork = useUpdateWork(work.siteId);
  const { data: profiles } = useSiteProfiles(work.siteId);
  const canSave = title.trim().length > 0;

  return (
    <Modal title="作品編集" onClose={onClose}>
      <form
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          if (!canSave) return;
          updateWork.mutate(
            { id: work.id, siteProfileId, title, authorName, sourceUrl, description },
            { onSuccess: onClose },
          );
        }}
      >
        <TextInput label="作品タイトル" value={title} onChange={setTitle} />
        <ProfileSelect profiles={profiles ?? []} value={siteProfileId} onChange={setSiteProfileId} />
        <TextInput label="著者名" value={authorName} onChange={setAuthorName} />
        <TextInput label="取得元URL" value={sourceUrl} onChange={setSourceUrl} />
        <TextArea label="説明" value={description} onChange={setDescription} />
        <ModalActions disabled={!canSave || updateWork.isPending} onCancel={onClose} />
      </form>
    </Modal>
  );
}

function PageEditor({ page, onClose }: { page: Page; onClose: () => void }) {
  const [title, setTitle] = useState(page.title ?? '');
  const [pageNumber, setPageNumber] = useState(page.pageNumber?.toString() ?? '');
  const [sourceUrl, setSourceUrl] = useState(page.sourceUrl ?? '');
  const [sourceType, setSourceType] = useState<Page['sourceType']>(page.sourceType);
  const [requestedEncoding, setRequestedEncoding] = useState(page.requestedEncoding ?? 'utf-8');
  const [contentText, setContentText] = useState(page.contentText ?? '');
  const updatePage = useUpdatePage(page.workId);

  return (
    <Modal title="ページ編集" onClose={onClose}>
      <form
        className="grid gap-3"
        onSubmit={(event) => {
          event.preventDefault();
          updatePage.mutate(
            {
              id: page.id,
              workId: page.workId,
              title,
              pageNumber: pageNumber ? Number(pageNumber) : null,
              sourceUrl,
              sourceType,
              requestedEncoding,
              contentText,
            },
            { onSuccess: onClose },
          );
        }}
      >
        <TextInput label="ページタイトル" value={title} onChange={setTitle} />
        <TextInput label="章番号" value={pageNumber} onChange={setPageNumber} type="number" />
        <TextInput label="取得元URL" value={sourceUrl} onChange={setSourceUrl} />
        <SourceTypeSelect value={sourceType} onChange={setSourceType} />
        <EncodingSelect value={requestedEncoding} onChange={setRequestedEncoding} />
        <TextArea label="本文" value={contentText} onChange={setContentText} rows={8} />
        <ModalActions disabled={updatePage.isPending} onCancel={onClose} />
      </form>
    </Modal>
  );
}

export function ConfirmDialog({
  confirm,
  onClose,
  onDeleted,
}: {
  confirm: ConfirmState;
  onClose: () => void;
  onDeleted: (deleted: { type: 'site' | 'work' | 'page'; id: number }) => void;
}) {
  const deleteSite = useDeleteSite();
  const deleteWork = useDeleteWork(confirm?.type === 'work' ? confirm.work.siteId : null);
  const deletePage = useDeletePage(confirm?.type === 'page' ? confirm.page.workId : null);

  if (!confirm) return null;

  const label =
    confirm.type === 'site'
      ? confirm.site.name
      : confirm.type === 'work'
        ? confirm.work.title
        : confirm.page.title || '無題のページ';
  const pending = deleteSite.isPending || deleteWork.isPending || deletePage.isPending;

  function handleDelete() {
    if (!confirm) return;
    if (confirm.type === 'site') {
      deleteSite.mutate(confirm.site.id, {
        onSuccess: () => {
          onDeleted({ type: 'site', id: confirm.site.id });
          onClose();
        },
      });
    } else if (confirm.type === 'work') {
      deleteWork.mutate(confirm.work.id, {
        onSuccess: () => {
          onDeleted({ type: 'work', id: confirm.work.id });
          onClose();
        },
      });
    } else {
      deletePage.mutate(confirm.page.id, {
        onSuccess: () => {
          onDeleted({ type: 'page', id: confirm.page.id });
          onClose();
        },
      });
    }
  }

  return (
    <Modal title="削除確認" onClose={onClose}>
      <div className="grid gap-4">
        <p className="text-sm leading-6 text-zinc-700">
          「{label}」を削除します。配下のデータも削除されます。
        </p>
        <div className="flex justify-end gap-2">
          <button className="h-9 rounded-md border border-zinc-300 px-3 text-sm" onClick={onClose} type="button">
            キャンセル
          </button>
          <button className="h-9 rounded-md bg-red-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300" disabled={pending} onClick={handleDelete} type="button">
            削除
          </button>
        </div>
      </div>
    </Modal>
  );
}

function Modal({ title, children, onClose }: { title: string; children: ReactNode; onClose: () => void }) {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30 px-4">
      <div className="w-full max-w-lg rounded-lg bg-white p-5 shadow-xl">
        <div className="mb-4 flex items-center justify-between">
          <h2 className="text-base font-semibold">{title}</h2>
          <button className="h-8 rounded-md px-2 text-sm text-zinc-500 hover:bg-zinc-100" onClick={onClose} type="button">
            閉じる
          </button>
        </div>
        {children}
      </div>
    </div>
  );
}

function TextInput({
  label,
  value,
  onChange,
  type = 'text',
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: string;
}) {
  return (
    <label className="grid gap-1 text-sm">
      <span className="font-medium text-zinc-700">{label}</span>
      <input
        className="h-9 rounded-md border border-zinc-300 px-3"
        type={type}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  );
}

function TextArea({
  label,
  value,
  onChange,
  rows = 3,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  rows?: number;
}) {
  return (
    <label className="grid gap-1 text-sm">
      <span className="font-medium text-zinc-700">{label}</span>
      <textarea
        className="rounded-md border border-zinc-300 px-3 py-2"
        rows={rows}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  );
}

function SourceTypeSelect({
  value,
  onChange,
}: {
  value: Page['sourceType'];
  onChange: (value: Page['sourceType']) => void;
}) {
  return (
    <label className="grid gap-1 text-sm">
      <span className="font-medium text-zinc-700">URL種別</span>
      <select
        className="h-9 rounded-md border border-zinc-300 px-3"
        value={value}
        onChange={(event) => onChange(event.target.value as Page['sourceType'])}
      >
        <option value="normal">normal</option>
        <option value="wayback">wayback</option>
        <option value="local">local</option>
      </select>
    </label>
  );
}

function ProfileSelect({
  profiles,
  value,
  onChange,
}: {
  profiles: SiteProfile[];
  value: number | null;
  onChange: (value: number | null) => void;
}) {
  return (
    <label className="grid gap-1 text-sm">
      <span className="font-medium text-zinc-700">プロファイル</span>
      <select
        className="h-9 rounded-md border border-zinc-300 px-3"
        value={value ?? ''}
        onChange={(event) => onChange(event.target.value ? Number(event.target.value) : null)}
      >
        <option value="">未選択</option>
        {profiles.map((profile) => (
          <option key={profile.id} value={profile.id}>
            {profile.name}
          </option>
        ))}
      </select>
    </label>
  );
}

function defaultProfileJson() {
  return JSON.stringify(
    {
      schema_version: 1,
      name: '',
      base_url: '',
      source_type: 'normal',
      encoding: 'auto',
      page_pattern: {
        title_selector: 'h1',
        content_selector: 'body',
        remove_selectors: ['script', 'style', 'nav', 'footer'],
      },
      fetch_options: {
        interval_ms: 1000,
        user_agent: 'NovelVault/0.1',
        timeout_sec: 30,
      },
    },
    null,
    2,
  );
}

function EncodingSelect({
  value,
  onChange,
}: {
  value: string;
  onChange: (value: 'auto' | 'utf-8' | 'shift_jis' | 'euc-jp') => void;
}) {
  const selected = value === 'auto' || value === 'shift_jis' || value === 'euc-jp' ? value : 'utf-8';

  return (
    <label className="grid gap-1 text-sm">
      <span className="font-medium text-zinc-700">文字コード</span>
      <select
        className="h-9 rounded-md border border-zinc-300 px-3"
        value={selected}
        onChange={(event) => onChange(event.target.value as 'auto' | 'utf-8' | 'shift_jis' | 'euc-jp')}
      >
        <option value="auto">auto</option>
        <option value="utf-8">utf-8</option>
        <option value="shift_jis">shift_jis</option>
        <option value="euc-jp">euc-jp</option>
      </select>
    </label>
  );
}

function ModalActions({
  disabled,
  onCancel,
  submitLabel = '保存',
}: {
  disabled: boolean;
  onCancel: () => void;
  submitLabel?: string;
}) {
  return (
    <div className="mt-2 flex justify-end gap-2">
      <button className="h-9 rounded-md border border-zinc-300 px-3 text-sm" onClick={onCancel} type="button">
        キャンセル
      </button>
      <button className="h-9 rounded-md bg-blue-600 px-3 text-sm font-medium text-white disabled:bg-zinc-300" disabled={disabled} type="submit">
        {submitLabel}
      </button>
    </div>
  );
}
