import { useMutation, useQueryClient } from '@tanstack/react-query';
import { fetchPageByUrl, type FetchPageByUrlArgs } from '../api/fetch';
import { pageKey, pagesByWorkKey } from './usePages';

export function useFetchPage(workId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: FetchPageByUrlArgs) => fetchPageByUrl(args),
    onSuccess: (page) => {
      queryClient.setQueryData(pageKey(page.id), page);
      queryClient.invalidateQueries({ queryKey: pagesByWorkKey(workId) });
    },
  });
}
