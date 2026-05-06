import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { WechatScanner } from '../WechatScanner';

vi.mock('@/utils/wechat', () => ({
  isWechatBrowser: vi.fn(() => true),
  isWxInitialized: vi.fn(() => true),
  initWxSdk: vi.fn(() => Promise.resolve()),
  scanQRCode: vi.fn(() => Promise.resolve('https://example.com/scanned')),
}));

import { isWechatBrowser, isWxInitialized, initWxSdk, scanQRCode } from '@/utils/wechat';

describe('WechatScanner', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('shows initializing state when SDK is not ready', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWxInitialized).mockReturnValue(false);
    vi.mocked(initWxSdk).mockReturnValue(new Promise(() => {}));

    render(<WechatScanner onScan={vi.fn()} />);
    await waitFor(() => {
      expect(screen.getByText('Initializing WeChat scanner...')).toBeDefined();
    });
  });

  it('shows scan button when SDK is ready', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWxInitialized).mockReturnValue(true);

    render(<WechatScanner onScan={vi.fn()} />);
    await waitFor(() => {
      expect(screen.getByText('Scan QR Code')).toBeDefined();
    });
  });

  it('shows error when not in WeChat browser', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(false);

    render(<WechatScanner onScan={vi.fn()} />);
    await waitFor(() => {
      expect(screen.getByText('This component is only available in WeChat browser')).toBeDefined();
    });
  });

  it('calls onScan with scanned result', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWxInitialized).mockReturnValue(true);
    vi.mocked(scanQRCode).mockResolvedValue('https://example.com/qr-link');

    const onScan = vi.fn();
    render(<WechatScanner onScan={onScan} />);

    const button = await screen.findByRole('button', { name: /Scan QR Code/ });
    fireEvent.click(button);

    await waitFor(() => {
      expect(onScan).toHaveBeenCalledWith('https://example.com/qr-link');
    });
  });

  it('calls onError when scan fails', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWxInitialized).mockReturnValue(true);
    vi.mocked(scanQRCode).mockRejectedValue(new Error('Scan cancelled'));

    const onError = vi.fn();
    render(<WechatScanner onScan={vi.fn()} onError={onError} />);

    const button = await screen.findByRole('button', { name: /Scan QR Code/ });
    fireEvent.click(button);

    await waitFor(() => {
      expect(onError).toHaveBeenCalledWith('Scan cancelled');
    });
  });

  it('shows error when SDK initialization fails', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWxInitialized).mockReturnValue(false);
    vi.mocked(initWxSdk).mockRejectedValue(new Error('Network error'));

    render(<WechatScanner onScan={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText('Network error')).toBeDefined();
    });
  });

  it('shows retry button on initialization error in WeChat browser', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWxInitialized).mockReturnValue(false);
    vi.mocked(initWxSdk).mockRejectedValue(new Error('Network error'));

    render(<WechatScanner onScan={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByText('Retry Initialization')).toBeDefined();
    });
  });
});
