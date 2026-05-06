import { useEffect } from 'react';
import { isWechatBrowser, initWxSdk, configureShare } from '@/utils/wechat';

export function useWechatViewport() {
  useEffect(() => {
    if (!isWechatBrowser()) return;

    const viewport = document.querySelector('meta[name="viewport"]');
    if (viewport) {
      viewport.setAttribute(
        'content',
        'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover'
      );
    } else {
      const meta = document.createElement('meta');
      meta.name = 'viewport';
      meta.content = 'width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover';
      document.head.appendChild(meta);
    }

    document.documentElement.style.setProperty('--safe-area-inset-top', 'env(safe-area-inset-top, 0px)');
    document.documentElement.style.setProperty('--safe-area-inset-bottom', 'env(safe-area-inset-bottom, 0px)');
  }, []);
}

export function useWechatBackButton(onBack?: () => void) {
  useEffect(() => {
    if (!isWechatBrowser()) return;

    const handler = () => {
      if (onBack) {
        onBack();
      } else {
        window.history.back();
      }
    };

    window.addEventListener('popstate', handler);
    return () => {
      window.removeEventListener('popstate', handler);
    };
  }, [onBack]);
}

export function useWechatShare(data: {
  title: string;
  desc: string;
  link: string;
  imgUrl: string;
}) {
  useEffect(() => {
    if (!isWechatBrowser()) return;

    initWxSdk()
      .then(() => {
        configureShare(data);
      })
      .catch(() => {});
  }, [data]);
}
