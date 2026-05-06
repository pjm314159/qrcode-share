import { create } from 'zustand';
import type {
  Channel,
  CreateChannelRequest,
  UpdateChannelRequest,
  ChannelListResponse,
} from '@/types';
import * as channelApi from '@/api/channels';

interface ChannelState {
  channels: Channel[];
  currentChannel: Channel | null;
  total: number;
  page: number;
  limit: number;
  loading: boolean;
  error: string | null;

  fetchChannels: (params?: {
    page?: number;
    limit?: number;
    channel_type?: string;
    search?: string;
  }) => Promise<void>;
  fetchChannel: (channelId: string, password?: string) => Promise<void>;
  createChannel: (data: CreateChannelRequest) => Promise<Channel>;
  updateChannel: (
    channelId: string,
    data: UpdateChannelRequest
  ) => Promise<void>;
  deleteChannel: (channelId: string) => Promise<void>;
  setCurrentChannel: (channel: Channel | null) => void;
  clearError: () => void;
}

export const useChannelStore = create<ChannelState>((set) => ({
  channels: [],
  currentChannel: null,
  total: 0,
  page: 1,
  limit: 20,
  loading: false,
  error: null,

  fetchChannels: async (params) => {
    set({ loading: true, error: null });
    try {
      const result: ChannelListResponse = await channelApi.listChannels(params);
      set({
        channels: result.channels,
        total: result.total,
        page: result.page,
        limit: result.limit,
        loading: false,
      });
    } catch (err) {
      set({
        loading: false,
        error: (err as { message?: string })?.message || 'Failed to fetch channels',
      });
    }
  },

  fetchChannel: async (channelId, password) => {
    set({ loading: true, error: null });
    try {
      const channel = await channelApi.getChannel(channelId, password);
      set({ currentChannel: channel, loading: false });
    } catch (err) {
      set({
        loading: false,
        error: (err as { message?: string })?.message || 'Failed to fetch channel',
      });
      throw err;
    }
  },

  createChannel: async (data) => {
    set({ loading: true, error: null });
    try {
      const channel = await channelApi.createChannel(data);
      set((state) => ({
        channels: [channel, ...state.channels],
        total: state.total + 1,
        currentChannel: channel,
        loading: false,
      }));
      return channel;
    } catch (err) {
      set({
        loading: false,
        error: (err as { message?: string })?.message || 'Failed to create channel',
      });
      throw err;
    }
  },

  updateChannel: async (channelId, data) => {
    set({ loading: true, error: null });
    try {
      const updated = await channelApi.updateChannel(channelId, data);
      set((state) => ({
        channels: state.channels.map((c) =>
          c.id === channelId ? updated : c
        ),
        currentChannel:
          state.currentChannel?.id === channelId
            ? updated
            : state.currentChannel,
        loading: false,
      }));
    } catch (err) {
      set({
        loading: false,
        error: (err as { message?: string })?.message || 'Failed to update channel',
      });
    }
  },

  deleteChannel: async (channelId) => {
    set({ loading: true, error: null });
    try {
      await channelApi.deleteChannel(channelId);
      set((state) => ({
        channels: state.channels.filter((c) => c.id !== channelId),
        total: state.total - 1,
        currentChannel:
          state.currentChannel?.id === channelId
            ? null
            : state.currentChannel,
        loading: false,
      }));
    } catch (err) {
      set({
        loading: false,
        error: (err as { message?: string })?.message || 'Failed to delete channel',
      });
    }
  },

  setCurrentChannel: (channel) => {
    set({ currentChannel: channel });
  },

  clearError: () => {
    set({ error: null });
  },
}));
