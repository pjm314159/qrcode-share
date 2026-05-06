import { useCallback, useRef, useState, useEffect } from 'react';
import QRCode from 'qrcode';
import { Card, Button } from '@/components/ui';
import { IconCopy, IconDownload } from '@/components/icons';

interface QRCodeDisplayProps {
  value: string;
  size?: number;
  title?: string;
}

export function QRCodeDisplay({ value, size = 200, title = 'Scan to join' }: QRCodeDisplayProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [dataUrl, setDataUrl] = useState<string>('');

  useEffect(() => {
    QRCode.toDataURL(value, {
      width: size,
      margin: 2,
      color: {
        dark: '#0a0a0a',
        light: '#ffffff',
      },
    }).then((url) => {
      setDataUrl(url);
    }).catch(() => {
      // ignore
    });
  }, [value, size]);

  const handleCopyLink = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(value);
    } catch {
      const textarea = document.createElement('textarea');
      textarea.value = value;
      document.body.appendChild(textarea);
      textarea.select();
      document.execCommand('copy');
      document.body.removeChild(textarea);
    }
  }, [value]);

  const handleDownload = useCallback(() => {
    if (!dataUrl) return;
    const link = document.createElement('a');
    link.download = 'qrcode.png';
    link.href = dataUrl;
    link.click();
  }, [dataUrl]);

  return (
    <Card variant="cream">
      <h3 className="mb-3 text-center text-lg font-semibold text-ink">
        {title}
      </h3>
      <div className="flex justify-center py-4">
        {dataUrl && (
          <img src={dataUrl} alt="QR Code" width={size} height={size} />
        )}
      </div>
      <canvas ref={canvasRef} className="hidden" />
      <div className="flex gap-2">
        <Button variant="secondary" size="sm" onClick={handleCopyLink} className="flex-1">
          <IconCopy size={16} className="mr-1.5" />
          Copy Link
        </Button>
        <Button variant="secondary" size="sm" onClick={handleDownload} className="flex-1">
          <IconDownload size={16} className="mr-1.5" />
          Download
        </Button>
      </div>
    </Card>
  );
}
