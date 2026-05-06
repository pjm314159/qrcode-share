import { useState, useEffect } from 'react';
import { isWechatBrowser, isWechatSdkAvailable } from '@/utils/wechat';
import { QRScanner } from './QRScanner';
import { WechatScanner } from '@/components/wechat';

interface SmartQRScannerProps {
  onScan: (decodedText: string) => void;
  onError?: (error: string) => void;
  autoStart?: boolean;
}

export function SmartQRScanner({ onScan, onError, autoStart = false }: SmartQRScannerProps) {
  const [useWechatSdk, setUseWechatSdk] = useState(false);
  const [checked, setChecked] = useState(false);

  useEffect(() => {
    if (!isWechatBrowser()) {
      const timer = setTimeout(() => {
        setChecked(true);
      }, 0);
      return () => clearTimeout(timer);
    }

    isWechatSdkAvailable().then((available) => {
      const timer = setTimeout(() => {
        setUseWechatSdk(available);
        setChecked(true);
      }, 0);
      return () => clearTimeout(timer);
    });
  }, []);

  if (!checked) {
    return null;
  }

  if (useWechatSdk) {
    return <WechatScanner onScan={onScan} onError={onError} />;
  }

  return <QRScanner onScan={onScan} onError={onError} autoStart={autoStart} />;
}
