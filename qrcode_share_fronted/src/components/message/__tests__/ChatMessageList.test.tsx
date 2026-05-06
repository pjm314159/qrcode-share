import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';

const mockFetchMessages = vi.fn().mockResolvedValue(undefined);
const mockClearMessages = vi.fn();
const mockRemoveExpiredMessages = vi.fn();

vi.mock('@/stores/messageStore', () => ({
  useMessageStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      messages: [],
      hasMore: false,
      loading: false,
      error: null,
      fetchMessages: mockFetchMessages,
      clearMessages: mockClearMessages,
      removeExpiredMessages: mockRemoveExpiredMessages,
    };
    return selector(state);
  }),
}));

vi.mock('@/hooks/useTimer', () => ({
  useInterval: vi.fn(),
}));

vi.mock('@/utils/helpers', () => ({
  extractDomain: vi.fn(() => 'example.com'),
  formatRemainingTime: vi.fn(() => '59:59'),
}));

import { ChatMessageList } from '../ChatMessageList';

describe('ChatMessageList', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders empty state when no messages', () => {
    render(<ChatMessageList channelId="test" onLinkClick={() => {}} ready={true} />);
    expect(screen.getByText('No messages yet')).toBeDefined();
  });

  it('renders empty state subtitle', () => {
    render(<ChatMessageList channelId="test" onLinkClick={() => {}} ready={true} />);
    expect(screen.getByText('Be the first to share a link!')).toBeDefined();
  });

  it('calls fetchMessages on mount when ready', () => {
    render(<ChatMessageList channelId="test" onLinkClick={() => {}} ready={true} />);
    expect(mockFetchMessages).toHaveBeenCalledWith('test');
  });

  it('does not call fetchMessages when not ready', () => {
    render(<ChatMessageList channelId="test" onLinkClick={() => {}} ready={false} />);
    expect(mockFetchMessages).not.toHaveBeenCalled();
  });

  it('calls clearMessages on unmount', () => {
    const { unmount } = render(<ChatMessageList channelId="test" onLinkClick={() => {}} ready={true} />);
    unmount();
    expect(mockClearMessages).toHaveBeenCalled();
  });
});
