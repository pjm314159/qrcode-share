import type { ImgHTMLAttributes } from 'react';
import type { ImageSources } from '@/constants/images';

export interface PictureProps
  extends Omit<ImgHTMLAttributes<HTMLImageElement>, 'src' | 'srcSet'> {
  /** Format set produced by the image pipeline (AVIF → WebP). */
  image: ImageSources;
}

/**
 * Renders an image with progressive format fallback: AVIF when supported,
 * otherwise WebP (which is also the `<img src>` fallback required by the
 * `<picture>` element). All `<img>` attributes (`alt`, `width`, `height`,
 * `className`, ...) are forwarded to the inner `<img>` element.
 */
export function Picture({ image, alt = '', ...imgProps }: PictureProps) {
  return (
    <picture>
      <source srcSet={image.avif} type="image/avif" />
      <img src={image.webp} alt={alt} {...imgProps} />
    </picture>
  );
}
