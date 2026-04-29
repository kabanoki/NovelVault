import { useEffect, useState } from 'react';

export type ReaderFontSize = 'small' | 'medium' | 'large';

const STORAGE_KEY = 'novel-vault.readerFontSize';

function normalizeFontSize(value: string | null): ReaderFontSize {
  return value === 'small' || value === 'large' ? value : 'medium';
}

export function useReaderSettings() {
  const [readerFontSize, setReaderFontSize] = useState<ReaderFontSize>(() =>
    normalizeFontSize(window.localStorage.getItem(STORAGE_KEY)),
  );

  useEffect(() => {
    window.localStorage.setItem(STORAGE_KEY, readerFontSize);
  }, [readerFontSize]);

  return { readerFontSize, setReaderFontSize };
}
