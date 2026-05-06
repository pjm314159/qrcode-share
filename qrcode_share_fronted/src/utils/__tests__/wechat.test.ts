import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

describe('wechat utils', () => {
  const originalUserAgent = navigator.userAgent;

  beforeEach(() => {
    vi.resetModules();
  });

  afterEach(() => {
    Object.defineProperty(navigator, 'userAgent', {
      value: originalUserAgent,
      configurable: true,
    });
    vi.restoreAllMocks();
  });

  function mockUserAgent(ua: string) {
    Object.defineProperty(navigator, 'userAgent', {
      value: ua,
      configurable: true,
    });
  }

  describe('isWechatBrowser', () => {
    it('returns true for WeChat browser user agent', async () => {
      mockUserAgent(
        'Mozilla/5.0 (Linux; Android 10) AppleWebKit/537.36 MicroMessenger/7.0.20'
      );
      const { isWechatBrowser } = await import('../wechat');
      expect(isWechatBrowser()).toBe(true);
    });

    it('returns false for regular browser user agent', async () => {
      mockUserAgent(
        'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0'
      );
      const { isWechatBrowser } = await import('../wechat');
      expect(isWechatBrowser()).toBe(false);
    });

    it('returns false for empty user agent', async () => {
      mockUserAgent('');
      const { isWechatBrowser } = await import('../wechat');
      expect(isWechatBrowser()).toBe(false);
    });
  });

  describe('isWechatSdkAvailable', () => {
    it('returns false in non-WeChat browser', async () => {
      mockUserAgent('Chrome/120.0.0.0');
      const { isWechatSdkAvailable } = await import('../wechat');
      expect(await isWechatSdkAvailable()).toBe(false);
    });

    it('returns false when server returns available=false', async () => {
      mockUserAgent('MicroMessenger/8.0.33');
      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ available: false, reason: 'WX_APPID not configured' }),
      });
      vi.stubGlobal('fetch', fetchMock);

      const { isWechatSdkAvailable } = await import('../wechat');
      expect(await isWechatSdkAvailable()).toBe(false);
    });

    it('returns true when server returns available=true', async () => {
      mockUserAgent('MicroMessenger/8.0.33');
      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ available: true, reason: null }),
      });
      vi.stubGlobal('fetch', fetchMock);

      const { isWechatSdkAvailable } = await import('../wechat');
      expect(await isWechatSdkAvailable()).toBe(true);
    });

    it('returns false when fetch fails', async () => {
      mockUserAgent('MicroMessenger/8.0.33');
      const fetchMock = vi.fn().mockRejectedValue(new Error('Network error'));
      vi.stubGlobal('fetch', fetchMock);

      const { isWechatSdkAvailable } = await import('../wechat');
      expect(await isWechatSdkAvailable()).toBe(false);
    });
  });

  describe('getWechatVersion', () => {
    it('extracts version from WeChat user agent', async () => {
      mockUserAgent(
        'Mozilla/5.0 MicroMessenger/8.0.33 AppleWebKit/537.36'
      );
      const { getWechatVersion } = await import('../wechat');
      expect(getWechatVersion()).toBe('8.0.33');
    });

    it('returns null for non-WeChat browser', async () => {
      mockUserAgent(
        'Mozilla/5.0 Chrome/120.0.0.0'
      );
      const { getWechatVersion } = await import('../wechat');
      expect(getWechatVersion()).toBeNull();
    });
  });

  describe('initWxSdk', () => {
    it('does nothing in non-WeChat browser', async () => {
      mockUserAgent('Chrome/120.0.0.0');
      const { initWxSdk } = await import('../wechat');
      await expect(initWxSdk()).resolves.toBeUndefined();
    });

    it('throws when WeChat SDK is not available on server', async () => {
      mockUserAgent('MicroMessenger/8.0.33');
      const fetchMock = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ available: false, reason: 'WX_APPID not configured' }),
      });
      vi.stubGlobal('fetch', fetchMock);

      const { initWxSdk } = await import('../wechat');
      expect(initWxSdk()).rejects.toThrow('WeChat JS-SDK is not available');
    });
  });

  describe('scanQRCode', () => {
    it('rejects in non-WeChat browser', async () => {
      mockUserAgent('Chrome/120.0.0.0');
      const { scanQRCode } = await import('../wechat');
      expect(scanQRCode()).rejects.toThrow('Not in WeChat browser');
    });

    it('rejects when SDK not initialized', async () => {
      mockUserAgent('MicroMessenger/8.0.33');
      const { scanQRCode } = await import('../wechat');
      expect(scanQRCode()).rejects.toThrow('WeChat JS-SDK not initialized');
    });
  });

  describe('configureShare', () => {
    it('does nothing in non-WeChat browser', async () => {
      mockUserAgent('Chrome/120.0.0.0');
      const { configureShare } = await import('../wechat');
      expect(() =>
        configureShare({
          title: 'Test',
          desc: 'Test desc',
          link: 'https://example.com',
          imgUrl: 'https://example.com/icon.png',
        })
      ).not.toThrow();
    });
  });
});
