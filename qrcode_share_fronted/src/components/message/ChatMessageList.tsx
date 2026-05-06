import { useEffect, useCallback, useRef } from 'react';
import { useMessageStore } from '@/stores/messageStore';
import { IconEmpty } from '@/components/icons';
import { ChatMessageCard } from '@/components';

interface ChatMessageListProps {
  channelId: string;
  onLinkClick: (link: string) => void;
  ready: boolean;
}

export function ChatMessageList({ channelId, onLinkClick, ready }: ChatMessageListProps) {
  const messages = useMessageStore((s) => s.messages);
  const hasMore = useMessageStore((s) => s.hasMore);
  const loading = useMessageStore((s) => s.loading);
  const fetchMessages = useMessageStore((s) => s.fetchMessages);
  const removeExpiredMessages = useMessageStore((s) => s.removeExpiredMessages);
  const clearMessages = useMessageStore((s) => s.clearMessages);
  const scrollRef = useRef<HTMLDivElement>(null);
  const prevMessageCountRef = useRef(0);

  useEffect(() => {
    if (!ready) return;
    fetchMessages(channelId);
    return () => {
      clearMessages();
    };
  }, [channelId, fetchMessages, clearMessages, ready]);

  useEffect(() => {
    const interval = setInterval(() => {
      removeExpiredMessages();
    }, 30_000);
    return () => clearInterval(interval);
  }, [removeExpiredMessages]);

  useEffect(() => {
    if (messages.length > prevMessageCountRef.current && scrollRef.current) {
      scrollRef.current.scrollTop = 0;
    }
    prevMessageCountRef.current = messages.length;
  }, [messages.length]);

  const handleLoadMore = useCallback(() => {
    const oldest = messages[messages.length - 1];
    if (oldest) {
      fetchMessages(channelId, { cursor: oldest.id });
    }
  }, [channelId, messages, fetchMessages]);

  if (messages.length === 0 && !loading) {
    return (
      <div className="flex flex-1 items-center justify-center py-16 text-muted">
        <div className="text-center">
          <IconEmpty size={48} className="mx-auto text-muted-soft" />
          <p className="mt-3 text-lg font-medium text-ink">No messages yet</p>
          <p className="mt-1 text-sm">Be the first to share a link!</p>
        </div>
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="flex-1 overflow-y-auto px-4 py-4 space-y-3">
      {hasMore && (
        <div className="flex justify-center pb-2">
          <button
            onClick={handleLoadMore}
            disabled={loading}
            className="text-sm text-ink hover:text-ink-active disabled:opacity-50"
          >
            {loading ? 'Loading...' : 'Load earlier messages'}
          </button>
        </div>
      )}
      {messages.filter(Boolean).map((message) => (
        <ChatMessageCard key={message.id} message={message} onLinkClick={onLinkClick} />
      ))}
    </div>
  );
}
