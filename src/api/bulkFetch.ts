import { command } from './client';

export interface BulkFetchResult {
  createdCount: number;
  successCount: number;
  failedCount: number;
}

export function bulkFetchByProfile(workId: number, siteProfileId: number): Promise<BulkFetchResult> {
  return command<BulkFetchResult>('bulk_fetch_by_profile', {
    args: { workId, siteProfileId },
  });
}
