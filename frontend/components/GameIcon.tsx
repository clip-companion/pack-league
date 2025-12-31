import { useState } from 'react';
import { cn } from '../lib/cn';

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
 * Falls back to a text placeholder if image fails to load or src is null
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
  const [hasError, setHasError] = useState(false);

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
      src={src}
      alt={alt}
      title={title || alt}
      className={cn('object-cover', shapeClass, className)}
      style={sizeStyle}
      onError={() => setHasError(true)}
      loading="lazy"
    />
  );
}
