declare module 'weixin-js-sdk' {
  interface WxConfig {
    debug?: boolean;
    appId: string;
    timestamp: number;
    nonceStr: string;
    signature: string;
    jsApiList: string[];
  }

  interface ScanQRCodeParams {
    needResult: number;
    scanType: string[];
    success: (res: { resultStr: string }) => void;
    error?: (err?: unknown) => void;
  }

  interface ShareData {
    title: string;
    desc: string;
    link: string;
    imgUrl: string;
    success?: () => void;
    cancel?: () => void;
  }

  interface Wx {
    config(config: WxConfig): void;
    ready(callback: () => void): void;
    error(callback: (err: unknown) => void): void;
    scanQRCode(params: ScanQRCodeParams): void;
    updateAppMessageShareData(data: ShareData): void;
    updateTimelineShareData(data: ShareData): void;
  }

  const wx: Wx;
  export default wx;
  export { Wx, WxConfig, ScanQRCodeParams, ShareData };
}
