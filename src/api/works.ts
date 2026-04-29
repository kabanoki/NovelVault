import { command } from './client';
import type { Work } from '../types/entities';

export interface WorkCreateArgs {
  siteId: number;
  siteProfileId?: number | null;
  title: string;
  authorName?: string | null;
  description?: string | null;
  sourceUrl?: string | null;
}

export interface WorkUpdateArgs {
  id: number;
  siteProfileId?: number | null;
  title: string;
  authorName?: string | null;
  description?: string | null;
  sourceUrl?: string | null;
}

export function workListBySite(siteId: number): Promise<Work[]> {
  return command<Work[]>('work_list_by_site', { args: { siteId } });
}

export function workCreate(args: WorkCreateArgs): Promise<Work> {
  return command<Work>('work_create', { args });
}

export function workUpdate(args: WorkUpdateArgs): Promise<Work> {
  return command<Work>('work_update', { args });
}

export function workDelete(id: number): Promise<void> {
  return command<void>('work_delete', { args: { id } });
}
