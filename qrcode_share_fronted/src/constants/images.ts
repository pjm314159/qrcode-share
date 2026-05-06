export const IMAGES = {
  logo: '/images/logo-mark.png',
  hero: '/images/hero-illustration.png',
  emptyInbox: '/images/empty-inbox.png',
  notFound: '/images/not-found.png',
  featureScan: '/images/feature-scan.png',
  featureShare: '/images/feature-share.png',
  featureOpen: '/images/feature-open.png',
  featureCreate: '/images/feature-create.png',
  passwordLock: '/images/password-lock.png',
  statusConnected: '/images/status-connected.png',
  statusConnecting: '/images/status-connecting.png',
  statusDisconnected: '/images/status-disconnected.png',
  footerMountains: '/images/footer-mountains.png',
} as const;

export type ImageKey = keyof typeof IMAGES;
