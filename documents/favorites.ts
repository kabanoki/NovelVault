// =============================================================================
// favorites.ts
// お気に入り機能の型定義（TypeScript側）
// 対応要件定義: v4.3 / スキーマ: 002_add_favorites.sql
// =============================================================================
// 既存の commands.ts に追記する想定
// =============================================================================

import type { IsoDateString } from './commands';


// =============================================================================
// エンティティ型
// =============================================================================

/** お気に入りレコード（DB直の形） */
export interface Favorite {
  id:        number;
  pageId:    number;
  createdAt: IsoDateString;
}

/**
 * お気に入り一覧の1行
 * サイト名・作品名・ページタイトルを含む結合済みの形
 */
export interface FavoriteListItem {
  favoriteId:   number;
  favoritedAt:  IsoDateString;
  pageId:       number;
  pageTitle:    string | null;
  pageNumber:   number | null;
  workId:       number;
  workTitle:    string;
  siteId:       number;
  siteName:     string;
}

/**
 * サイト・作品でグループ化されたお気に入り一覧
 * フロントエンドで UI 描画する際に使う構造
 */
export interface FavoritesGrouped {
  groups: FavoriteSiteGroup[];
}

export interface FavoriteSiteGroup {
  siteId:   number;
  siteName: string;
  works:    FavoriteWorkGroup[];
}

export interface FavoriteWorkGroup {
  workId:    number;
  workTitle: string;
  pages:     FavoriteListItem[];
}


// =============================================================================
// コマンド引数型
// =============================================================================

export interface FavoriteAddArgs {
  pageId: number;
}

export interface FavoriteRemoveArgs {
  pageId: number;
}

export interface FavoriteCheckArgs {
  pageId: number;
}

/** お気に入り判定結果 */
export interface FavoriteCheckResult {
  isFavorite: boolean;
  favoriteId: number | null;
}


// =============================================================================
// invoke ラッパー（コメントアウト形式）
// =============================================================================
// import { invoke } from '@tauri-apps/api/core';
//
// export const favoriteAdd    = (args: FavoriteAddArgs)    => invoke<Favorite>('favorite_add', { args });
// export const favoriteRemove = (args: FavoriteRemoveArgs) => invoke<void>('favorite_remove', { args });
// export const favoriteList   = ()                          => invoke<FavoritesGrouped>('favorite_list');
// export const favoriteCheck  = (args: FavoriteCheckArgs)  => invoke<FavoriteCheckResult>('favorite_check', { args });
