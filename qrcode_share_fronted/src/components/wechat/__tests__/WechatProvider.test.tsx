import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, render } from '@testing-library/react';
import { useWechatViewport, useWechatBackButton } from '@/hooks/useWechat';
import { WechatProvider } from '../WechatProvider';

vi.mock('@/utils/wechat', () => ({
  isWechatBrowser: vi.fn(() => false),
  getWechatVersion: vi.fn(() => null),
  initWxSdk: vi.fn(() => Promise.resolve()),
  configureShare: vi.fn(),
}));

import { isWechatBrowser } from '@/utils/wechat';

describe('useWechatViewport', () => {
  it('does not modify viewport in non-WeChat browser', () => {
    vi.mocked(isWechatBrowser).mockReturnValue(false);
    renderHook(() => useWechatViewport());
    const viewport = document.querySelector('meta[name="viewport"]');
    expect(viewport).toBeNull();
  });

  it('sets viewport meta in WeChat browser', () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    renderHook(() => useWechatViewport());
    const viewport = document.querySelector('meta[name="viewport"]');
    expect(viewport).not.toBeNull();
    expect(viewport?.getAttribute('content')).toContain('viewport-fit=cover');
  });
});

describe('useWechatBackButton', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it('does not add listener in non-WeChat browser', () => {
    vi.mocked(isWechatBrowser).mockReturnValue(false);
    const addSpy = vi.spyOn(window, 'addEventListener');
    renderHook(() => useWechatBackButton());
    expect(addSpy).not.toHaveBeenCalledWith('popstate', expect.any(Function));
  });

  it('adds popstate listener in WeChat browser', () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    const addSpy = vi.spyOn(window, 'addEventListener');
    renderHook(() => useWechatBackButton());
    expect(addSpy).toHaveBeenCalledWith('popstate', expect.any(Function));
  });
});

describe('WechatProvider', () => {
  it('renders children', () => {
    vi.mocked(isWechatBrowser).mockReturnValue(false);
    const { getByText } = render(
      <WechatProvider>Test Child</WechatProvider>
    );
    expect(getByText('Test Child')).toBeDefined();
  });

  it('sets wechat dataset attribute in WeChat browser', () => {
    vi.mocked(isWechatBrowser).mockReturnValue(true);
    render(<WechatProvider>Test</WechatProvider>);
    expect(document.documentElement.dataset.wechat).toBe('true');
  });
});
