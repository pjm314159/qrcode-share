import { create } from 'zustand';
import type { Message, CreateMessageRequest } from '@/types';
import * as messageApi from '@/api/messages';
import { useSettingsStore } from '@/stores/settingsStore';

interface MessageState {
  messages: Message[];
  hasMore: boolean;
  loading: boolean;
  error: string | null;
  channelPassword: string | undefined;

  setChannelPassword: (password: string | undefined) => void;
  fetchMessages: (
    channelId: string,
    params?: { cursor?: string; limit?: number }
  ) => Promise<void>;
  sendMessage: (
    channelId: string,
    data: CreateMessageRequest
  ) => Promise<Message>;
  addMessage: (message: Message) => boolean;
  removeExpiredMessages: () => void;
  clearMessages: () => void;
  resetChannel: () => void;
  clearError: () => void;
}

export const useMessageStore = create<MessageState>((set, get) => ({
  messages: [],
  hasMore: false,
  loading: false,
  error: null,
  channelPassword: undefined,

  setChannelPassword: (password) => {
    set({ channelPassword: password });
  },

  fetchMessages: async (channelId, params) => {
    set({ loading: true, error: null });
    try {
      const result = await messageApi.getMessages(channelId, params, get().channelPassword);
      set({
        messages: params?.cursor
          ? [...get().messages, ...result.messages]
          : result.messages,
        hasMore: result.has_more,
        loading: false,
      });
    } catch (err) {
      set({
        loading: false,
        error: (err as { message?: string })?.message || 'Failed to fetch messages',
      });
    }
  },

  sendMessage: async (channelId, data) => {
    set({ loading: true, error: null });
    try {
      const message = await messageApi.sendMessage(channelId, data, get().channelPassword);
      const existing = get().messages;
      if (existing.some((m) => m?.id === message.id)) {
        set({ loading: false });
      } else {
        set({ messages: [message, ...existing], loading: false });
      }
      if (useSettingsStore.getState().autoOpenReceivedLinks && message.link) {
        window.location.href = message.link;
      }
      return message;
    } catch (err) {
      set({
        loading: false,
        error: (err as { message?: string })?.message || 'Failed to send message',
      });
      throw err;
    }
  },

  addMessage: (message) => {
    if (!message || !message.id) return false;
    const existing = get().messages;
    if (existing.some((m) => m?.id === message.id)) return false;
    set({ messages: [message, ...existing] });
    return true;
  },

  removeExpiredMessages: () => {
    const now = new Date().toISOString();
    set((state) => ({
      messages: state.messages.filter((m) => !m?.expire_at || m.expire_at > now),
    }));
  },

  clearMessages: () => {
    set({ messages: [], hasMore: false });
  },

  resetChannel: () => {
    set({ messages: [], hasMore: false, channelPassword: undefined });
  },

  clearError: () => {
    set({ error: null });
  },
}));
