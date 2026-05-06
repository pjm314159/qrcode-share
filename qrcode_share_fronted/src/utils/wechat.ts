import wx from 'weixin-js-sdk';
import { isWechatBrowser } from './helpers';
import type { WxConfig, ShareData } from 'weixin-js-sdk';

export { isWechatBrowser };

interface WxJsapiTicketResponse {
  app_id: string;
  timestamp: number;
  nonce_str: string;
  signature: string;
}

interface WxStatusResponse {
  available: boolean;
  reason: string | null;
}

let initialized = false;
let initPromise: Promise<void> | null = null;
let wechatAvailable: boolean | null = null;

export function isWxInitialized(): boolean {
  return initialized;
}

export async function isWechatSdkAvailable(): Promise<boolean> {
  if (wechatAvailable !== null) {
    return wechatAvailable;
  }

  if (!isWechatBrowser()) {
    wechatAvailable = false;
    return false;
  }

  try {
    const baseUrl = import.meta.env.VITE_API_URL || window.location.origin;
    const response = await fetch(`${baseUrl}/api/wechat/status`);

    if (!response.ok) {
      wechatAvailable = false;
      console.warn('[WeChat] Failed to check WeChat status:', response.status);
      return false;
    }

    const data: WxStatusResponse = await response.json();
    wechatAvailable = data.available;

    if (!data.available) {
      const reason = data.reason || 'Unknown reason';
      if (reason.includes('not configured')) {
        console.info('[WeChat] JS-SDK not configured, using standard browser scanner');
      } else {
        console.warn('[WeChat] JS-SDK not available:', reason);
      }
    }

    return data.available;
  } catch (err) {
    wechatAvailable = false;
    console.warn('[WeChat] Failed to check WeChat status:', err);
    return false;
  }
}

export async function initWxSdk(): Promise<void> {
  if (!isWechatBrowser()) {
    return;
  }

  if (initialized) {
    return;
  }

  if (initPromise) {
    return initPromise;
  }

  initPromise = (async () => {
    try {
      const available = await isWechatSdkAvailable();
      if (!available) {
        throw new Error('WeChat JS-SDK is not available on this server');
      }

      const baseUrl = import.meta.env.VITE_API_URL || window.location.origin;
      const url = `${baseUrl}/api/wechat/jsapi-ticket`;

      const response = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ url: window.location.href.split('#')[0] }),
      });

      if (!response.ok) {
        throw new Error(`Failed to fetch JSAPI ticket: ${response.status}`);
      }

      const data: WxJsapiTicketResponse = await response.json();

      const config: WxConfig = {
        debug: import.meta.env.DEV && import.meta.env.VITE_WX_DEBUG === 'true',
        appId: data.app_id,
        timestamp: data.timestamp,
        nonceStr: data.nonce_str,
        signature: data.signature,
        jsApiList: ['scanQRCode', 'updateAppMessageShareData', 'updateTimelineShareData'],
      };

      wx.config(config);

      await new Promise<void>((resolve, reject) => {
        wx.ready(() => {
          initialized = true;
          resolve();
        });
        wx.error((err) => {
          reject(err);
        });
      });
    } catch (err) {
      initPromise = null;
      throw err;
    }
  })();

  return initPromise;
}

export function scanQRCode(): Promise<string> {
  return new Promise((resolve, reject) => {
    if (!isWechatBrowser()) {
      reject(new Error('Not in WeChat browser'));
      return;
    }

    if (!initialized) {
      reject(new Error('WeChat JS-SDK not initialized. Call initWxSdk() first.'));
      return;
    }

    wx.scanQRCode({
      needResult: 1,
      scanType: ['qrCode'],
      success: (res) => {
        if (res.resultStr) {
          resolve(res.resultStr);
        } else {
          reject(new Error('No result from QR scan'));
        }
      },
      error: (err) => {
        reject(new Error(err ? String(err) : 'QR scan failed or was cancelled'));
      },
    });
  });
}

export function configureShare(data: ShareData): void {
  if (!isWechatBrowser() || !initialized) {
    return;
  }

  wx.updateAppMessageShareData({
    title: data.title,
    desc: data.desc,
    link: data.link,
    imgUrl: data.imgUrl,
    success: data.success,
    cancel: data.cancel,
  });

  wx.updateTimelineShareData({
    title: data.title,
    desc: data.desc,
    link: data.link,
    imgUrl: data.imgUrl,
    success: data.success,
    cancel: data.cancel,
  });
}

export function getWechatVersion(): string | null {
  if (!isWechatBrowser()) {
    return null;
  }

  const ua = navigator.userAgent;
  const match = ua.match(/MicroMessenger\/([\d.]+)/);
  return match ? match[1] : null;
}
