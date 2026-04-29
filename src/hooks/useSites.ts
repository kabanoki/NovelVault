import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { siteCreate, siteDelete, siteList, siteUpdate, type SiteCreateArgs, type SiteUpdateArgs } from '../api/sites';

const sitesKey = ['sites'] as const;

export function useSites() {
  return useQuery({
    queryKey: sitesKey,
    queryFn: siteList,
  });
}

export function useCreateSite() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: SiteCreateArgs) => siteCreate(args),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: sitesKey });
    },
  });
}

export function useUpdateSite() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: SiteUpdateArgs) => siteUpdate(args),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: sitesKey });
    },
  });
}

export function useDeleteSite() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => siteDelete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: sitesKey });
    },
  });
}
