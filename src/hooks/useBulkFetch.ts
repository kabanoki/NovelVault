import { useMutation, useQueryClient } from '@tanstack/react-query';
import { bulkFetchByProfile } from '../api/bulkFetch';
import { pagesByWorkKey } from './usePages';

export function useBulkFetch(workId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (siteProfileId: number) => bulkFetchByProfile(workId as number, siteProfileId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: pagesByWorkKey(workId) });
    },
  });
}
