import { useState, useCallback } from 'react';
import { SmartQRScanner } from '@/components/qrcode';
import { Input } from '@/components/ui';
import { useMessageStore } from '@/stores/messageStore';
import { useChannelStore } from '@/stores/channelStore';
import { validateLink, extractDomain } from '@/utils/helpers';
import {
  IconScan,
  IconSend,
  IconClose,
} from '@/components/icons';

interface MessageEditorProps {
  channelId: string;
  onClose: () => void;
  autoStartScanner?: boolean;
}

export function MessageEditor({ channelId, onClose, autoStartScanner = false }: MessageEditorProps) {
  const [name, setName] = useState(() => localStorage.getItem('lastSenderName') || '');
  const [link, setLink] = useState('');
  const [expireSeconds, setExpireSeconds] = useState('3600');
  const [showScanner, setShowScanner] = useState(autoStartScanner);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const sendMessage = useMessageStore((s) => s.sendMessage);
  const currentChannel = useChannelStore((s) => s.currentChannel);

  const isLinkAllowed = useCallback(
    (url: string): boolean => {
      if (!currentChannel?.link_limitation?.length) return true;
      const d = extractDomain(url);
      return currentChannel.link_limitation.some((allowed) => d.endsWith(allowed));
    },
    [currentChannel]
  );

  const doSend = useCallback(
    async (scannedLink: string, senderName?: string) => {
      if (!validateLink(scannedLink)) {
        setError('Invalid URL. Must start with http:// or https://');
        return false;
      }
      if (!isLinkAllowed(scannedLink)) {
        setError('This domain is not allowed in this channel');
        return false;
      }

      setSending(true);
      setError(null);
      try {
        const sender = senderName || name.trim() || 'Anonymous';
        localStorage.setItem('lastSenderName', sender);
        await sendMessage(channelId, {
          name: sender,
          link: scannedLink,
          expire_seconds: parseInt(expireSeconds, 10) || 3600,
        });
        return true;
      } catch (err) {
        setError((err as { message?: string })?.message || 'Failed to send message');
        return false;
      } finally {
        setSending(false);
      }
    },
    [channelId, name, expireSeconds, sendMessage, isLinkAllowed]
  );

  const handleQRScan = useCallback(
    async (scannedLink: string) => {
      const success = await doSend(scannedLink);
      if (success) {
        setShowScanner(false);
        onClose();
      }
    },
    [doSend, onClose]
  );

  const handleManualSend = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault();
      setError(null);
      if (!link.trim()) {
        setError('Link is required');
        return;
      }
      const success = await doSend(link.trim());
      if (success) {
        setLink('');
        onClose();
      }
    },
    [link, doSend, onClose]
  );

  return (
    <div className="fixed inset-0 z-50 flex items-end justify-center bg-ink/30 backdrop-blur-sm" onClick={onClose}>
      <div
        className="w-full max-w-lg rounded-t-2xl bg-canvas p-5 shadow-xl animate-slide-up"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="mx-auto mb-3 h-1 w-10 rounded-full bg-hairline" />
        <div className="mb-4 flex items-center justify-between">
          <h3 className="flex items-center gap-2 text-lg font-semibold text-ink">
            <IconSend size={20} />
            Share a Link
          </h3>
          <button
            onClick={onClose}
            className="rounded-full p-1 text-muted hover:bg-surface-soft hover:text-ink"
          >
            <IconClose size={20} />
          </button>
        </div>

        <div className="space-y-4">
          <div className="grid grid-cols-3 gap-3">
            <div className="col-span-2">
              <Input
                label="Name"
                placeholder="Your name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                maxLength={100}
              />
            </div>
            <Input
              label="Expire (s)"
              type="number"
              min={60}
              max={86400}
              value={expireSeconds}
              onChange={(e) => setExpireSeconds(e.target.value)}
            />
          </div>

          <div>
            <button
              onClick={() => setShowScanner(!showScanner)}
              className="flex w-full items-center justify-center gap-2 rounded-md border-2 border-dashed border-brand-pink/40 bg-brand-pink/5 px-4 py-3 text-sm font-medium text-brand-pink transition-colors hover:border-brand-pink/60 hover:bg-brand-pink/10"
            >
              <IconScan size={18} />
              {showScanner ? 'Close Scanner' : 'Scan QR Code'}
            </button>

            {showScanner && (
              <div className="mt-3">
                <SmartQRScanner onScan={handleQRScan} onError={(err) => setError(err)} autoStart={autoStartScanner} />
                {sending && (
                  <p className="mt-2 text-center text-sm text-ink">Sending scanned link...</p>
                )}
              </div>
            )}
          </div>

          <div className="relative">
            <div className="absolute inset-x-0 top-1/2 h-px bg-hairline" />
            <div className="relative flex justify-center">
              <span className="bg-canvas px-3 text-xs text-muted-soft">
                or paste a link
              </span>
            </div>
          </div>

          <form onSubmit={handleManualSend} className="space-y-3">
            <Input
              type="url"
              placeholder="https://example.com"
              value={link}
              onChange={(e) => setLink(e.target.value)}
              maxLength={2048}
            />
            <button
              type="submit"
              disabled={sending || !link.trim()}
              className="w-full rounded-md bg-ink px-4 py-3 text-sm font-semibold text-on-primary hover:bg-ink-active disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
            >
              {sending ? 'Sending...' : 'Send Link'}
            </button>
          </form>

          {error && <p className="text-sm text-error">{error}</p>}
        </div>
      </div>
    </div>
  );
}
