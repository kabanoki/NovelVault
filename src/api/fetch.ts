import { command } from './client';
import type { Page } from '../types/entities';

export interface FetchPageByUrlArgs {
  pageId: number;
  url: string;
  sourceType: Page['sourceType'];
  titleSelector: string;
  contentSelector: string;
  removeSelectors: string[];
  encoding: 'auto' | 'utf-8' | 'shift_jis' | 'euc-jp';
}

export function fetchPageByUrl(args: FetchPageByUrlArgs): Promise<Page> {
  return command<Page>('fetch_page_by_url', { args });
}
