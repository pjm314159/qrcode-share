import React, { useState, useEffect, useCallback } from 'react';
import { useParams, useNavigate, useLocation } from 'react-router-dom';
import { LinkWarning, Loading } from '@/components';
import { ChatMessageList } from '@/components/message/ChatMessageList';
import { MessageEditor } from '@/components/message/MessageEditor';
import { ChannelSettingsPanel } from '@/components/channel/ChannelSettingsPanel';
import { useLinkWarning } from '@/hooks/useLinkWarning';
import { useChannelStore } from '@/stores/channelStore';
import { useConnectionStore } from '@/stores/connectionStore';
import { useMessageStore } from '@/stores/messageStore';
import { useSettingsStore } from '@/stores/settingsStore';
import { useWebSocket } from '@/hooks';
import { Input } from '@/components/ui';
import {
  IconScan,
  IconPaste,
  IconBack,
  IconSettings,
  IconLock,
  IconSuccess,
  IconConnecting,
  IconDisconnected,
  IconPeople,
} from '@/components/icons';

export function ChannelPage() {
  const { channelId } = useParams<{ channelId: string }>();
  const navigate = useNavigate();
  const location = useLocation();
  const currentChannel = useChannelStore((s) => s.currentChannel);
  const fetchChannel = useChannelStore((s) => s.fetchChannel);
  const channelLoading = useChannelStore((s) => s.loading);
  const connectionState = useConnectionStore((s) => s.state);
  const subscriberCount = useConnectionStore((s) => s.subscriberCount);
  const autoOpenLinks = useSettingsStore((s) => s.autoOpenLinks);
  const setChannelPassword = useMessageStore((s) => s.setChannelPassword);
  const resetChannel = useMessageStore((s) => s.resetChannel);

  const navState = location.state as { password?: string } | null;
  const initialPassword = navState?.password || '';

  const [password, setPassword] = useState(initialPassword);
  const [passwordSubmitted, setPasswordSubmitted] = useState(!!initialPassword);
  const [needsPassword, setNeedsPassword] = useState(false);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [showEditor, setShowEditor] = useState(false);
  const [scannerMode, setScannerMode] = useState(false);
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    if (initialPassword) {
      setChannelPassword(initialPassword);
    }
  }, [initialPassword, setChannelPassword]);

  useEffect(() => {
    return () => {
      resetChannel();
    };
  }, [resetChannel]);

  const { pendingLink, requestOpen, confirm, cancel } = useLinkWarning();

  const wsReady = !needsPassword && !!currentChannel;
  useWebSocket(wsReady ? channelId || null : null, passwordSubmitted ? password : undefined);

  useEffect(() => {
    if (!channelId) return;
    if (needsPassword && !passwordSubmitted) return;

    fetchChannel(channelId, passwordSubmitted ? password : undefined).catch(
      (err) => {
        if (err?.status === 401 || err?.code === 'PASSWORD_REQUIRED' || err?.code === 'CHANNEL_PASSWORD_REQUIRED') {
          setPasswordSubmitted(false);
          setNeedsPassword(true);
        } else if (err?.status === 403 || err?.code === 'WRONG_PASSWORD') {
          setPasswordSubmitted(false);
          setPasswordError('Wrong password. Please try again.');
          setNeedsPassword(true);
        }
      }
    );
  }, [channelId, fetchChannel, passwordSubmitted, password, needsPassword]);

  const handlePasswordSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      setPasswordError(null);
      setPasswordSubmitted(true);
      setNeedsPassword(false);
      setChannelPassword(password);
    },
    [password, setChannelPassword]
  );

  const handleLinkClick = useCallback(
    (link: string) => {
      if (autoOpenLinks) {
        window.location.href = link;
      } else {
        requestOpen(link);
      }
    },
    [autoOpenLinks, requestOpen]
  );

  if (!channelId) {
    return (
      <div className="flex h-screen items-center justify-center text-muted">
        Channel not found
      </div>
    );
  }

  if (needsPassword && !passwordSubmitted) {
    return (
      <div className="flex h-screen items-center justify-center bg-canvas">
        <div className="mx-4 w-full max-w-sm">
          <div className="rounded-lg border border-hairline bg-canvas p-6 shadow-sm">
            <div className="flex items-center gap-3 mb-4">
              <IconLock size={24} className="text-brand-ochre" />
              <h2 className="text-lg font-semibold text-ink">
                This channel requires a password
              </h2>
            </div>
            <form onSubmit={handlePasswordSubmit} className="space-y-3">
              <Input
                type="password"
                placeholder="Enter channel password"
                value={password}
                onChange={(e) => setPassword(e.target.value)}
                required
                autoFocus
              />
              {passwordError && (
                <p className="text-sm text-error">{passwordError}</p>
              )}
              <button
                type="submit"
                className="w-full rounded-md bg-ink px-4 py-3 text-sm font-semibold text-on-primary hover:bg-ink-active transition-colors"
              >
                Join Channel
              </button>
            </form>
          </div>
        </div>
      </div>
    );
  }

  if (channelLoading && !currentChannel && !passwordSubmitted) {
    return (
      <div className="flex h-screen items-center justify-center bg-canvas">
        <Loading />
      </div>
    );
  }

  const ConnectionIcon = connectionState === 'connected'
    ? IconSuccess
    : connectionState === 'connecting' || connectionState === 'reconnecting'
      ? IconConnecting
      : IconDisconnected;

  const connectionColor = connectionState === 'connected'
    ? 'text-success'
    : connectionState === 'connecting' || connectionState === 'reconnecting'
      ? 'text-warning'
      : 'text-error';

  const connectionText =
    connectionState === 'connected'
      ? 'Connected'
      : connectionState === 'connecting'
        ? 'Connecting...'
        : connectionState === 'reconnecting'
          ? 'Reconnecting...'
          : 'Disconnected';

  return (
    <div className="flex h-screen flex-col bg-canvas">
      <div className="flex items-center justify-between border-b border-hairline bg-canvas px-4 py-3">
        <button
          onClick={() => navigate('/')}
          className="flex items-center gap-1 rounded-md px-2 py-1 text-muted hover:bg-surface-soft hover:text-ink transition-colors"
        >
          <IconBack size={18} />
        </button>

        <div className="text-center">
          <h1 className="font-semibold text-ink">
            {currentChannel?.name || 'Channel'}
          </h1>
          <p className="flex items-center justify-center gap-1 text-xs text-muted">
            <ConnectionIcon size={12} className={connectionColor} />
            {connectionText}
            {subscriberCount > 0 && (
              <>
                <span className="mx-0.5">·</span>
                <IconPeople size={12} className="text-muted-soft" />
                {subscriberCount} online
              </>
            )}
          </p>
        </div>

        <button
          onClick={() => setShowSettings(true)}
          className="rounded-md px-2 py-1 text-muted hover:bg-surface-soft hover:text-ink transition-colors"
        >
          <IconSettings size={20} />
        </button>
      </div>

      <ChatMessageList channelId={channelId} onLinkClick={handleLinkClick} ready={!!currentChannel} />

      <div className="border-t border-hairline bg-canvas px-4 py-3">
        <div className="flex gap-2">
          <button
            onClick={() => { setShowEditor(true); setScannerMode(true); }}
            className="flex-1 flex items-center justify-center gap-2 rounded-xl border-2 border-dashed border-brand-pink/40 bg-brand-pink/5 px-4 py-3 text-sm font-medium text-brand-pink transition-all hover:border-brand-pink/60 hover:bg-brand-pink/10 active:scale-[0.98]"
          >
            <IconScan size={18} />
            Scan & Share
          </button>
          <button
            onClick={() => { setShowEditor(true); setScannerMode(false); }}
            className="flex-1 flex items-center justify-center gap-2 rounded-xl border-2 border-dashed border-brand-teal/40 bg-brand-teal/5 px-4 py-3 text-sm font-medium text-brand-teal transition-all hover:border-brand-teal/60 hover:bg-brand-teal/10 active:scale-[0.98]"
          >
            <IconPaste size={18} />
            Paste Link
          </button>
        </div>
      </div>

      {showEditor && (
        <MessageEditor channelId={channelId} onClose={() => setShowEditor(false)} autoStartScanner={scannerMode} />
      )}

      {showSettings && (
        <ChannelSettingsPanel
          channelId={channelId}
          channelName={currentChannel?.name}
          onClose={() => setShowSettings(false)}
        />
      )}

      {pendingLink && (
        <LinkWarning link={pendingLink} onConfirm={confirm} onCancel={cancel} />
      )}
    </div>
  );
}
