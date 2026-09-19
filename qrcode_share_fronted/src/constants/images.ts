/**
 * Static image assets.
 *
 * Every image ships in two formats and is rendered through `<Picture />`
 * (`@/components/common`), which emits `<source type="image/avif">` and falls
 * back to WebP for browsers without AVIF support. PNG is no longer shipped —
 * the original PNG sources live in `.temp/images-png/` (git-ignored).
 *
 * Conversion recipe (near-lossless, 10-bit 4:4:4, CICP = bt709 / sRGB / bt709):
 *   ffmpeg -i x.png -c:v libaom-av1 -still-picture 1 -crf 10 -cpu-used 0 -row-mt 1 \
 *     -pix_fmt yuv444p10le -color_range pc -colorspace bt709 -color_primaries bt709 \
 *     -color_trc iec61966-2-1 -f avif x.avif
 *   avifenc -s 0 -j all -d 10 -y 444 -q 92 --qalpha 100 --cicp 1/13/1 -r f -o x.avif x.png  # alpha images
 *   ffmpeg -i x.png -c:v libwebp -lossless 1 -compression_level 6 -preset drawing x.webp
 */
export interface ImageSources {
  /** AVIF — preferred. */
  avif: string;
  /** WebP — fallback for browsers without AVIF support. */
  webp: string;
}

function sources(name: string): ImageSources {
  return {
    avif: `/images/${name}.avif`,
    webp: `/images/${name}.webp`,
  };
}

export const IMAGES = {
  logo: sources('logo-mark'),
  hero: sources('hero-illustration'),
  emptyInbox: sources('empty-inbox'),
  notFound: sources('not-found'),
  featureScan: sources('feature-scan'),
  featureShare: sources('feature-share'),
  featureOpen: sources('feature-open'),
  featureCreate: sources('feature-create'),
  passwordLock: sources('password-lock'),
  statusConnected: sources('status-connected'),
  statusConnecting: sources('status-connecting'),
  statusDisconnected: sources('status-disconnected'),
  footerMountains: sources('footer-mountains'),
} satisfies Record<string, ImageSources>;

export type ImageKey = keyof typeof IMAGES;

/** Image used as the browser tab icon; kept in sync with `index.html`. */
export const FAVICON = sources('2-64x64');
