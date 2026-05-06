import { useState, useCallback } from 'react';

export function useLinkWarning() {
  const [pendingLink, setPendingLink] = useState<string | null>(null);

  const requestOpen = useCallback((link: string) => {
    setPendingLink(link);
  }, []);

  const confirm = useCallback(() => {
    setPendingLink(null);
  }, []);

  const cancel = useCallback(() => {
    setPendingLink(null);
  }, []);

  return { pendingLink, requestOpen, confirm, cancel };
}
