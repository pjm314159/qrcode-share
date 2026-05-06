import { useState, useEffect, useCallback } from 'react';
import { useChannelStore } from '@/stores/channelStore';
import { ChannelCard } from './ChannelCard';
import { Button } from '@/components/ui';
import { IconEmpty, IconChevronLeft, IconArrowRight } from '@/components/icons';
import type { Channel } from '@/types';

interface ChannelListProps {
  onChannelClick: (channel: Channel) => void;
}

export function ChannelList({ onChannelClick }: ChannelListProps) {
  const channels = useChannelStore((s) => s.channels);
  const total = useChannelStore((s) => s.total);
  const page = useChannelStore((s) => s.page);
  const limit = useChannelStore((s) => s.limit);
  const loading = useChannelStore((s) => s.loading);
  const error = useChannelStore((s) => s.error);
  const fetchChannels = useChannelStore((s) => s.fetchChannels);
  const clearError = useChannelStore((s) => s.clearError);

  const [search, setSearch] = useState('');

  const loadChannels = useCallback(() => {
    fetchChannels({ page, limit, search: search || undefined });
  }, [fetchChannels, page, limit, search]);

  useEffect(() => {
    loadChannels();
  }, [loadChannels]);

  const handleSearch = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      fetchChannels({ page: 1, limit, search: search || undefined });
    },
    [fetchChannels, limit, search]
  );

  const handleNextPage = useCallback(() => {
    fetchChannels({ page: page + 1, limit, search: search || undefined });
  }, [fetchChannels, page, limit, search]);

  const handlePrevPage = useCallback(() => {
    if (page > 1) {
      fetchChannels({ page: page - 1, limit, search: search || undefined });
    }
  }, [fetchChannels, page, limit, search]);

  const totalPages = Math.ceil(total / limit);

  return (
    <div className="space-y-4">
      <form onSubmit={handleSearch} className="flex gap-2">
        <input
          type="text"
          placeholder="Search channels..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="w-full rounded-md border border-hairline bg-canvas px-4 py-3 text-sm text-ink
            focus:border-ink focus:outline-none focus:ring-2 focus:ring-ink/20"
        />
        <Button type="submit" size="sm">
          Search
        </Button>
      </form>

      {error && (
        <div className="rounded-md bg-error/10 p-3 text-sm text-error">
          <p>{error}</p>
          <button onClick={clearError} className="mt-1 underline">
            Dismiss
          </button>
        </div>
      )}

      {loading && channels.length === 0 ? (
        <div className="flex justify-center py-12">
          <div className="h-8 w-8 animate-spin rounded-full border-4 border-surface-card border-t-ink" />
        </div>
      ) : channels.length === 0 ? (
        <div className="py-12 text-center text-muted">
          <IconEmpty size={48} className="mx-auto text-muted-soft" />
          <p className="mt-4 text-lg font-medium text-ink">No channels found</p>
          <p className="mt-1 text-sm">Create a new channel to get started</p>
        </div>
      ) : (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
            {channels.map((channel) => (
              <ChannelCard
                key={channel.id}
                channel={channel}
                onClick={onChannelClick}
              />
            ))}
          </div>

          {totalPages > 1 && (
            <div className="flex items-center justify-between pt-4">
              <Button
                variant="ghost"
                size="sm"
                onClick={handlePrevPage}
                disabled={page <= 1 || loading}
              >
                <IconChevronLeft size={16} className="mr-1" />
                Previous
              </Button>
              <span className="text-sm text-muted">
                Page {page} of {totalPages}
              </span>
              <Button
                variant="ghost"
                size="sm"
                onClick={handleNextPage}
                disabled={page >= totalPages || loading}
              >
                Next
                <IconArrowRight size={16} className="ml-1" />
              </Button>
            </div>
          )}
        </>
      )}
    </div>
  );
}
