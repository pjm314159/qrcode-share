import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';
import { ChannelCard } from '../ChannelCard';
import type { Channel } from '@/types';

const baseChannel: Channel = {
  id: 'ch_001',
  name: 'Test Channel',
  has_password: false,
  link_limitation: undefined,
  channel_type: undefined,
  location: undefined,
  teacher: undefined,
  created_at: '2024-01-01T00:00:00Z',
  subscriber_count: 0,
  message_count: 5,
};

describe('ChannelCard', () => {
  it('renders channel name', () => {
    render(<ChannelCard channel={baseChannel} onClick={vi.fn()} />);
    expect(screen.getByText('Test Channel')).toBeDefined();
  });

  it('shows message count', () => {
    render(<ChannelCard channel={baseChannel} onClick={vi.fn()} />);
    expect(screen.getByText(/5 msg/)).toBeDefined();
  });

  it('shows locked badge when channel has password', () => {
    const protectedChannel: Channel = {
      ...baseChannel,
      has_password: true,
    };
    render(<ChannelCard channel={protectedChannel} onClick={vi.fn()} />);
    expect(screen.getByText('Locked')).toBeDefined();
  });

  it('does not show locked badge for public channels', () => {
    render(<ChannelCard channel={baseChannel} onClick={vi.fn()} />);
    expect(screen.queryByText('Locked')).toBeNull();
  });

  it('shows teacher when present', () => {
    const channelWithTeacher: Channel = {
      ...baseChannel,
      teacher: 'Prof. Smith',
    };
    render(<ChannelCard channel={channelWithTeacher} onClick={vi.fn()} />);
    expect(screen.getByText(/Prof\. Smith/)).toBeDefined();
  });

  it('calls onClick when clicked', () => {
    const onClick = vi.fn();
    render(<ChannelCard channel={baseChannel} onClick={onClick} />);
    fireEvent.click(screen.getByText('Test Channel'));
    expect(onClick).toHaveBeenCalledWith(baseChannel);
  });
});
