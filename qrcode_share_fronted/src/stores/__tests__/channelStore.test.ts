import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useChannelStore } from '../channelStore';
import * as channelApi from '@/api/channels';
import type { Channel, ChannelListResponse } from '@/types';

vi.mock('@/api/channels');

const mockChannel: Channel = {
  id: 'ch_001',
  name: 'Test Channel',
  has_password: false,
  link_limitation: undefined,
  channel_type: undefined,
  location: undefined,
  teacher: undefined,
  created_at: '2024-01-01T00:00:00Z',
  subscriber_count: 0,
  message_count: 0,
};

describe('ChannelStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useChannelStore.setState({
      channels: [],
      currentChannel: null,
      total: 0,
      page: 1,
      limit: 20,
      loading: false,
      error: null,
    });
  });

  describe('fetchChannels', () => {
    it('should fetch channels and update state', async () => {
      const listResponse: ChannelListResponse = {
        channels: [mockChannel],
        total: 1,
        page: 1,
        limit: 20,
      };
      vi.mocked(channelApi.listChannels).mockResolvedValueOnce(listResponse);

      await useChannelStore.getState().fetchChannels();

      const state = useChannelStore.getState();
      expect(state.channels).toHaveLength(1);
      expect(state.channels[0].id).toBe('ch_001');
      expect(state.total).toBe(1);
      expect(state.loading).toBe(false);
    });

    it('should set error on failure', async () => {
      vi.mocked(channelApi.listChannels).mockRejectedValueOnce({
        message: 'Network error',
      });

      await useChannelStore.getState().fetchChannels();

      const state = useChannelStore.getState();
      expect(state.error).toBe('Network error');
      expect(state.loading).toBe(false);
    });
  });

  describe('createChannel', () => {
    it('should add new channel to the list', async () => {
      vi.mocked(channelApi.createChannel).mockResolvedValueOnce(mockChannel);

      const result = await useChannelStore
        .getState()
        .createChannel({ name: 'Test Channel' });

      const state = useChannelStore.getState();
      expect(state.channels).toHaveLength(1);
      expect(state.currentChannel?.id).toBe('ch_001');
      expect(state.total).toBe(1);
      expect(result.id).toBe('ch_001');
    });

    it('should set error on creation failure', async () => {
      vi.mocked(channelApi.createChannel).mockRejectedValueOnce({
        message: 'Create failed',
      });

      expect(
          useChannelStore.getState().createChannel({name: 'Test'})
      ).rejects.toBeDefined();

      const state = useChannelStore.getState();
      expect(state.error).toBe('Create failed');
    });
  });

  describe('deleteChannel', () => {
    it('should remove channel from the list', async () => {
      useChannelStore.setState({ channels: [mockChannel], total: 1 });
      vi.mocked(channelApi.deleteChannel).mockResolvedValueOnce(undefined);

      await useChannelStore.getState().deleteChannel('ch_001');

      const state = useChannelStore.getState();
      expect(state.channels).toHaveLength(0);
      expect(state.total).toBe(0);
    });
  });

  describe('setCurrentChannel', () => {
    it('should set current channel', () => {
      useChannelStore.getState().setCurrentChannel(mockChannel);

      expect(useChannelStore.getState().currentChannel?.id).toBe('ch_001');
    });

    it('should clear current channel with null', () => {
      useChannelStore.getState().setCurrentChannel(null);

      expect(useChannelStore.getState().currentChannel).toBeNull();
    });
  });

  describe('clearError', () => {
    it('should clear error state', () => {
      useChannelStore.setState({ error: 'Some error' });
      useChannelStore.getState().clearError();

      expect(useChannelStore.getState().error).toBeNull();
    });
  });
});
