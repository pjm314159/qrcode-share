import { useState, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button, Input, Card } from '@/components/ui';

export function JoinChannelForm() {
  const navigate = useNavigate();
  const [channelId, setChannelId] = useState('');
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      setError(null);

      const trimmed = channelId.trim().toLowerCase();
      if (!trimmed) {
        setError('Channel ID is required');
        return;
      }

      if (!/^[a-z0-9_-]+$/.test(trimmed)) {
        setError('Channel ID can only contain letters, numbers, hyphens and underscores');
        return;
      }

      navigate(`/channel/${trimmed}`);
    },
    [channelId, navigate]
  );

  return (
    <Card variant="cream">
      <h3 className="mb-4 text-lg font-semibold text-ink">
        Join a Channel
      </h3>
      <form onSubmit={handleSubmit} className="space-y-3">
        <Input
          label="Channel ID"
          placeholder="Enter channel ID"
          value={channelId}
          onChange={(e) => setChannelId(e.target.value)}
          required
          maxLength={50}
        />
        {error && <p className="text-sm text-error">{error}</p>}
        <Button type="submit" className="w-full">
          Join Channel
        </Button>
      </form>
    </Card>
  );
}
