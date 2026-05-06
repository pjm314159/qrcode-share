import { useState, useEffect, useCallback } from 'react';

interface CountdownTimerProps {
  expireAt: string;
  onExpire?: () => void;
}

export function CountdownTimer({ expireAt, onExpire }: CountdownTimerProps) {
  const [remaining, setRemaining] = useState(() =>
    Math.max(0, Math.floor((new Date(expireAt).getTime() - Date.now()) / 1000))
  );

  const updateRemaining = useCallback(() => {
    const diff = Math.max(0, Math.floor((new Date(expireAt).getTime() - Date.now()) / 1000));
    setRemaining(diff);
  }, [expireAt]);

  useEffect(() => {
    const interval = setInterval(updateRemaining, 1000);
    return () => clearInterval(interval);
  }, [updateRemaining]);

  useEffect(() => {
    if (remaining === 0 && onExpire) {
      onExpire();
    }
  }, [remaining, onExpire]);

  const formatTime = (seconds: number): string => {
    if (seconds <= 0) return 'Expired';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    if (hours > 0) {
      return `${hours}h ${minutes}m ${secs}s`;
    }
    if (minutes > 0) {
      return `${minutes}m ${secs}s`;
    }
    return `${secs}s`;
  };

  return (
    <span
      className={`text-xs font-medium ${
        remaining === 0
          ? 'text-error'
          : remaining < 60
            ? 'text-warning'
            : 'text-muted-soft'
      }`}
    >
      {formatTime(remaining)}
    </span>
  );
}
