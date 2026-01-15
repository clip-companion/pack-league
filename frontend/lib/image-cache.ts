// Image caching utilities for League pack
// Caches CDN images locally via the pack context API for offline access and faster loading
//
// NOTE: This module does NOT directly access electronAPI.
// All cache operations go through the PackCacheAPI provided by the host application.

import type { PackCacheAPI } from "@companion/gamepack-runtime";

// In-memory cache of URL -> data URL (for instant access after first load)
const memoryCache = new Map<string, string>();

// Pending fetches to avoid duplicate requests
const pendingFetches = new Map<string, Promise<string>>();

/**
 * Convert a CDN URL to a cache-safe filename.
 * e.g., "https://ddragon.../14.24.1/img/champion/Ahri.png" -> "images/champion_Ahri.png"
 */
function urlToCacheKey(url: string): string {
  try {
    const urlObj = new URL(url);
    // Extract meaningful parts: /cdn/14.24.1/img/champion/Ahri.png -> champion_Ahri.png
    // or /cdn/img/perk-images/Styles/... -> perk-images_Styles_...
    const pathParts = urlObj.pathname.split('/').filter(Boolean);

    // Find index of 'img' and take everything after
    const imgIndex = pathParts.indexOf('img');
    if (imgIndex >= 0 && imgIndex < pathParts.length - 1) {
      const relevantParts = pathParts.slice(imgIndex + 1);
      return `images/${relevantParts.join('_')}`;
    }

    // Fallback: use last two path segments
    const lastTwo = pathParts.slice(-2);
    return `images/${lastTwo.join('_')}`;
  } catch {
    // Fallback: simple hash-like key
    return `images/${url.replace(/[^a-zA-Z0-9]/g, '_').slice(-100)}`;
  }
}

/**
 * Fetch an image from URL and convert to base64 data URL.
 */
async function fetchImageAsDataUrl(url: string): Promise<string> {
  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch image: ${response.status}`);
  }

  const blob = await response.blob();
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onloadend = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(blob);
  });
}

/**
 * Get a cached image URL using the provided cache API.
 * Returns data URL if cached, fetches and caches if not.
 * Falls back to original URL if caching fails.
 *
 * @param cdnUrl - The CDN URL to cache
 * @param cache - The pack cache API (from usePackCache())
 */
export async function getCachedImageUrl(cdnUrl: string, cache: PackCacheAPI): Promise<string> {
  // Check memory cache first (instant)
  if (memoryCache.has(cdnUrl)) {
    return memoryCache.get(cdnUrl)!;
  }

  // Check if there's already a pending fetch for this URL
  if (pendingFetches.has(cdnUrl)) {
    return pendingFetches.get(cdnUrl)!;
  }

  const cacheKey = urlToCacheKey(cdnUrl);

  // Start the fetch/cache process
  const fetchPromise = (async () => {
    try {
      // Check disk cache via provided API
      const cached = await cache.read<string>(cacheKey);
      if (cached) {
        memoryCache.set(cdnUrl, cached);
        return cached;
      }

      // Fetch from CDN
      const dataUrl = await fetchImageAsDataUrl(cdnUrl);

      // Save to memory cache
      memoryCache.set(cdnUrl, dataUrl);

      // Save to disk cache (async, don't wait)
      cache.write(cacheKey, dataUrl).catch(err => {
        console.warn('[ImageCache] Failed to write cache:', err);
      });

      return dataUrl;
    } catch (err) {
      console.warn('[ImageCache] Failed to cache image, using CDN URL:', cdnUrl, err);
      // Fall back to CDN URL
      return cdnUrl;
    } finally {
      // Clean up pending fetch
      pendingFetches.delete(cdnUrl);
    }
  })();

  pendingFetches.set(cdnUrl, fetchPromise);
  return fetchPromise;
}

/**
 * Check if an image is already cached (in memory).
 */
export function isImageCached(cdnUrl: string): boolean {
  return memoryCache.has(cdnUrl);
}

/**
 * Clear the in-memory image cache.
 * (Disk cache is managed separately via the cache management UI)
 */
export function clearMemoryCache(): void {
  memoryCache.clear();
}
