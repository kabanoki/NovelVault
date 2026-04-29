import { command } from './client';

export interface DuplicateSourceUrlPage {
  pageId: number;
  pageTitle: string | null;
  pageNumber: number | null;
}

export interface DuplicateSourceUrlGroup {
  siteId: number;
  siteName: string;
  workId: number;
  workTitle: string;
  sourceType: string;
  sourceUrl: string;
  pages: DuplicateSourceUrlPage[];
}

export function listDuplicateSourceUrls(): Promise<DuplicateSourceUrlGroup[]> {
  return command<DuplicateSourceUrlGroup[]>('duplicate_source_url_list');
}
