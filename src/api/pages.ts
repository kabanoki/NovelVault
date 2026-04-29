import { command } from './client';
import type { Page } from '../types/entities';

export interface PageCreateArgs {
  workId: number;
  pageNumber?: number | null;
  title?: string | null;
  sourceUrl?: string | null;
  sourceType: 'normal' | 'wayback' | 'local';
  requestedEncoding?: string | null;
  contentText?: string | null;
}

export interface PageUpdateArgs extends PageCreateArgs {
  id: number;
}

export function pageListByWork(workId: number): Promise<Page[]> {
  return command<Page[]>('page_list_by_work', { args: { workId } });
}

export function pageGet(id: number): Promise<Page> {
  return command<Page>('page_get', { args: { id } });
}

export function pageCreate(args: PageCreateArgs): Promise<Page> {
  return command<Page>('page_create', { args });
}

export function pageUpdate(args: PageUpdateArgs): Promise<Page> {
  return command<Page>('page_update', { args });
}

export function pageDelete(id: number): Promise<void> {
  return command<void>('page_delete', { args: { id } });
}
