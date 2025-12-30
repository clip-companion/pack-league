/**
 * League of Legends Game Pack
 *
 * Provides all League-specific components, utilities, and data loading
 * for the multi-game architecture.
 */

import type { GamePack, GamePackResources, ResourceState } from "@companion/pack-runtime";
import type { LeagueMatch, LeagueLiveMatch } from "./types";
import { MatchCard } from "./MatchCard";
import { LiveMatchCard } from "./LiveMatchCard";
import { initDDragon, onStateChange as onDDragonStateChange } from "./lib/ddragon";
import { DDragonStatus } from "./components/DDragonStatus";

/**
 * Resource management for League DDragon assets
 */
const leagueResources: GamePackResources = {
  isReady: () => {
    // Check if DDragon is ready by getting current state
    let ready = false;
    const unsubscribe = onDDragonStateChange((state) => {
      ready = state.ready;
    });
    unsubscribe();
    return ready;
  },

  init: initDDragon,

  getState: () => {
    let state: ResourceState = { loading: false, ready: false };
    const unsubscribe = onDDragonStateChange((ddState) => {
      state = {
        loading: ddState.loading,
        ready: ddState.ready,
        progress: ddState.progress ? Math.round((ddState.progress.loaded / ddState.progress.total) * 100) : undefined,
        currentItem: ddState.progress?.currentItem,
        version: ddState.version,
      };
    });
    unsubscribe();
    return state;
  },

  onStateChange: (callback) => {
    return onDDragonStateChange((ddState) => {
      callback({
        loading: ddState.loading,
        ready: ddState.ready,
        progress: ddState.progress ? Math.round((ddState.progress.loaded / ddState.progress.total) * 100) : undefined,
        currentItem: ddState.progress?.currentItem,
        version: ddState.version,
      });
    });
  },
};

/**
 * League of Legends game pack implementation
 */
const leaguePack: GamePack<LeagueMatch, LeagueLiveMatch> = {
  gameId: 1,
  slug: "league",

  MatchCard,
  LiveMatchCard,

  resources: leagueResources,
  AssetsStatus: DDragonStatus,

  isMatch: (match): match is LeagueMatch => {
    return match.gameId === 1;
  },

  // Legacy - kept for backwards compatibility
  initAssets: initDDragon,
};

export default leaguePack;

// Re-export types for convenience
export type { LeagueMatch, LeagueLiveMatch } from "./types";
export * from "./types";

// Re-export components
export { MatchCard } from "./MatchCard";
export { LiveMatchCard } from "./LiveMatchCard";
export { DDragonStatus } from "./components/DDragonStatus";

// Re-export hooks
export { useDDragon, useDDragonReady } from "./hooks/useDDragon";

// Re-export lib utilities
export * from "./lib/ddragon";
export * from "./lib/constants";
