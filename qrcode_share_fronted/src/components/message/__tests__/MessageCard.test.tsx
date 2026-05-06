import React from 'react';
import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MessageCard } from '../MessageCard';
import type { Message } from '@/types';

const futureDate = new Date(Date.now() + 3600_000).toISOString();
const pastDate = new Date(Date.now() - 3600_000).toISOString();

const baseMessage: Message = {
  id: 'msg_001',
  name: 'Test Message',
  link: 'https://example.com/page',
  link_domain: 'example.com',
  expire_at: futureDate,
  created_at: '2024-01-01T00:00:00Z',
};

describe('MessageCard', () => {
  it('renders message name', () => {
    render(<MessageCard message={baseMessage} />);
    expect(screen.getByText('Test Message')).toBeDefined();
  });

  it('renders link', () => {
    render(<MessageCard message={baseMessage} />);
    expect(screen.getByText('https://example.com/page')).toBeDefined();
  });

  it('renders domain badge', () => {
    render(<MessageCard message={baseMessage} />);
    expect(screen.getByText('example.com')).toBeDefined();
  });

  it('shows expired for past expiry', () => {
    const expiredMessage: Message = { ...baseMessage, expire_at: pastDate };
    render(<MessageCard message={expiredMessage} />);
    expect(screen.getByText('Expired')).toBeDefined();
  });

  it('link opens in new tab', () => {
    render(<MessageCard message={baseMessage} />);
    const link = screen.getByText('https://example.com/page');
    expect(link.getAttribute('target')).toBe('_blank');
    expect(link.getAttribute('rel')).toBe('noopener noreferrer');
  });
});
