import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { ChannelPage } from '../ChannelPage';

const mockFetchChannel = vi.fn().mockResolvedValue(undefined);
const mockDeleteChannel = vi.fn().mockResolvedValue(undefined);
const mockConnect = vi.fn();
const mockDisconnect = vi.fn();
const mockSendMessage = vi.fn().mockResolvedValue(undefined);
const mockFetchMessages = vi.fn().mockResolvedValue(undefined);
const mockClearMessages = vi.fn();
const mockRemoveExpiredMessages = vi.fn();
const mockSetChannelPassword = vi.fn();
const mockResetChannel = vi.fn();

vi.mock('@/stores/channelStore', () => ({
  useChannelStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      currentChannel: {
        id: 'test-channel',
        name: 'Test Channel',
        has_password: false,
        link_limitation: [],
        channel_type: 'classroom',
        location: 'Room 301',
        teacher: 'Prof. Smith',
        created_at: '2024-01-01T00:00:00Z',
        subscriber_count: 3,
        message_count: 10,
      },
      loading: false,
      error: null,
      fetchChannel: mockFetchChannel,
      deleteChannel: mockDeleteChannel,
    };
    return selector(state);
  }),
}));

vi.mock('@/stores/connectionStore', () => ({
  useConnectionStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      client: null,
      state: 'connected',
      subscriberCount: 3,
      connect: mockConnect,
      disconnect: mockDisconnect,
      setSubscriberCount: vi.fn(),
    };
    return selector(state);
  }),
}));

vi.mock('@/stores/messageStore', () => ({
  useMessageStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      messages: [],
      hasMore: false,
      loading: false,
      error: null,
      sendMessage: mockSendMessage,
      fetchMessages: mockFetchMessages,
      clearMessages: mockClearMessages,
      removeExpiredMessages: mockRemoveExpiredMessages,
      setChannelPassword: mockSetChannelPassword,
      resetChannel: mockResetChannel,
    };
    return selector(state);
  }),
}));

vi.mock('@/stores/settingsStore', () => ({
  useSettingsStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      autoOpenLinks: false,
      autoOpenReceivedLinks: false,
      setAutoOpenLinks: vi.fn(),
      setAutoOpenReceivedLinks: vi.fn(),
    };
    return selector(state);
  }),
}));

vi.mock('@/hooks', () => ({
  useWebSocket: vi.fn(),
}));

function renderChannelPage() {
  return render(
    <BrowserRouter>
      <Routes>
        <Route path="/channel/:channelId" element={<ChannelPage />} />
      </Routes>
    </BrowserRouter>
  );
}

describe('ChannelPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    window.history.pushState({}, '', '/channel/test-channel');
  });

  it('renders channel name in heading', () => {
    renderChannelPage();
    expect(screen.getByRole('heading', { level: 1, name: 'Test Channel' })).toBeDefined();
  });

  it('renders connection status', () => {
    renderChannelPage();
    expect(screen.getByText(/Connected/)).toBeDefined();
  });

  it('renders subscriber count', () => {
    renderChannelPage();
    expect(screen.getByText(/3 online/)).toBeDefined();
  });

  it('renders scan and share button', () => {
    renderChannelPage();
    expect(screen.getByText(/Scan & Share/)).toBeDefined();
  });

  it('renders paste link button', () => {
    renderChannelPage();
    expect(screen.getByText(/Paste Link/)).toBeDefined();
  });

  it('renders back button without text label', () => {
    renderChannelPage();
    const backButtons = screen.getAllByRole('button');
    const backButton = backButtons.find(btn => btn.querySelector('svg'));
    expect(backButton).toBeDefined();
  });

  it('calls fetchChannel on mount', () => {
    renderChannelPage();
    expect(mockFetchChannel).toHaveBeenCalledWith('test-channel', undefined);
  });
});
