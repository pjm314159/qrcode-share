import { describe, it, expect, vi, beforeEach } from 'vitest';
import * as messageApi from '../messages';
import { apiClient } from '../client';
import type { Message, ApiResponse } from '@/types';

vi.mock('../client', () => ({
  apiClient: {
    post: vi.fn(),
    get: vi.fn(),
  },
  API_BASE_URL: 'http://localhost:3000',
}));

const mockMessage: Message = {
  id: 'msg_001',
  name: 'Test Message',
  link: 'https://example.com',
  link_domain: 'example.com',
  expire_at: '2024-12-31T23:59:59Z',
  created_at: '2024-01-01T00:00:00Z',
};

function makeSuccessResponse<T>(data: T): ApiResponse<T> {
  return { success: true, data, error: null };
}

describe('Message API', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('sendMessage', () => {
    it('should send a message and return it', async () => {
      const mockResponse = makeSuccessResponse(mockMessage);
      vi.mocked(apiClient.post).mockResolvedValueOnce({ data: mockResponse });

      const result = await messageApi.sendMessage('ch_001', {
        name: 'Test Message',
        link: 'https://example.com',
        expire_seconds: 3600,
      });

      expect(apiClient.post).toHaveBeenCalledWith(
        '/api/channels/ch_001/messages',
        {
          name: 'Test Message',
          link: 'https://example.com',
          expire_seconds: 3600,
        },
        { headers: {} }
      );
      expect(result).toEqual(mockMessage);
    });

    it('should send password header when provided', async () => {
      const mockResponse = makeSuccessResponse(mockMessage);
      vi.mocked(apiClient.post).mockResolvedValueOnce({ data: mockResponse });

      await messageApi.sendMessage('ch_001', {
        name: 'Test Message',
        link: 'https://example.com',
        expire_seconds: 3600,
      }, 'secret');

      expect(apiClient.post).toHaveBeenCalledWith(
        '/api/channels/ch_001/messages',
        expect.any(Object),
        { headers: { 'X-Channel-Password': 'secret' } }
      );
    });
  });

  describe('getMessages', () => {
    it('should fetch messages for a channel', async () => {
      const listResponse = {
        messages: [mockMessage],
        has_more: false,
      };
      const mockResponse = makeSuccessResponse(listResponse);
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });

      const result = await messageApi.getMessages('ch_001');

      expect(apiClient.get).toHaveBeenCalledWith(
        '/api/channels/ch_001/messages',
        { params: undefined, headers: {} }
      );
      expect(result.messages).toHaveLength(1);
    });

    it('should fetch messages with cursor pagination', async () => {
      const listResponse = {
        messages: [],
        has_more: false,
      };
      const mockResponse = makeSuccessResponse(listResponse);
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });

      await messageApi.getMessages('ch_001', {
        cursor: 'msg_050',
        limit: 20,
      });

      expect(apiClient.get).toHaveBeenCalledWith(
        '/api/channels/ch_001/messages',
        { params: { cursor: 'msg_050', limit: 20 }, headers: {} }
      );
    });
  });

  describe('getMessage', () => {
    it('should fetch a single message', async () => {
      const mockResponse = makeSuccessResponse(mockMessage);
      vi.mocked(apiClient.get).mockResolvedValueOnce({ data: mockResponse });

      const result = await messageApi.getMessage('ch_001', 'msg_001');

      expect(apiClient.get).toHaveBeenCalledWith(
        '/api/channels/ch_001/messages/msg_001',
        { headers: {} }
      );
      expect(result.id).toBe('msg_001');
    });
  });
});
