import React, { useState, useCallback } from 'react';
import { Button, Input, Card } from '@/components/ui';
import { IconChevronLeft } from '@/components/icons';
import type { CreateChannelRequest, Channel } from '@/types';

interface CreateChannelFormProps {
  onSubmit: (data: CreateChannelRequest) => Promise<Channel>;
  onSuccess: (channel: Channel) => void;
}

export function CreateChannelForm({ onSubmit, onSuccess }: CreateChannelFormProps) {
  const [name, setName] = useState('');
  const [password, setPassword] = useState('');
  const [channelType, setChannelType] = useState<string>('');
  const [location, setLocation] = useState('');
  const [teacher, setTeacher] = useState('');
  const [linkLimitation, setLinkLimitation] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAdvanced, setShowAdvanced] = useState(false);

  const handleSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      setError(null);

      if (!name.trim()) {
        setError('Channel name is required');
        return;
      }

      setLoading(true);
      try {
        const data: CreateChannelRequest = {
          name: name.trim(),
          password: password.trim() || undefined,
          channel_type: channelType || undefined,
          location: location || undefined,
          teacher: teacher || undefined,
          link_limitation: linkLimitation
            ? linkLimitation
                .split(',')
                .map((d) => d.trim())
                .filter(Boolean)
            : undefined,
        };
        const channel = await onSubmit(data);
        onSuccess(channel);
      } catch (err) {
        setError((err as { message?: string })?.message || 'Failed to create channel');
      } finally {
        setLoading(false);
      }
    },
    [name, password, channelType, location, teacher, linkLimitation, onSubmit, onSuccess]
  );

  return (
    <Card variant="cream">
      <form onSubmit={handleSubmit} className="space-y-4">
        <Input
          label="Channel Name"
          placeholder="Enter channel name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          required
          maxLength={100}
        />

        <Input
          label="Password (optional)"
          type="password"
          placeholder="Leave empty for public channel"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          maxLength={50}
          helperText="Set a password to restrict access"
        />

        <Button type="submit" loading={loading} className="w-full">
          Create Channel
        </Button>

        <button
          type="button"
          onClick={() => setShowAdvanced(!showAdvanced)}
          className="flex items-center gap-1 text-sm text-muted hover:text-ink transition-colors"
        >
          <IconChevronLeft size={14} className={`transform transition-transform ${showAdvanced ? 'rotate-90' : '-rotate-90'}`} />
          Advanced options
        </button>

        {showAdvanced && (
          <div className="space-y-4 animate-slide-down">
            <Input
              label="Channel Type (optional)"
              placeholder="e.g., classroom, meeting"
              value={channelType}
              onChange={(e) => setChannelType(e.target.value)}
            />

            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Location (optional)"
                placeholder="e.g., Room 301"
                value={location}
                onChange={(e) => setLocation(e.target.value)}
              />
              <Input
                label="Teacher (optional)"
                placeholder="e.g., Prof. Smith"
                value={teacher}
                onChange={(e) => setTeacher(e.target.value)}
              />
            </div>

            <Input
              label="Link Limitation (optional)"
              placeholder="e.g., example.com, github.com"
              value={linkLimitation}
              onChange={(e) => setLinkLimitation(e.target.value)}
              helperText="Comma-separated allowed domains"
            />
          </div>
        )}

        {error && (
          <p className="text-sm text-error">{error}</p>
        )}
      </form>
    </Card>
  );
}
