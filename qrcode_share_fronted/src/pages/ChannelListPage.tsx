import { useNavigate } from 'react-router-dom';
import { ChannelList } from '@/components/channel';
import { IconPeople } from '@/components/icons';
import type { Channel } from '@/types';

export function ChannelListPage() {
  const navigate = useNavigate();

  const handleChannelClick = (channel: Channel) => {
    navigate(`/channel/${channel.id}`);
  };

  return (
    <div>
      <div className="relative overflow-hidden rounded-xl px-6 py-8">
        <div className="absolute inset-0 bg-gradient-to-b from-canvas to-brand-peach/10" />
        <div className="relative z-10">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-brand-pink/10">
              <IconPeople size={20} className="text-brand-pink" />
            </div>
            <div>
              <h1 className="text-2xl font-bold text-ink">Channels</h1>
              <div className="mt-1 h-0.5 w-10 rounded-full bg-brand-pink" />
            </div>
          </div>
          <p className="mt-3 text-sm text-muted">
            Browse and join existing channels
          </p>
        </div>
      </div>
      <div className="mt-6">
        <ChannelList onChannelClick={handleChannelClick} />
      </div>
    </div>
  );
}
