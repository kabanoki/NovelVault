import { command } from './client';
import type { SiteProfile } from '../types/entities';

export interface SiteProfileCreateArgs {
  siteId: number;
  name: string;
  profileJson: string;
}

export interface SiteProfileUpdateArgs {
  id: number;
  name: string;
  profileJson: string;
}

export function siteProfileList(siteId: number): Promise<SiteProfile[]> {
  return command<SiteProfile[]>('site_profile_list', { args: { siteId } });
}

export function siteProfileCreate(args: SiteProfileCreateArgs): Promise<SiteProfile> {
  return command<SiteProfile>('site_profile_create', { args });
}

export function siteProfileUpdate(args: SiteProfileUpdateArgs): Promise<SiteProfile> {
  return command<SiteProfile>('site_profile_update', { args });
}

export function siteProfileDelete(id: number): Promise<void> {
  return command<void>('site_profile_delete', { args: { id } });
}
