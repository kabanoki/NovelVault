import { command } from './client';
import type { Site } from '../types/entities';

export interface SiteCreateArgs {
  name: string;
  baseUrl: string;
}

export interface SiteUpdateArgs extends SiteCreateArgs {
  id: number;
}

export function siteList(): Promise<Site[]> {
  return command<Site[]>('site_list');
}

export function siteCreate(args: SiteCreateArgs): Promise<Site> {
  return command<Site>('site_create', { args });
}

export function siteUpdate(args: SiteUpdateArgs): Promise<Site> {
  return command<Site>('site_update', { args });
}

export function siteDelete(id: number): Promise<void> {
  return command<void>('site_delete', { args: { id } });
}
