import { command } from './client';
import type { Favorite, FavoriteCheckResult, FavoritesGrouped } from '../types/entities';

export function favoriteAdd(pageId: number): Promise<Favorite> {
  return command<Favorite>('favorite_add', { args: { pageId } });
}

export function favoriteRemove(pageId: number): Promise<void> {
  return command<void>('favorite_remove', { args: { pageId } });
}

export function favoriteCheck(pageId: number): Promise<FavoriteCheckResult> {
  return command<FavoriteCheckResult>('favorite_check', { args: { pageId } });
}

export function favoriteList(): Promise<FavoritesGrouped> {
  return command<FavoritesGrouped>('favorite_list');
}
