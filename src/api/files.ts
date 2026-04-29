import { command } from './client';

export interface FileOutputResult {
  path: string;
}

export function exportPageText(id: number): Promise<FileOutputResult> {
  return command<FileOutputResult>('export_page_text', { args: { id } });
}

export function backupDatabase(): Promise<FileOutputResult> {
  return command<FileOutputResult>('backup_database');
}
