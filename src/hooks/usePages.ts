import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { pageCreate, pageDelete, pageGet, pageListByWork, pageUpdate, type PageCreateArgs, type PageUpdateArgs } from '../api/pages';

export const pagesByWorkKey = (workId: number | null) => ['pages', workId] as const;
export const pageKey = (pageId: number | null) => ['page', pageId] as const;

export function usePages(workId: number | null) {
  return useQuery({
    queryKey: pagesByWorkKey(workId),
    queryFn: () => pageListByWork(workId as number),
    enabled: workId !== null,
  });
}

export function usePage(pageId: number | null) {
  return useQuery({
    queryKey: pageKey(pageId),
    queryFn: () => pageGet(pageId as number),
    enabled: pageId !== null,
  });
}

export function useCreatePage(workId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: PageCreateArgs) => pageCreate(args),
    onSuccess: (page) => {
      queryClient.invalidateQueries({ queryKey: pagesByWorkKey(workId) });
      queryClient.setQueryData(pageKey(page.id), page);
    },
  });
}

export function useUpdatePage(workId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: PageUpdateArgs) => pageUpdate(args),
    onSuccess: (page) => {
      queryClient.invalidateQueries({ queryKey: pagesByWorkKey(workId) });
      queryClient.setQueryData(pageKey(page.id), page);
    },
  });
}

export function useDeletePage(workId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => pageDelete(id),
    onSuccess: (_, id) => {
      queryClient.invalidateQueries({ queryKey: pagesByWorkKey(workId) });
      queryClient.removeQueries({ queryKey: pageKey(id) });
    },
  });
}
