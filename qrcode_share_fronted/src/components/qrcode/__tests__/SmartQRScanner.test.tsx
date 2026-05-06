import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';

vi.mock('@/utils/wechat', () => ({
  isWechatBrowser: vi.fn(() => false),
  isWechatSdkAvailable: vi.fn(() => Promise.resolve(false)),
}));

vi.mock('@/components/qrcode/QRScanner', () => ({
  QRScanner: () => <div data-testid="qr-scanner">QRScanner</div>,
}));

vi.mock('@/components/wechat/WechatScanner', () => ({
  WechatScanner: () => <div data-testid="wechat-scanner">WechatScanner</div>,
}));

import { isWechatBrowser, isWechatSdkAvailable } from '@/utils/wechat';
import { SmartQRScanner } from '../SmartQRScanner';

describe('SmartQRScanner', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('renders QRScanner in non-WeChat browser', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(false);

    render(<SmartQRScanner onScan={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByTestId('qr-scanner')).toBeDefined();
    });
  });

  it('renders QRScanner when WeChat SDK is not available', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWechatSdkAvailable).mockResolvedValue(false);

    render(<SmartQRScanner onScan={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByTestId('qr-scanner')).toBeDefined();
    });
    expect(screen.queryByTestId('wechat-scanner')).toBeNull();
  });

  it('renders WechatScanner when WeChat SDK is available', async () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWechatSdkAvailable).mockResolvedValue(true);

    render(<SmartQRScanner onScan={vi.fn()} />);

    await waitFor(() => {
      expect(screen.getByTestId('wechat-scanner')).toBeDefined();
    });
    expect(screen.queryByTestId('qr-scanner')).toBeNull();
  });

  it('renders nothing while checking WeChat SDK availability', () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    vi.mocked(isWechatSdkAvailable).mockReturnValue(new Promise(() => {}));

    const { container } = render(<SmartQRScanner onScan={vi.fn()} />);
    expect(container.innerHTML).toBe('');
  });
});
