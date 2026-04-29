import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { favoriteAdd, favoriteCheck, favoriteList, favoriteRemove } from '../api/favorites';

export const favoritesKey = ['favorites'] as const;
export const favoriteCheckKey = (pageId: number | null) => ['favorite', pageId] as const;

export function useFavorites() {
  return useQuery({
    queryKey: favoritesKey,
    queryFn: favoriteList,
  });
}

export function useFavoriteCheck(pageId: number | null) {
  return useQuery({
    queryKey: favoriteCheckKey(pageId),
    queryFn: () => favoriteCheck(pageId as number),
    enabled: pageId !== null,
  });
}

export function useFavoriteAdd(pageId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => favoriteAdd(pageId as number),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: favoritesKey });
      queryClient.invalidateQueries({ queryKey: favoriteCheckKey(pageId) });
    },
  });
}

export function useFavoriteRemove(pageId: number | null) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => favoriteRemove(pageId as number),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: favoritesKey });
      queryClient.invalidateQueries({ queryKey: favoriteCheckKey(pageId) });
    },
  });
}
