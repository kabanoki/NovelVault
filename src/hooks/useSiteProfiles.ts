import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  siteProfileCreate,
  siteProfileDelete,
  siteProfileList,
  siteProfileUpdate,
  type SiteProfileCreateArgs,
  type SiteProfileUpdateArgs,
} from '../api/siteProfiles';

export const siteProfilesKey = (siteId: number | null) => ['siteProfiles', siteId] as const;

export function useSiteProfiles(siteId: number | null) {
  return useQuery({
    queryKey: siteProfilesKey(siteId),
    queryFn: () => siteProfileList(siteId as number),
    enabled: siteId !== null,
  });
}

export function useCreateSiteProfile(siteId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: SiteProfileCreateArgs) => siteProfileCreate(args),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: siteProfilesKey(siteId) });
    },
  });
}

export function useUpdateSiteProfile(siteId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (args: SiteProfileUpdateArgs) => siteProfileUpdate(args),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: siteProfilesKey(siteId) });
    },
  });
}

export function useDeleteSiteProfile(siteId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (id: number) => siteProfileDelete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: siteProfilesKey(siteId) });
    },
  });
}
