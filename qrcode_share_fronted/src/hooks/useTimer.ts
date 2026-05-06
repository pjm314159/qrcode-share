import { useEffect, useRef, useCallback } from 'react';

export function useInterval(callback: () => void, delay: number | null) {
  const savedCallback = useRef(callback);

  useEffect(() => {
    savedCallback.current = callback;
  }, [callback]);

  useEffect(() => {
    if (delay === null) return;

    const id = setInterval(() => savedCallback.current(), delay);
    return () => clearInterval(id);
  }, [delay]);
}

export function useCountdown(targetDate: string): number {
  const targetTime = new Date(targetDate).getTime();

  const getRemaining = useCallback((): number => {
    const diff = targetTime - Date.now();
    return Math.max(0, Math.floor(diff / 1000));
  }, [targetTime]);

  return getRemaining();
}
