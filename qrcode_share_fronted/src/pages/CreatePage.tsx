import { useNavigate } from 'react-router-dom';
import { CreateChannelForm } from '@/components/channel';
import { DecorativeBlob } from '@/components/common';
import type { Channel, CreateChannelRequest } from '@/types';
import * as channelApi from '@/api/channels';

export function CreatePage() {
  const navigate = useNavigate();

  const handleCreate = async (data: CreateChannelRequest): Promise<Channel> => {
    return channelApi.createChannel(data);
  };

  const handleCreated = (channel: Channel) => {
    navigate(`/channel/${channel.id}`);
  };

  return (
    <div className="relative mx-auto max-w-lg overflow-hidden">
      <DecorativeBlob color="#ffb084" size={200} x="-10%" y="20%" opacity={0.1} />
      <DecorativeBlob color="#b8a4ed" size={160} x="80%" y="60%" opacity={0.08} />

      <div className="relative z-10">
        <h1 className="text-2xl font-bold text-ink">Create Channel</h1>
        <p className="mt-1 text-sm text-muted">
          Set up a new channel to start sharing links
        </p>
        <div className="mt-6">
          <CreateChannelForm onSubmit={handleCreate} onSuccess={handleCreated} />
        </div>
      </div>
    </div>
  );
}
