import { useEffect, useCallback } from 'react';
import { useMessageStore } from '@/stores/messageStore';
import { MessageCard } from './MessageCard';
import { Button } from '@/components/ui';
import { IconEmpty } from '@/components/icons';

interface MessageListProps {
  channelId: string;
  onLinkClick?: (link: string) => void;
}

export function MessageList({ channelId, onLinkClick }: MessageListProps) {
  const messages = useMessageStore((s) => s.messages);
  const hasMore = useMessageStore((s) => s.hasMore);
  const loading = useMessageStore((s) => s.loading);
  const fetchMessages = useMessageStore((s) => s.fetchMessages);
  const removeExpiredMessages = useMessageStore((s) => s.removeExpiredMessages);
  const clearMessages = useMessageStore((s) => s.clearMessages);

  useEffect(() => {
    fetchMessages(channelId);
    return () => {
      clearMessages();
    };
  }, [channelId, fetchMessages, clearMessages]);

  useEffect(() => {
    const interval = setInterval(() => {
      removeExpiredMessages();
    }, 30_000);
    return () => clearInterval(interval);
  }, [removeExpiredMessages]);

  const handleLoadMore = useCallback(() => {
    const oldest = messages[messages.length - 1];
    if (oldest) {
      fetchMessages(channelId, { cursor: oldest.id });
    }
  }, [channelId, messages, fetchMessages]);

  if (messages.length === 0 && !loading) {
    return (
      <div className="py-8 text-center text-muted">
        <IconEmpty size={48} className="mx-auto text-muted-soft" />
        <p className="mt-4 text-lg font-medium text-ink">No messages yet</p>
        <p className="mt-1 text-sm">Be the first to share a link!</p>
      </div>
    );
  }

  return (
    <div className="space-y-2">
      {messages.map((message) => (
        <MessageCard key={message.id} message={message} onLinkClick={onLinkClick} />
      ))}

      {hasMore && (
        <div className="flex justify-center pt-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={handleLoadMore}
            loading={loading}
          >
            Load more messages
          </Button>
        </div>
      )}
    </div>
  );
}
