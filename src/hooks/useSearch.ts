import { useQuery } from '@tanstack/react-query';
import { searchFullText, searchTitles } from '../api/search';

export function useTitleSearch(query: string, enabled = true) {
  const trimmed = query.trim();

  return useQuery({
    queryKey: ['search', 'titles', trimmed],
    queryFn: () => searchTitles(trimmed),
    enabled: enabled && trimmed.length > 0,
  });
}

export function useFullTextSearch(query: string, enabled: boolean) {
  const trimmed = query.trim();

  return useQuery({
    queryKey: ['search', 'fulltext', trimmed],
    queryFn: () => searchFullText(trimmed),
    enabled: enabled && trimmed.length > 0,
  });
}
