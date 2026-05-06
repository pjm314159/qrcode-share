import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as channelApi from '../channels';
import { apiClient } from '../client';
import type { Channel, ApiResponse } from '@/types';

vi.mock('../client', () => ({
  apiClient: {
    post: vi.fn(),
    get: vi.fn(),
    patch: vi.fn(),
    delete: vi.fn(),
  },
  API_BASE_URL: 'http://localhost:3000',
}));

const mockChannel: Channel = {
  id: 'test1234',
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

function makeSuccessResponse<T>(data: T): ApiResponse<T> {
  return { success: true, data, error: null };
}

describe('Channel API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('createChannel', () => {
    it('should create a channel and return it', async () => {
      const mockResponse = makeSuccessResponse(mockChannel);
      vi.mocked(apiClient.post).mockResolvedValueOnce({ data: mockResponse });

      const result = await channelApi.createChannel({ name: 'Test Channel' });

      expect(apiClient.post).toHaveBeenCalledWith('/api/channels', {
        name: 'Test Channel',
      });
      expect(result).toEqual(mockChannel);
    });
  });

  describe('getChannel', () => {
    it('should fetch a channel by ID', async () => {
      const mockResponse = makeSuccessResponse(mockChannel);
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });

      const result = await channelApi.getChannel('test1234');

      expect(apiClient.get).toHaveBeenCalledWith('/api/channels/test1234', {
        headers: {},
      });
      expect(result).toEqual(mockChannel);
    });

    it('should send password header when provided', async () => {
      const mockResponse = makeSuccessResponse(mockChannel);
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });

      await channelApi.getChannel('test1234', 'secret123');

      expect(apiClient.get).toHaveBeenCalledWith('/api/channels/test1234', {
        headers: { 'X-Channel-Password': 'secret123' },
      });
    });
  });

  describe('listChannels', () => {
    it('should list channels with default params', async () => {
      const listResponse = {
        channels: [mockChannel],
        total: 1,
        page: 1,
        limit: 20,
      };
      const mockResponse = makeSuccessResponse(listResponse);
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });

      const result = await channelApi.listChannels();

      expect(apiClient.get).toHaveBeenCalledWith('/api/channels', {
        params: undefined,
      });
      expect(result.channels).toHaveLength(1);
    });

    it('should list channels with search params', async () => {
      const listResponse = {
        channels: [],
        total: 0,
        page: 1,
        limit: 20,
      };
      const mockResponse = makeSuccessResponse(listResponse);
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });

      await channelApi.listChannels({ search: 'test', page: 2 });

      expect(apiClient.get).toHaveBeenCalledWith('/api/channels', {
        params: { search: 'test', page: 2 },
      });
    });
  });

  describe('updateChannel', () => {
    it('should update a channel', async () => {
      const updated = { ...mockChannel, name: 'Updated' };
      const mockResponse = makeSuccessResponse(updated);
      vi.mocked(apiClient.patch).mockResolvedValueOnce({ data: mockResponse });

      const result = await channelApi.updateChannel('test1234', {
        name: 'Updated',
      });

      expect(apiClient.patch).toHaveBeenCalledWith('/api/channels/test1234', {
        name: 'Updated',
      });
      expect(result.name).toBe('Updated');
    });
  });

  describe('deleteChannel', () => {
    it('should delete a channel', async () => {
      vi.mocked(apiClient.delete).mockResolvedValueOnce({ status: 200 });

      await channelApi.deleteChannel('test1234');

      expect(apiClient.delete).toHaveBeenCalledWith('/api/channels/test1234');
    });
  });
});
