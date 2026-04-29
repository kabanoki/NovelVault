import { command } from './client';
import type { FullTextSearchItem, TitleSearchResults } from '../types/entities';

export function searchTitles(query: string): Promise<TitleSearchResults> {
  return command<TitleSearchResults>('search_titles', { args: { query } });
}

export function searchFullText(query: string): Promise<FullTextSearchItem[]> {
  return command<FullTextSearchItem[]>('search_full_text', { args: { query } });
}

export function rebuildSearchIndex(): Promise<void> {
  return command<void>('rebuild_search_index');
}
