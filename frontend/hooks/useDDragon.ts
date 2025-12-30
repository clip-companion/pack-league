/**
 * React hook for DDragon state reactivity
 *
 * Subscribes to DDragon loading state changes and triggers re-renders
 * when the cache becomes ready. Also triggers lazy initialization on mount.
 */

import { useState, useEffect, useRef } from 'react';
import { onStateChange, initDDragon } from '../lib/ddragon';

interface DDragonState {
  loading: boolean;
  ready: boolean;
  progress: { total: number; loaded: number; currentItem: string };
  version: string | null;
}

export function useDDragon(): DDragonState {
  const [state, setState] = useState<DDragonState>({
    loading: false,
    ready: false,
    progress: { total: 4, loaded: 0, currentItem: '' },
    version: null,
  });
  const initCalled = useRef(false);

  useEffect(() => {
    // Subscribe to state changes
    const unsubscribe = onStateChange(setState);

    // Trigger lazy initialization (only once globally, initDDragon handles dedup)
    if (!initCalled.current) {
      initCalled.current = true;
      initDDragon().catch((err) => {
        console.error('[DDragon] Failed to initialize:', err);
      });
    }

    return unsubscribe;
  }, []);

  return state;
}

/**
 * Hook that returns true when DDragon is ready
 * Use this in components that need to re-render when icons become available
 */
export function useDDragonReady(): boolean {
  const { ready } = useDDragon();
  return ready;
}
