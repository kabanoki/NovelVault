import { useMutation } from '@tanstack/react-query';
import { backupDatabase, exportPageText } from '../api/files';

export function useExportPageText() {
  return useMutation({
    mutationFn: (pageId: number) => exportPageText(pageId),
  });
}

export function useBackupDatabase() {
  return useMutation({
    mutationFn: backupDatabase,
  });
}
