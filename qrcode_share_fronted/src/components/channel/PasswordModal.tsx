import React, { useState, useCallback } from 'react';
import { Input, Button } from '@/components/ui';
import { IMAGES } from '@/constants/images';

interface PasswordModalProps {
  channelName?: string;
  onSubmit: (password: string) => void;
  loading?: boolean;
}

export function PasswordModal({ channelName, onSubmit, loading = false }: PasswordModalProps) {
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      setError(null);

      if (!password.trim()) {
        setError('Password is required');
        return;
      }

      onSubmit(password.trim());
    },
    [password, onSubmit]
  );

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-ink/50 backdrop-blur-sm">
      <div className="mx-4 w-full max-w-sm rounded-lg bg-canvas p-6 shadow-xl">
        <div className="flex justify-center mb-4">
          <img
            src={IMAGES.passwordLock}
            alt="Locked channel"
            width={120}
            height={90}
          />
        </div>
        <h2 className="text-lg font-semibold text-ink text-center mb-2">
          Protected Channel
        </h2>
        {channelName && (
          <p className="mb-4 text-sm text-muted text-center">
            {channelName} requires a password to join
          </p>
        )}

        <form onSubmit={handleSubmit} className="space-y-3">
          <Input
            type="password"
            label="Password"
            placeholder="Enter channel password"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            autoFocus
          />
          {error && <p className="text-sm text-error">{error}</p>}
          <Button type="submit" className="w-full" loading={loading}>
            Join Channel
          </Button>
        </form>
      </div>
    </div>
  );
}
