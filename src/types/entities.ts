export type IsoDateString = string;

export interface Site {
  id: number;
  name: string;
  baseUrl: string;
  createdAt: IsoDateString;
  updatedAt: IsoDateString;
}

export interface Work {
  id: number;
  siteId: number;
  siteProfileId: number | null;
  title: string;
  authorName: string | null;
  description: string | null;
  sourceUrl: string | null;
  sortOrder: number;
  createdAt: IsoDateString;
  updatedAt: IsoDateString;
}

export interface SiteProfile {
  id: number;
  siteId: number;
  name: string;
  profileJson: string;
  createdAt: IsoDateString;
  updatedAt: IsoDateString;
}

export interface Page {
  id: number;
  workId: number;
  pageNumber: number | null;
  sortOrder: number;
  title: string | null;
  sourceUrl: string | null;
  sourceType: 'normal' | 'wayback' | 'local';
  canonicalUrl: string | null;
  archivedAt: string | null;
  requestedEncoding: string | null;
  detectedEncoding: string | null;
  contentText: string | null;
  contentHtmlPath: string | null;
  fetchStatus: 'pending' | 'success' | 'fetch_failed' | 'parse_failed' | 'save_failed' | 'skipped';
  fetchError: string | null;
  fetchedAt: IsoDateString | null;
  createdAt: IsoDateString;
  updatedAt: IsoDateString;
}

export interface TitleSearchResults {
  sites: Site[];
  works: WorkSearchItem[];
  pages: PageSearchItem[];
}

export interface WorkSearchItem {
  id: number;
  siteId: number;
  siteName: string;
  title: string;
  authorName: string | null;
}

export interface PageSearchItem {
  id: number;
  workId: number;
  workTitle: string;
  siteId: number;
  siteName: string;
  title: string | null;
  pageNumber: number | null;
}

export interface FullTextSearchItem {
  pageId: number;
  pageTitle: string | null;
  pageNumber: number | null;
  workId: number;
  workTitle: string;
  siteId: number;
  siteName: string;
  snippet: string;
}

export interface Favorite {
  id: number;
  pageId: number;
  createdAt: IsoDateString;
}

export interface FavoriteCheckResult {
  isFavorite: boolean;
  favoriteId: number | null;
}

export interface FavoriteListItem {
  favoriteId: number;
  favoritedAt: IsoDateString;
  pageId: number;
  pageTitle: string | null;
  pageNumber: number | null;
  workId: number;
  workTitle: string;
  siteId: number;
  siteName: string;
}

export interface FavoritesGrouped {
  groups: FavoriteSiteGroup[];
}

export interface FavoriteSiteGroup {
  siteId: number;
  siteName: string;
  works: FavoriteWorkGroup[];
}

export interface FavoriteWorkGroup {
  workId: number;
  workTitle: string;
  pages: FavoriteListItem[];
}
