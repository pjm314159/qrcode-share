import React from 'react';
import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent } from '@testing-library/react';

vi.mock('@/hooks/useTimer', () => ({
  useInterval: vi.fn(),
}));

vi.mock('@/utils/helpers', () => ({
  extractDomain: vi.fn((link: string) => {
    try { return new URL(link).hostname; } catch { return ''; }
  }),
  formatRemainingTime: vi.fn((seconds: number) => {
    if (seconds <= 0) return 'Expired';
    const m = Math.floor(seconds / 60);
    const s = seconds % 60;
    return `${m}:${s.toString().padStart(2, '0')}`;
  }),
}));

import { ChatMessageCard } from '../ChatMessageCard';
import type { Message } from '@/types';

const futureMessage: Message = {
  id: '1',
  name: 'Test User',
  link: 'https://example.com/page',
  expire_at: new Date(Date.now() + 3600000).toISOString(),
  message_type: 'url',
  created_at: new Date().toISOString(),
};

const expiredMessage: Message = {
  id: '2',
  name: 'Old User',
  link: 'https://example.com/old',
  expire_at: new Date(Date.now() - 1000).toISOString(),
  message_type: 'url',
  created_at: new Date().toISOString(),
};

describe('ChatMessageCard', () => {
  it('renders message name', () => {
    render(<ChatMessageCard message={futureMessage} onLinkClick={() => {}} />);
    expect(screen.getByText('Test User')).toBeDefined();
  });

  it('renders message link', () => {
    render(<ChatMessageCard message={futureMessage} onLinkClick={() => {}} />);
    expect(screen.getByText('https://example.com/page')).toBeDefined();
  });

  it('renders message type badge', () => {
    render(<ChatMessageCard message={futureMessage} onLinkClick={() => {}} />);
    expect(screen.getByText('url')).toBeDefined();
  });

  it('calls onLinkClick when clicked', () => {
    const onClick = vi.fn();
    render(<ChatMessageCard message={futureMessage} onLinkClick={onClick} />);
    fireEvent.click(screen.getByText('Test User').closest('div')!);
    expect(onClick).toHaveBeenCalledWith('https://example.com/page');
  });

  it('does not call onLinkClick when expired', () => {
    const onClick = vi.fn();
    render(<ChatMessageCard message={expiredMessage} onLinkClick={onClick} />);
    fireEvent.click(screen.getByText('Old User').closest('div')!);
    expect(onClick).not.toHaveBeenCalled();
  });

  it('shows expired state', () => {
    const { container } = render(<ChatMessageCard message={expiredMessage} onLinkClick={() => {}} />);
    const card = container.firstChild as HTMLElement;
    expect(card.className).toContain('opacity-40');
  });
});
