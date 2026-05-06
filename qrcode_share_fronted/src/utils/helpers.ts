export function isWechatBrowser(): boolean {
  const ua = navigator.userAgent.toLowerCase();
  return ua.includes('micromessenger');
}

export function getBaseUrl(): string {
  return import.meta.env.VITE_API_URL || window.location.origin;
}

export function getWsBaseUrl(): string {
  const configured = import.meta.env.VITE_WS_URL;
  if (configured) return configured;

  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  return `${protocol}//${window.location.host}`;
}

export function validateLink(link: string): boolean {
  try {
    const url = new URL(link);
    return url.protocol === 'http:' || url.protocol === 'https:';
  } catch {
    return false;
  }
}

export function extractDomain(link: string): string {
  try {
    return new URL(link).hostname;
  } catch {
    return '';
  }
}

export function formatRemainingTime(seconds: number): string {
  if (seconds <= 0) return 'Expired';
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${minutes}m ${secs}s`;
}

export function generateChannelId(): string {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
  let result = '';
  for (let i = 0; i < 8; i++) {
    result += chars.charAt(Math.floor(Math.random() * chars.length));
  }
  return result;
}
