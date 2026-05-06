import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useMessageStore } from '../messageStore';
import * as messageApi from '@/api/messages';
import type { Message } from '@/types';

vi.mock('@/api/messages');

const mockMessage: Message = {
  id: 'msg_001',
  name: 'Test Message',
  link: 'https://example.com',
  link_domain: 'example.com',
  expire_at: '2099-12-31T23:59:59Z',
  created_at: '2024-01-01T00:00:00Z',
};

describe('MessageStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useMessageStore.setState({
      messages: [],
      hasMore: false,
      loading: false,
      error: null,
      channelPassword: undefined,
    });
  });

  describe('fetchMessages', () => {
    it('should fetch messages and update state', async () => {
      vi.mocked(messageApi.getMessages).mockResolvedValueOnce({
        messages: [mockMessage],
        has_more: false,
      });

      await useMessageStore.getState().fetchMessages('ch_001');

      const state = useMessageStore.getState();
      expect(state.messages).toHaveLength(1);
      expect(state.messages[0].id).toBe('msg_001');
      expect(state.hasMore).toBe(false);
      expect(state.loading).toBe(false);
    });

    it('should append messages when using cursor', async () => {
      useMessageStore.setState({ messages: [mockMessage] });

      const olderMessage: Message = {
        ...mockMessage,
        id: 'msg_002',
        name: 'Older Message',
      };
      vi.mocked(messageApi.getMessages).mockResolvedValueOnce({
        messages: [olderMessage],
        has_more: true,
      });

      await useMessageStore
        .getState()
        .fetchMessages('ch_001', { cursor: 'msg_001' });

      const state = useMessageStore.getState();
      expect(state.messages).toHaveLength(2);
      expect(state.hasMore).toBe(true);
    });

    it('should set error on failure', async () => {
      vi.mocked(messageApi.getMessages).mockRejectedValueOnce({
        message: 'Network error',
      });

      await useMessageStore.getState().fetchMessages('ch_001');

      expect(useMessageStore.getState().error).toBe('Network error');
    });
  });

  describe('sendMessage', () => {
    it('should add message to the beginning of the list', async () => {
      vi.mocked(messageApi.sendMessage).mockResolvedValueOnce(mockMessage);

      const result = await useMessageStore.getState().sendMessage('ch_001', {
        name: 'Test Message',
        link: 'https://example.com',
        expire_seconds: 3600,
      });

      const state = useMessageStore.getState();
      expect(state.messages[0].id).toBe('msg_001');
      expect(result.id).toBe('msg_001');
    });

    it('should not duplicate message if already added via WebSocket', async () => {
      useMessageStore.setState({ messages: [mockMessage] });

      vi.mocked(messageApi.sendMessage).mockResolvedValueOnce(mockMessage);

      await useMessageStore.getState().sendMessage('ch_001', {
        name: 'Test Message',
        link: 'https://example.com',
        expire_seconds: 3600,
      });

      const state = useMessageStore.getState();
      expect(state.messages).toHaveLength(1);
    });
  });

  describe('addMessage', () => {
    it('should add message from WebSocket', () => {
      const result = useMessageStore.getState().addMessage(mockMessage);

      expect(useMessageStore.getState().messages).toHaveLength(1);
      expect(result).toBe(true);
    });

    it('should not add duplicate messages', () => {
      useMessageStore.setState({ messages: [mockMessage] });
      const result = useMessageStore.getState().addMessage(mockMessage);

      expect(useMessageStore.getState().messages).toHaveLength(1);
      expect(result).toBe(false);
    });

    it('should reject invalid messages', () => {
      const result = useMessageStore.getState().addMessage(null as unknown as Message);
      expect(result).toBe(false);
    });
  });

  describe('removeExpiredMessages', () => {
    it('should remove expired messages', () => {
      const expiredMessage: Message = {
        ...mockMessage,
        id: 'msg_expired',
        expire_at: '2020-01-01T00:00:00Z',
      };
      useMessageStore.setState({ messages: [mockMessage, expiredMessage] });

      useMessageStore.getState().removeExpiredMessages();

      const state = useMessageStore.getState();
      expect(state.messages).toHaveLength(1);
      expect(state.messages[0].id).toBe('msg_001');
    });

    it('should keep messages without expire_at', () => {
      const noExpireMessage: Message = {
        ...mockMessage,
        id: 'msg_no_expire',
        expire_at: '',
      };
      useMessageStore.setState({ messages: [noExpireMessage] });

      useMessageStore.getState().removeExpiredMessages();

      expect(useMessageStore.getState().messages).toHaveLength(1);
    });
  });

  describe('clearMessages', () => {
    it('should clear all messages', () => {
      useMessageStore.setState({ messages: [mockMessage], hasMore: true });
      useMessageStore.getState().clearMessages();

      const state = useMessageStore.getState();
      expect(state.messages).toHaveLength(0);
      expect(state.hasMore).toBe(false);
    });
  });

  describe('clearError', () => {
    it('should clear error state', () => {
      useMessageStore.setState({ error: 'Some error' });
      useMessageStore.getState().clearError();

      expect(useMessageStore.getState().error).toBeNull();
    });
  });
});
