import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { workCreate, workDelete, workListBySite, workUpdate, type WorkCreateArgs, type WorkUpdateArgs } from '../api/works';

export const worksBySiteKey = (siteId: number | null) => ['works', siteId] as const;

export function useWorks(siteId: number | null) {
  return useQuery({
    queryKey: worksBySiteKey(siteId),
    queryFn: () => workListBySite(siteId as number),
    enabled: siteId !== null,
  });
}

export function useCreateWork(siteId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: WorkCreateArgs) => workCreate(args),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: worksBySiteKey(siteId) });
    },
  });
}

export function useUpdateWork(siteId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: WorkUpdateArgs) => workUpdate(args),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: worksBySiteKey(siteId) });
    },
  });
}

export function useDeleteWork(siteId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => workDelete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: worksBySiteKey(siteId) });
    },
  });
}
