import { useNavigate } from 'react-router-dom';
import { Card } from '@/components/ui';
import { CreateChannelForm } from '@/components/channel';
import { JoinChannelForm } from '@/components/channel';
import { IconArrowRight } from '@/components/icons';
import { DecorativeBlob } from '@/components/common';
import { IMAGES } from '@/constants/images';
import type { Channel, CreateChannelRequest } from '@/types';
import * as channelApi from '@/api/channels';

export function HomePage() {
  const navigate = useNavigate();

  const handleCreate = async (data: CreateChannelRequest): Promise<Channel> => {
    return channelApi.createChannel(data);
  };

  const handleCreated = (channel: Channel) => {
    navigate(`/channel/${channel.id}`);
  };

  return (
    <div className="space-y-16">
      <section className="relative overflow-hidden rounded-2xl px-6 py-12 md:py-16">
        <div className="absolute inset-0 bg-gradient-to-b from-canvas to-brand-peach/10" />

        <DecorativeBlob color="#b8a4ed" size={280} x="10%" y="5%" opacity={0.12} />
        <DecorativeBlob color="#a4d4c5" size={240} x="65%" y="50%" opacity={0.12} />
        <DecorativeBlob color="#ff4d8b" size={180} x="5%" y="60%" opacity={0.08} />

        <div className="relative z-10 flex flex-col items-center text-center">
          <h1 className="text-4xl font-bold text-ink md:text-5xl">
            QRcode Share
          </h1>
          <div className="mt-2 h-1 w-16 rounded-full bg-brand-pink" />
          <p className="mt-4 text-lg text-body max-w-xl">
            Share links instantly through QR codes. Scan, share, and open links in real-time with anyone in the same channel.
          </p>

          <div className="mt-8">
            <img
              src={IMAGES.hero}
              alt="QRcode Share illustration"
              width={400}
              height={300}
              className="rounded-xl drop-shadow-lg"
            />
          </div>
        </div>
      </section>

      <section className="grid gap-6 md:grid-cols-2">
        <Card variant="feature-peach">
          <div className="flex items-center gap-3 mb-4">
            <img src={IMAGES.featureCreate} alt="" width={48} height={48} className="rounded-lg" />
            <h2 className="text-2xl font-bold">Create a Channel</h2>
          </div>
          <p className="mb-6 text-sm opacity-80">
            Set up a new channel and share the QR code with others to start exchanging links.
          </p>
          <CreateChannelForm onSubmit={handleCreate} onSuccess={handleCreated} />
        </Card>

        <div className="space-y-6">
          <JoinChannelForm />

          <div className="text-center">
            <button
              onClick={() => navigate('/channels')}
              className="inline-flex items-center gap-1 text-sm font-medium text-muted hover:text-ink transition-colors"
            >
              Browse all channels
              <IconArrowRight size={16} />
            </button>
          </div>
        </div>
      </section>

      <section>
        <h2 className="text-2xl font-bold text-ink text-center mb-8">How It Works</h2>
        <div className="grid gap-6 md:grid-cols-3">
          <Card variant="feature-pink">
            <img src={IMAGES.featureScan} alt="" width={120} height={90} className="mb-4 rounded-lg" />
            <h3 className="text-xl font-bold mb-2">Scan</h3>
            <p className="text-sm opacity-80">
              Scan any QR code to extract the link and share it instantly with your channel.
            </p>
          </Card>

          <Card variant="feature-teal">
            <img src={IMAGES.featureShare} alt="" width={120} height={90} className="mb-4 rounded-lg" />
            <h3 className="text-xl font-bold mb-2">Share</h3>
            <p className="text-sm opacity-80">
              Share links through channels. Everyone in the channel receives your links in real-time.
            </p>
          </Card>

          <Card variant="feature-lavender">
            <img src={IMAGES.featureOpen} alt="" width={120} height={90} className="mb-4 rounded-lg" />
            <h3 className="text-xl font-bold mb-2">Open</h3>
            <p className="text-sm opacity-80">
              Open received links instantly. Enable auto-open for seamless link navigation.
            </p>
          </Card>
        </div>
      </section>
    </div>
  );
}
