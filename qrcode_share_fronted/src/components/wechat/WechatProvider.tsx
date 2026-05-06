import { useEffect } from 'react';
import { isWechatBrowser, getWechatVersion } from '@/utils/wechat';

export function WechatProvider({ children }: { children: React.ReactNode }) {
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

    const version = getWechatVersion();
    if (version) {
      document.documentElement.dataset.wechatVersion = version;
    }

    document.documentElement.dataset.wechat = 'true';
  }, []);

  return <>{children}</>;
}
