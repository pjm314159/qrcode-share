import { create } from 'zustand';
import { WebSocketClient } from '@/api/websocket';
import type { ConnectionState } from '@/api/websocket';

interface ConnectionStore {
  client: WebSocketClient | null;
  state: ConnectionState;
  subscriberCount: number;

  connect: (channelId: string, password?: string) => void;
  disconnect: () => void;
  setSubscriberCount: (count: number) => void;
}

export const useConnectionStore = create<ConnectionStore>((set, get) => ({
  client: null,
  state: 'disconnected',
  subscriberCount: 0,

  connect: (channelId, password) => {
    const prev = get().client;
    if (prev) {
      prev.disconnect();
    }

    const wsClient = new WebSocketClient(channelId, password);

    wsClient.onStateChange((newState) => {
      set({ state: newState });
    });

    wsClient.connect();
    set({ client: wsClient, state: 'connecting' });
  },

  disconnect: () => {
    const { client } = get();
    if (client) {
      client.disconnect();
    }
    set({ client: null, state: 'disconnected', subscriberCount: 0 });
  },

  setSubscriberCount: (count) => {
    set({ subscriberCount: count });
  },
}));
