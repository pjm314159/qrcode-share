import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { HomePage } from '../HomePage';
import { NotFoundPage } from '../NotFoundPage';
import { ChannelListPage } from '../ChannelListPage';

vi.mock('@/stores/channelStore', () => ({
  useChannelStore: vi.fn((selector) => {
    const state = {
      channels: [],
      total: 0,
      page: 1,
      limit: 20,
      loading: false,
      error: null,
      fetchChannels: vi.fn(),
      fetchChannel: vi.fn(),
      createChannel: vi.fn(),
      updateChannel: vi.fn(),
      deleteChannel: vi.fn(),
      setCurrentChannel: vi.fn(),
      clearError: vi.fn(),
    };
    return selector(state);
  }),
}));

vi.mock('@/stores/connectionStore', () => ({
  useConnectionStore: vi.fn((selector) => {
    const state = {
      client: null,
      state: 'disconnected',
      subscriberCount: 0,
      connect: vi.fn(),
      disconnect: vi.fn(),
      setSubscriberCount: vi.fn(),
    };
    return selector(state);
  }),
}));

function renderWithRouter(ui: React.ReactElement) {
  return render(<BrowserRouter>{ui}</BrowserRouter>);
}

describe('HomePage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders app title', () => {
    renderWithRouter(<HomePage />);
    expect(screen.getByText('QRcode Share')).toBeDefined();
  });

  it('renders title accent line', () => {
    renderWithRouter(<HomePage />);
    const accentLine = document.querySelector('.bg-brand-pink');
    expect(accentLine).toBeDefined();
  });

  it('renders hero illustration', () => {
    renderWithRouter(<HomePage />);
    expect(screen.getByAltText('QRcode Share illustration')).toBeDefined();
  });

  it('renders decorative blobs', () => {
    renderWithRouter(<HomePage />);
    const blobs = document.querySelectorAll('.pointer-events-none.rounded-full');
    expect(blobs.length).toBeGreaterThanOrEqual(2);
  });

  it('renders join channel form', () => {
    renderWithRouter(<HomePage />);
    expect(screen.getByText('Join a Channel')).toBeDefined();
  });

  it('renders create channel form', () => {
    renderWithRouter(<HomePage />);
    expect(screen.getByText('Create a Channel')).toBeDefined();
  });

  it('renders feature cards', () => {
    renderWithRouter(<HomePage />);
    expect(screen.getByText('Scan')).toBeDefined();
    expect(screen.getByText('Share')).toBeDefined();
    expect(screen.getByText('Open')).toBeDefined();
  });

  it('renders browse channels link', () => {
    renderWithRouter(<HomePage />);
    expect(screen.getByText(/Browse all channels/)).toBeDefined();
  });
});

describe('NotFoundPage', () => {
  it('renders page not found heading', () => {
    renderWithRouter(<NotFoundPage />);
    expect(screen.getByText('Page Not Found')).toBeDefined();
  });

  it('renders description text', () => {
    renderWithRouter(<NotFoundPage />);
    expect(screen.getByText(/does not exist/)).toBeDefined();
  });

  it('renders back to home button', () => {
    renderWithRouter(<NotFoundPage />);
    expect(screen.getByText('Back to Home')).toBeDefined();
  });
});

describe('ChannelListPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders page heading', () => {
    renderWithRouter(<ChannelListPage />);
    expect(screen.getByText('Channels')).toBeDefined();
  });

  it('renders search component', () => {
    renderWithRouter(<ChannelListPage />);
    expect(screen.getByPlaceholderText(/Search channels/)).toBeDefined();
  });

  it('renders header with accent line', () => {
    renderWithRouter(<ChannelListPage />);
    const accentLine = document.querySelector('.bg-brand-pink');
    expect(accentLine).toBeTruthy();
  });

  it('renders header with subtitle', () => {
    renderWithRouter(<ChannelListPage />);
    expect(screen.getByText(/Browse and join/)).toBeDefined();
  });

  it('renders header section with gradient background', () => {
    renderWithRouter(<ChannelListPage />);
    const headerSection = document.querySelector('.bg-gradient-to-b');
    expect(headerSection).toBeTruthy();
  });
});
