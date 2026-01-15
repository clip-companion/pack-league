import { useState, useEffect } from 'react';
import { usePackCache } from '@companion/gamepack-runtime';
import { cn } from '../lib/cn';
import { getCachedImageUrl, isImageCached } from '../lib/image-cache';

interface GameIconProps {
  src: string | null | undefined;
  alt: string;
  size?: number;
  shape?: 'circle' | 'square';
  title?: string;
  fallback?: string;
  className?: string;
}

/**
 * Game icon component - displays champion, item, spell, or rune icons from DDragon
 * Automatically caches images locally for offline access and faster subsequent loads.
 * Uses the pack context's cache API - never directly accesses Electron internals.
 * Falls back to a text placeholder if image fails to load or src is null.
 */
export function GameIcon({
  src,
  alt,
  size = 24,
  shape = 'square',
  title,
  fallback,
  className,
}: GameIconProps) {
  const cache = usePackCache();
  const [hasError, setHasError] = useState(false);
  const [cachedSrc, setCachedSrc] = useState<string | null>(() => {
    // If already in memory cache, use it immediately
    if (src && isImageCached(src)) {
      return src; // Will be resolved synchronously in getCachedImageUrl
    }
    return null;
  });

  // Load and cache the image
  useEffect(() => {
    if (!src) {
      setCachedSrc(null);
      return;
    }

    // Reset error state when src changes
    setHasError(false);

    // If already cached in memory, getCachedImageUrl returns immediately
    if (isImageCached(src)) {
      getCachedImageUrl(src, cache).then(setCachedSrc);
      return;
    }

    // Start loading - use CDN URL immediately, then switch to cached when ready
    setCachedSrc(src); // Show CDN URL first for fast initial render

    getCachedImageUrl(src, cache)
      .then(cached => {
        // Only update if still the same src
        setCachedSrc(cached);
      })
      .catch(() => {
        // Keep using CDN URL on cache failure
      });
  }, [src, cache]);

  const sizeStyle = { width: size, height: size };
  const shapeClass = shape === 'circle' ? 'rounded-full' : 'rounded-sm';

  // Show fallback if no src or image failed to load
  if (!src || hasError) {
    const fallbackText = fallback || alt?.charAt(0)?.toUpperCase() || '?';
    return (
      <div
        className={cn(
          'overflow-hidden bg-slate-700 flex items-center justify-center font-bold text-white',
          shapeClass,
          className
        )}
        style={{
          ...sizeStyle,
          fontSize: Math.max(8, size * 0.4),
        }}
        title={title || alt}
      >
        {fallbackText}
      </div>
    );
  }

  return (
    <img
      src={cachedSrc || src}
      alt={alt}
      title={title || alt}
      className={cn('object-cover', shapeClass, className)}
      style={sizeStyle}
      onError={() => setHasError(true)}
      loading="lazy"
    />
  );
}
