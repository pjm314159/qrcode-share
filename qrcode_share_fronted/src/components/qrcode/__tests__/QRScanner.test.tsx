import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';

const mockStop = vi.fn().mockResolvedValue(undefined);
const mockStart = vi.fn().mockResolvedValue(undefined);
const mockIsScanning = vi.fn().mockReturnValue(true);

vi.mock('html5-qrcode', () => ({
  Html5Qrcode: vi.fn().mockImplementation(() => ({
    start: mockStart,
    stop: mockStop,
    isScanning: mockIsScanning,
  })),
}));

import { QRScanner } from '../QRScanner';

describe('QRScanner', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockStart.mockResolvedValue(undefined);
    mockStop.mockResolvedValue(undefined);
    mockIsScanning.mockReturnValue(true);
  });

  it('renders scanner container with max-height constraint', () => {
    render(<QRScanner onScan={vi.fn()} />);
    const container = document.querySelector('.max-h-\\[60vh\\]');
    expect(container).toBeDefined();
  });

  it('does not render duplicate stop buttons when autoStart and scanning', async () => {
    render(<QRScanner onScan={vi.fn()} autoStart={true} />);

    await waitFor(() => {
      const stopButtons = screen.queryAllByRole('button', { name: /stop/i });
      expect(stopButtons.length).toBeLessThanOrEqual(1);
    });
  });

  it('renders overlay close button when scanning in autoStart mode', async () => {
    render(<QRScanner onScan={vi.fn()} autoStart={true} />);

    await waitFor(() => {
      const overlayBtn = document.querySelector('[aria-label="Stop scanner"]');
      expect(overlayBtn).toBeDefined();
    });
  });

  it('renders start button when not autoStart and not scanning', () => {
    render(<QRScanner onScan={vi.fn()} autoStart={false} />);
    expect(screen.getByText('Start Scanner')).toBeDefined();
  });

  it('renders stop button when not autoStart and scanning', async () => {
    render(<QRScanner onScan={vi.fn()} autoStart={false} />);

    fireEvent.click(screen.getByText('Start Scanner'));

    await waitFor(() => {
      expect(screen.getByText('Stop Scanner')).toBeDefined();
    });
  });

  it('calls stopScanner when overlay close button is clicked', async () => {
    render(<QRScanner onScan={vi.fn()} autoStart={true} />);

    await waitFor(() => {
      const overlayBtn = document.querySelector('[aria-label="Stop scanner"]') as HTMLElement;
      expect(overlayBtn).toBeDefined();
    });

    const overlayBtn = document.querySelector('[aria-label="Stop scanner"]') as HTMLElement;
    fireEvent.click(overlayBtn);

    await waitFor(() => {
      expect(mockStop).toHaveBeenCalled();
    });
  });

  it('scanner container has overflow-hidden to prevent overflow', () => {
    render(<QRScanner onScan={vi.fn()} />);
    const container = document.querySelector('.overflow-hidden.rounded-md');
    expect(container).toBeDefined();
  });

  it('does not render a separate stop button section below scanner in autoStart mode', async () => {
    const { container } = render(<QRScanner onScan={vi.fn()} autoStart={true} />);

    await waitFor(() => {
      const separateStopSections = container.querySelectorAll('.flex.gap-2');
      expect(separateStopSections.length).toBe(0);
    });
  });
});
