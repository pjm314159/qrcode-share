import React, { useState, useCallback } from 'react';
import { Button, Input, Card } from '@/components/ui';
import { IconSend } from '@/components/icons';

interface SendMessageFormProps {
  onSend: (link: string, name?: string) => Promise<void>;
  channelId: string;
}

export function SendMessageForm({ onSend }: SendMessageFormProps) {
  const [link, setLink] = useState('');
  const [name, setName] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      setError(null);

      if (!link.trim()) {
        setError('Link is required');
        return;
      }

      setLoading(true);
      try {
        await onSend(link.trim(), name.trim() || undefined);
        setLink('');
        setName('');
      } catch (err) {
        setError((err as { message?: string })?.message || 'Failed to send message');
      } finally {
        setLoading(false);
      }
    },
    [link, name, onSend]
  );

  return (
    <Card variant="cream">
      <div className="flex items-center gap-2 mb-3">
        <IconSend size={20} className="text-ink" />
        <h3 className="text-lg font-semibold text-ink">Share a Link</h3>
      </div>
      <form onSubmit={handleSubmit} className="space-y-3">
        <Input
          label="Link URL"
          type="url"
          placeholder="https://example.com"
          value={link}
          onChange={(e) => setLink(e.target.value)}
          required
        />
        <Input
          label="Link Name (optional)"
          placeholder="Give this link a name"
          value={name}
          onChange={(e) => setName(e.target.value)}
        />
        {error && <p className="text-sm text-error">{error}</p>}
        <Button type="submit" loading={loading} className="w-full">
          Send Link
        </Button>
      </form>
    </Card>
  );
}
