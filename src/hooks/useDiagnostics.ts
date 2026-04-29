import { useMutation } from '@tanstack/react-query';
import { listDuplicateSourceUrls } from '../api/diagnostics';

export function useDuplicateSourceUrlList() {
  return useMutation({
    mutationFn: listDuplicateSourceUrls,
  });
}
