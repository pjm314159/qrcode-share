import { useEffect, useRef, useState, useCallback } from 'react';
import { IconScan, IconClose } from '@/components/icons';

interface QRScannerProps {
  onScan: (decodedText: string) => void;
  onError?: (error: string) => void;
  autoStart?: boolean;
}

export function QRScanner({ onScan, onError, autoStart = false }: QRScannerProps) {
  const scannerRef = useRef<HTMLDivElement>(null);
  const scannerInstanceRef = useRef<unknown>(null);
  const isRunningRef = useRef(false);
  const [scanning, setScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const autoStartRef = useRef(autoStart);
  const onScanRef = useRef(onScan);
  const onErrorRef = useRef(onError);

  useEffect(() => {
    autoStartRef.current = autoStart;
    onScanRef.current = onScan;
    onErrorRef.current = onError;
  });

  const stopScanner = useCallback(async () => {
    if (scannerInstanceRef.current && isRunningRef.current) {
      const scanner = scannerInstanceRef.current as {
        stop: () => Promise<void>;
        isScanning: () => boolean;
      };
      try {
        if (scanner.isScanning?.()) {
          await scanner.stop();
        }
      } catch {
        // Ignore stop errors
      }
      isRunningRef.current = false;
      scannerInstanceRef.current = null;
    }
    setScanning(false);
  }, []);

  const startScanning = useCallback(async () => {
    if (!scannerRef.current) return;

    await stopScanner();

    try {
      const { Html5Qrcode } = await import('html5-qrcode');
      const scannerId = scannerRef.current.id || 'qr-scanner';

      if (!scannerRef.current.id) {
        scannerRef.current.id = 'qr-scanner';
      }

      const scanner = new Html5Qrcode(scannerId);
      scannerInstanceRef.current = scanner;

      await scanner.start(
        { facingMode: 'environment' },
        {
          fps: 10,
          qrbox: { width: 250, height: 250 },
        },
        (decodedText) => {
          onScanRef.current(decodedText);
          isRunningRef.current = false;
          scanner.stop().catch(() => {});
          setScanning(false);
        },
        () => {}
      );

      isRunningRef.current = true;
      setScanning(true);
      setError(null);
    } catch (err) {
      const message =
        err instanceof Error ? err.message : 'Failed to start scanner';
      setError(message);
      setScanning(false);
      isRunningRef.current = false;
      onErrorRef.current?.(message);
    }
  }, [stopScanner]);

  useEffect(() => {
    if (autoStartRef.current) {
      const timer = setTimeout(() => {
        startScanning().catch(() => {});
      }, 0);
      return () => clearTimeout(timer);
    }
  }, [startScanning]);

  useEffect(() => {
    return () => {
      if (scannerInstanceRef.current && isRunningRef.current) {
        const scanner = scannerInstanceRef.current as {
          stop: () => Promise<void>;
          isScanning: () => boolean;
        };
        try {
          if (scanner.isScanning?.()) {
            scanner.stop().catch(() => {});
          }
        } catch {
          // Ignore
        }
        isRunningRef.current = false;
      }
    };
  }, []);

  return (
    <div className="space-y-3">
      <div className="relative max-h-[60vh] overflow-hidden rounded-md">
        <div
          ref={scannerRef}
          id="qr-scanner"
          className="overflow-hidden rounded-md"
        />
        {scanning && (
          <button
            onClick={stopScanner}
            aria-label="Stop scanner"
            className="absolute right-2 top-2 z-10 rounded-full bg-ink/70 p-1.5 text-on-primary backdrop-blur-sm transition-colors hover:bg-ink"
          >
            <IconClose size={16} />
          </button>
        )}
      </div>

      {error && (
        <p className="text-sm text-error">{error}</p>
      )}

      {!autoStart && (
        <div className="flex gap-2">
          {!scanning ? (
            <button onClick={startScanning} className="inline-flex w-full items-center justify-center gap-2 rounded-md bg-ink px-4 py-3 text-sm font-semibold text-on-primary hover:bg-ink-active transition-colors">
              <IconScan size={16} />
              Start Scanner
            </button>
          ) : (
            <button onClick={stopScanner} className="inline-flex w-full items-center justify-center gap-2 rounded-md bg-error px-4 py-3 text-sm font-semibold text-on-primary hover:bg-error/90 transition-colors">
              <IconClose size={16} />
              Stop Scanner
            </button>
          )}
        </div>
      )}
    </div>
  );
}
