import React, { useState, useCallback } from 'react';
import { Input, Button } from '@/components/ui';
import { IconClose } from '@/components/icons';

interface ChannelSearchProps {
  onSearch: (query: string) => void;
  placeholder?: string;
}

export function ChannelSearch({
  onSearch,
  placeholder = 'Search channels...',
}: ChannelSearchProps) {
  const [query, setQuery] = useState('');

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      onSearch(query.trim());
    },
    [query, onSearch]
  );

  const handleClear = useCallback(() => {
    setQuery('');
    onSearch('');
  }, [onSearch]);

  return (
    <form onSubmit={handleSubmit} className="flex gap-2">
      <div className="relative flex-1">
        <Input
          placeholder={placeholder}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
        />
        {query && (
          <button
            type="button"
            onClick={handleClear}
            className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-soft hover:text-ink"
          >
            <IconClose size={14} />
          </button>
        )}
      </div>
      <Button type="submit" size="sm">
        Search
      </Button>
    </form>
  );
}
