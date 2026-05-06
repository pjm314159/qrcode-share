import type { Message } from '@/types';
import { extractDomain, formatRemainingTime } from '@/utils/helpers';
import { useInterval } from '@/hooks/useTimer';
import React, { useState, useCallback } from 'react';

interface MessageCardProps {
  message: Message;
  onLinkClick?: (link: string) => void;
}

export function MessageCard({ message, onLinkClick }: MessageCardProps) {
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

  const handleLinkClick = useCallback(
    (e: React.MouseEvent) => {
      if (onLinkClick) {
        e.preventDefault();
        onLinkClick(message.link);
      }
    },
    [onLinkClick, message.link]
  );

  return (
    <div
      className={`animate-slide-up rounded-lg border p-3 transition-opacity ${
        isExpired
          ? 'border-hairline bg-surface-soft opacity-50'
          : 'border-hairline bg-canvas'
      }`}
    >
      <div className="flex items-start justify-between gap-2">
        <div className="min-w-0 flex-1">
          <p className="font-medium text-ink truncate">
            {message.name}
          </p>
          <a
            href={message.link}
            target="_blank"
            rel="noopener noreferrer"
            onClick={handleLinkClick}
            className="mt-1 block text-sm text-ink hover:underline truncate"
          >
            {message.link}
          </a>
          {message.message_type && (
            <span className="mt-1 inline-block rounded-full bg-brand-lavender/20 px-2 py-0.5 text-xs font-medium text-brand-lavender">
              {message.message_type}
            </span>
          )}
          {domain && (
            <span className="mt-1 ml-1 inline-block rounded-full bg-surface-card px-2 py-0.5 text-xs text-muted">
              {domain}
            </span>
          )}
        </div>
        <div className="shrink-0 text-right">
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
