import { Card } from '@/components/ui';
import { IconLock, IconLocation, IconUser, IconPeople } from '@/components/icons';
import type { Channel } from '@/types';

interface ChannelCardProps {
  channel: Channel;
  onClick: (channel: Channel) => void;
}

export function ChannelCard({ channel, onClick }: ChannelCardProps) {
  const hasPassword = channel.has_password;
  const messageCount = channel.message_count ?? 0;

  return (
    <Card hoverable onClick={() => onClick(channel)} className="animate-fade-in">
      <div className="flex items-start justify-between gap-2">
        <h3 className="text-lg font-semibold text-ink truncate">
          {channel.name}
        </h3>
        <div className="flex shrink-0 items-center gap-1.5">
          {hasPassword && (
            <span className="inline-flex items-center rounded-full bg-brand-ochre/20 px-2 py-0.5 text-xs font-medium text-brand-ochre">
              <IconLock size={12} className="mr-1" />
              Locked
            </span>
          )}
          <span className="inline-flex items-center rounded-full bg-surface-card px-2 py-0.5 text-xs font-medium text-ink">
            {messageCount} msg{messageCount !== 1 ? 's' : ''}
          </span>
        </div>
      </div>

      <div className="mt-2 space-y-1">
        {channel.channel_type && (
          <p className="text-sm text-body">
            <span className="font-medium text-muted">Type:</span>{' '}
            {channel.channel_type}
          </p>
        )}
        {channel.location && (
          <p className="text-sm text-body flex items-center gap-1">
            <IconLocation size={14} className="text-muted-soft" />
            {channel.location}
          </p>
        )}
        {channel.teacher && (
          <p className="text-sm text-body flex items-center gap-1">
            <IconUser size={14} className="text-muted-soft" />
            {channel.teacher}
          </p>
        )}
        {channel.link_limitation && channel.link_limitation.length > 0 && (
          <p className="text-sm text-body">
            <span className="font-medium text-muted">Domains:</span>{' '}
            {channel.link_limitation.join(', ')}
          </p>
        )}
      </div>

      <div className="mt-3 flex items-center justify-between border-t border-hairline pt-2">
        <p className="text-xs text-muted-soft">
          Created {new Date(channel.created_at).toLocaleDateString()}
        </p>
        {channel.subscriber_count > 0 && (
          <p className="flex items-center gap-1 text-xs text-muted-soft">
            <IconPeople size={12} />
            {channel.subscriber_count} online
          </p>
        )}
      </div>
    </Card>
  );
}
