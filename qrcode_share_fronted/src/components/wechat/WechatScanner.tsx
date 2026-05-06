import { useEffect, useState, useCallback } from 'react';
import { Button } from '@/components/ui';
import { IconScan } from '@/components/icons';
import { initWxSdk, scanQRCode, isWechatBrowser, isWxInitialized } from '@/utils/wechat';

interface WechatScannerProps {
  onScan: (decodedText: string) => void;
  onError?: (error: string) => void;
}

export function WechatScanner({ onScan, onError }: WechatScannerProps) {
  const [ready, setReady] = useState(false);
  const [initializing, setInitializing] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!isWechatBrowser()) {
      const timer = setTimeout(() => {
        setError('This component is only available in WeChat browser');
      }, 0);
      return () => clearTimeout(timer);
    }

    if (isWxInitialized()) {
      const timer = setTimeout(() => {
        setReady(true);
      }, 0);
      return () => clearTimeout(timer);
    }

    const timer = setTimeout(() => {
      setInitializing(true);
      initWxSdk()
        .then(() => {
          setReady(true);
          setInitializing(false);
        })
        .catch((err) => {
          const message = err instanceof Error ? err.message : 'Failed to initialize WeChat JS-SDK';
          setError(message);
          setInitializing(false);
          onError?.(message);
        });
    }, 0);
    return () => clearTimeout(timer);
  }, [onError]);

  const handleScan = useCallback(async () => {
    try {
      setError(null);
      const result = await scanQRCode();
      onScan(result);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'QR scan failed';
      setError(message);
      onError?.(message);
    }
  }, [onScan, onError]);

  if (error) {
    return (
      <div className="space-y-2">
        <p className="text-sm text-error">{error}</p>
        {isWechatBrowser() && (
          <Button
            variant="secondary"
            size="sm"
            onClick={() => {
              setError(null);
              setInitializing(true);
              initWxSdk()
                .then(() => {
                  setReady(true);
                  setInitializing(false);
                })
                .catch((err) => {
                  setError(err instanceof Error ? err.message : 'Retry failed');
                  setInitializing(false);
                });
            }}
          >
            Retry Initialization
          </Button>
        )}
      </div>
    );
  }

  return (
    <div className="space-y-3">
      <p className="text-sm text-muted">
        {initializing
          ? 'Initializing WeChat scanner...'
          : ready
            ? 'Tap the button below to scan a QR code using WeChat'
            : 'Preparing scanner...'}
      </p>
      <Button
        onClick={handleScan}
        disabled={!ready || initializing}
        className="w-full"
      >
        <IconScan size={16} className="mr-1.5" />
        Scan QR Code
      </Button>
    </div>
  );
}
