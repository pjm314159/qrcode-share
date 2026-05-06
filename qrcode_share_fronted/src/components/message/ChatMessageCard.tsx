import { useState, useCallback } from 'react';
import { useInterval } from '@/hooks/useTimer';
import { extractDomain, formatRemainingTime } from '@/utils/helpers';
import type { Message } from '@/types';

interface ChatMessageCardProps {
  message: Message;
  onLinkClick: (link: string) => void;
}

export function ChatMessageCard({ message, onLinkClick }: ChatMessageCardProps) {
  const [remaining, setRemaining] = useState(() =>
    Math.max(0, Math.floor((new Date(message.expire_at).getTime() - Date.now()) / 1000))
  );

  const updateRemaining = useCallback(() => {
    const diff = Math.max(0, Math.floor((new Date(message.expire_at).getTime() - Date.now()) / 1000));
    setRemaining(diff);
  }, [message.expire_at]);

  useInterval(updateRemaining, 1000);

  const domain = extractDomain(message.link);
  const isExpired = remaining <= 0;

  const handleClick = useCallback(() => {
    if (!isExpired) {
      onLinkClick(message.link);
    }
  }, [isExpired, onLinkClick, message.link]);

  return (
    <div
      onClick={handleClick}
      className={`rounded-lg border p-4 transition-all ${
        isExpired
          ? 'border-hairline bg-surface-soft opacity-40 cursor-default'
          : 'border-hairline bg-canvas cursor-pointer hover:border-ink/20 hover:shadow-sm active:scale-[0.98]'
      }`}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <p className="font-semibold text-ink truncate">
            {message.name}
          </p>
          <p className="mt-1 text-sm text-ink truncate">
            {message.link}
          </p>
          <div className="mt-2 flex flex-wrap items-center gap-1.5">
            {message.message_type && (
              <span className="inline-block rounded-full bg-brand-lavender/20 px-2 py-0.5 text-xs font-medium text-brand-lavender">
                {message.message_type}
              </span>
            )}
            {domain && (
              <span className="inline-block rounded-full bg-surface-card px-2 py-0.5 text-xs text-muted">
                {domain}
              </span>
            )}
          </div>
        </div>
        <div className="shrink-0">
          <span
            className={`text-xs font-medium ${
              isExpired
                ? 'text-error'
                : remaining < 60
                  ? 'text-warning'
                  : 'text-muted-soft'
            }`}
          >
            {formatRemainingTime(remaining)}
          </span>
        </div>
      </div>
    </div>
  );
}
