import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';

const mockSetAutoOpenLinks = vi.fn();
const mockSetAutoOpenReceivedLinks = vi.fn();

vi.mock('@/stores/settingsStore', () => ({
  useSettingsStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      autoOpenLinks: false,
      autoOpenReceivedLinks: false,
      setAutoOpenLinks: mockSetAutoOpenLinks,
      setAutoOpenReceivedLinks: mockSetAutoOpenReceivedLinks,
    };
    return selector(state);
  }),
}));

import { ChannelSettingsPanel } from '../ChannelSettingsPanel';

describe('ChannelSettingsPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders Settings heading', () => {
    render(<ChannelSettingsPanel channelId="test-channel" onClose={() => {}} />);
    expect(screen.getByText('Settings')).toBeDefined();
  });

  it('renders Channel ID', () => {
    render(<ChannelSettingsPanel channelId="test-channel" onClose={() => {}} />);
    expect(screen.getByText('test-channel')).toBeDefined();
  });

  it('renders channel name when provided', () => {
    render(<ChannelSettingsPanel channelId="test-channel" channelName="My Channel" onClose={() => {}} />);
    expect(screen.getByText('My Channel')).toBeDefined();
  });

  it('renders Channel Info section', () => {
    render(<ChannelSettingsPanel channelId="test-channel" onClose={() => {}} />);
    expect(screen.getByText('Channel Info')).toBeDefined();
  });

  it('renders Link Behavior section', () => {
    render(<ChannelSettingsPanel channelId="test-channel" onClose={() => {}} />);
    expect(screen.getByText('Link Behavior')).toBeDefined();
  });

  it('renders Auto-open on click toggle', () => {
    render(<ChannelSettingsPanel channelId="test-channel" onClose={() => {}} />);
    expect(screen.getByText('Auto-open on click')).toBeDefined();
  });

  it('renders Auto-open on receive toggle', () => {
    render(<ChannelSettingsPanel channelId="test-channel" onClose={() => {}} />);
    expect(screen.getByText('Auto-open on receive')).toBeDefined();
  });

  it('renders Copy Link button', () => {
    render(<ChannelSettingsPanel channelId="test-channel" onClose={() => {}} />);
    expect(screen.getByText('Copy Link')).toBeDefined();
  });

  it('calls onClose when backdrop is clicked', () => {
    const onClose = vi.fn();
    const { container } = render(<ChannelSettingsPanel channelId="test-channel" onClose={onClose} />);
    fireEvent.click(container.firstChild as HTMLElement);
    expect(onClose).toHaveBeenCalled();
  });
});
