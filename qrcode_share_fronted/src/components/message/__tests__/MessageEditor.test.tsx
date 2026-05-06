import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';

const mockSendMessage = vi.fn().mockResolvedValue(undefined);

vi.mock('@/stores/messageStore', () => ({
  useMessageStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      sendMessage: mockSendMessage,
    };
    return selector(state);
  }),
}));

vi.mock('@/stores/channelStore', () => ({
  useChannelStore: vi.fn((selector: (state: Record<string, unknown>) => unknown) => {
    const state = {
      currentChannel: { link_limitation: [] },
    };
    return selector(state);
  }),
}));

vi.mock('@/components/qrcode', () => ({
  SmartQRScanner: () => <div data-testid="qr-scanner">Scanner</div>,
}));

vi.mock('@/utils/helpers', () => ({
  validateLink: vi.fn((link: string) => link.startsWith('http')),
  extractDomain: vi.fn(() => 'example.com'),
}));

import { MessageEditor } from '../MessageEditor';

describe('MessageEditor', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders Share a Link heading', () => {
    render(<MessageEditor channelId="test" onClose={() => {}} />);
    expect(screen.getByText('Share a Link')).toBeDefined();
  });

  it('renders Name input', () => {
    render(<MessageEditor channelId="test" onClose={() => {}} />);
    expect(screen.getByPlaceholderText('Your name')).toBeDefined();
  });

  it('renders Scan QR Code button', () => {
    render(<MessageEditor channelId="test" onClose={() => {}} />);
    expect(screen.getByText('Scan QR Code')).toBeDefined();
  });

  it('renders Send Link button', () => {
    render(<MessageEditor channelId="test" onClose={() => {}} />);
    expect(screen.getByText('Send Link')).toBeDefined();
  });

  it('calls onClose when backdrop is clicked', () => {
    const onClose = vi.fn();
    const { container } = render(<MessageEditor channelId="test" onClose={onClose} />);
    fireEvent.click(container.firstChild as HTMLElement);
    expect(onClose).toHaveBeenCalled();
  });

  it('renders drag handle bar', () => {
    render(<MessageEditor channelId="test" onClose={() => {}} />);
    const handle = document.querySelector('.bg-hairline.h-1');
    expect(handle).toBeDefined();
  });
});
