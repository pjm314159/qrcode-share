import { useState, useCallback } from 'react';
import { useSettingsStore } from '@/stores/settingsStore';
import {
  IconClose,
  IconCopy,
  IconWarning,
  IconDanger,
} from '@/components/icons';

interface ChannelSettingsPanelProps {
  channelId: string;
  channelName?: string;
  onClose: () => void;
}

export function ChannelSettingsPanel({ channelId, channelName, onClose }: ChannelSettingsPanelProps) {
  const autoOpenLinks = useSettingsStore((s) => s.autoOpenLinks);
  const setAutoOpenLinks = useSettingsStore((s) => s.setAutoOpenLinks);
  const autoOpenReceivedLinks = useSettingsStore((s) => s.autoOpenReceivedLinks);
  const setAutoOpenReceivedLinks = useSettingsStore((s) => s.setAutoOpenReceivedLinks);
  const [confirmAutoOpen, setConfirmAutoOpen] = useState(false);
  const [confirmAutoOpenReceived, setConfirmAutoOpenReceived] = useState(false);

  const handleToggleAutoOpen = useCallback(
    (value: boolean) => {
      if (value && !autoOpenLinks) {
        setConfirmAutoOpen(true);
      } else {
        setAutoOpenLinks(false);
      }
    },
    [autoOpenLinks, setAutoOpenLinks]
  );

  const confirmAutoOpenAction = useCallback(() => {
    setAutoOpenLinks(true);
    setConfirmAutoOpen(false);
  }, [setAutoOpenLinks]);

  const handleToggleAutoOpenReceived = useCallback(
    (value: boolean) => {
      if (value && !autoOpenReceivedLinks) {
        setConfirmAutoOpenReceived(true);
      } else {
        setAutoOpenReceivedLinks(false);
      }
    },
    [autoOpenReceivedLinks, setAutoOpenReceivedLinks]
  );

  const confirmAutoOpenReceivedAction = useCallback(() => {
    setAutoOpenReceivedLinks(true);
    setConfirmAutoOpenReceived(false);
  }, [setAutoOpenReceivedLinks]);

  const channelUrl = `${window.location.origin}/channel/${channelId}`;
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(channelUrl);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // ignore
    }
  }, [channelUrl]);

  return (
    <div className="fixed inset-0 z-50 flex justify-end bg-ink/50 backdrop-blur-sm" onClick={onClose}>
      <div
        className="w-full max-w-sm h-full bg-canvas shadow-xl overflow-y-auto animate-slide-up"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex items-center justify-between border-b border-hairline px-5 py-4">
          <h3 className="text-lg font-semibold text-ink">Settings</h3>
          <button
            onClick={onClose}
            className="rounded-full p-1 text-muted hover:bg-surface-soft hover:text-ink"
          >
            <IconClose size={20} />
          </button>
        </div>

        <div className="p-5 space-y-6">
          <div>
            <h4 className="text-sm font-semibold text-ink uppercase tracking-wider">
              Channel Info
            </h4>
            <div className="mt-3 space-y-2">
              <div className="rounded-md bg-surface-soft p-3">
                <p className="text-xs text-muted">Channel ID</p>
                <p className="mt-0.5 font-mono text-sm text-ink">{channelId}</p>
              </div>
              {channelName && (
                <div className="rounded-md bg-surface-soft p-3">
                  <p className="text-xs text-muted">Name</p>
                  <p className="mt-0.5 text-sm text-ink">{channelName}</p>
                </div>
              )}
              <div className="rounded-md bg-surface-soft p-3">
                <p className="text-xs text-muted">Invite Link</p>
                <p className="mt-0.5 break-all text-sm text-ink">{channelUrl}</p>
                <button
                  onClick={handleCopy}
                  className="mt-2 inline-flex items-center gap-1 text-xs font-medium text-ink hover:text-ink-active"
                >
                  <IconCopy size={14} />
                  {copied ? 'Copied!' : 'Copy Link'}
                </button>
              </div>
            </div>
          </div>

          <div>
            <h4 className="text-sm font-semibold text-ink uppercase tracking-wider">
              Link Behavior
            </h4>
            <div className="mt-3 space-y-3">
              <label className="flex items-center justify-between rounded-md bg-surface-soft p-3 cursor-pointer">
                <div>
                  <p className="text-sm font-medium text-ink">
                    Auto-open on click
                  </p>
                  <p className="text-xs text-muted">
                    Navigate to links without confirmation when clicked
                  </p>
                </div>
                <div
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    autoOpenLinks ? 'bg-ink' : 'bg-hairline'
                  }`}
                  onClick={() => handleToggleAutoOpen(!autoOpenLinks)}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-on-primary transition-transform ${
                      autoOpenLinks ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </div>
              </label>
              {autoOpenLinks && (
                <div className="rounded-md border border-warning/30 bg-warning/10 p-3">
                  <p className="flex items-start gap-1.5 text-xs font-medium text-warning">
                    <IconWarning size={14} className="shrink-0 mt-0.5" />
                    Clicking a link will navigate away from this page without confirmation.
                  </p>
                </div>
              )}

              <label className="flex items-center justify-between rounded-md bg-surface-soft p-3 cursor-pointer">
                <div>
                  <p className="text-sm font-medium text-ink">
                    Auto-open on receive
                  </p>
                  <p className="text-xs text-muted">
                    Navigate to links immediately when received via WebSocket
                  </p>
                </div>
                <div
                  className={`relative inline-flex h-6 w-11 items-center rounded-full transition-colors ${
                    autoOpenReceivedLinks ? 'bg-error' : 'bg-hairline'
                  }`}
                  onClick={() => handleToggleAutoOpenReceived(!autoOpenReceivedLinks)}
                >
                  <span
                    className={`inline-block h-4 w-4 transform rounded-full bg-on-primary transition-transform ${
                      autoOpenReceivedLinks ? 'translate-x-6' : 'translate-x-1'
                    }`}
                  />
                </div>
              </label>
              {autoOpenReceivedLinks && (
                <div className="rounded-md border border-error/30 bg-error/10 p-3">
                  <p className="flex items-start gap-1.5 text-xs font-medium text-error">
                    <IconDanger size={14} className="shrink-0 mt-0.5" />
                    DANGER: Links will navigate away from this page automatically as soon as they are received. This is extremely dangerous if you do not trust all channel members, as malicious links will be opened without any action from you.
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>

        {confirmAutoOpen && (
          <div className="fixed inset-0 z-[60] flex items-center justify-center bg-ink/50" onClick={() => setConfirmAutoOpen(false)}>
            <div className="mx-4 w-full max-w-sm rounded-lg bg-canvas p-6 shadow-xl" onClick={(e) => e.stopPropagation()}>
              <h3 className="flex items-center gap-2 text-lg font-semibold text-warning">
                <IconWarning size={20} />
                Confirm Auto-open
              </h3>
              <p className="mt-2 text-sm text-body">
                Clicking a link will navigate away from this page without asking for confirmation. This could be dangerous if shared links are untrusted.
              </p>
              <div className="mt-4 flex gap-2">
                <button
                  onClick={() => setConfirmAutoOpen(false)}
                  className="flex-1 rounded-md border border-hairline px-4 py-2 text-sm font-medium text-ink hover:bg-surface-soft"
                >
                  Cancel
                </button>
                <button
                  onClick={confirmAutoOpenAction}
                  className="flex-1 rounded-md bg-warning px-4 py-2 text-sm font-semibold text-on-primary hover:bg-warning/90"
                >
                  Enable
                </button>
              </div>
            </div>
          </div>
        )}

        {confirmAutoOpenReceived && (
          <div className="fixed inset-0 z-[60] flex items-center justify-center bg-ink/50" onClick={() => setConfirmAutoOpenReceived(false)}>
            <div className="mx-4 w-full max-w-sm rounded-lg bg-canvas p-6 shadow-xl" onClick={(e) => e.stopPropagation()}>
              <h3 className="flex items-center gap-2 text-lg font-semibold text-error">
                <IconDanger size={20} />
                Dangerous Operation
              </h3>
              <p className="mt-2 text-sm text-body">
                Links will navigate away from this page automatically as soon as they are received from other users. This means malicious links could be opened without any action from you. Only enable this in channels where you fully trust all members.
              </p>
              <div className="mt-3 rounded-md border border-error/30 bg-error/10 p-3">
                <p className="text-xs font-medium text-error">
                  This feature could expose you to phishing, malware, or other attacks if any channel member shares a malicious link.
                </p>
              </div>
              <div className="mt-4 flex gap-2">
                <button
                  onClick={() => setConfirmAutoOpenReceived(false)}
                  className="flex-1 rounded-md border border-hairline px-4 py-2 text-sm font-medium text-ink hover:bg-surface-soft"
                >
                  Cancel
                </button>
                <button
                  onClick={confirmAutoOpenReceivedAction}
                  className="flex-1 rounded-md bg-error px-4 py-2 text-sm font-semibold text-on-primary hover:bg-error/90"
                >
                  I understand the risk
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
