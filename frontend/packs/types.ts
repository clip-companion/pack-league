/**
 * Pack Types
 *
 * These types define the interface between game packs and the host application.
 * They are duplicated here for standalone pack builds.
 */

import type { ComponentType } from "react";

/**
 * Base match type that all game packs extend.
 */
export interface BaseMatch {
  id: string;
  gameId: number;
  playedAt: string;
  durationSecs: number;
  result: "win" | "loss" | "remake";
  createdAt: string;
}

/**
 * Props for match card components.
 */
export interface MatchCardProps<TMatch = BaseMatch> {
  match: TMatch;
  isSelected?: boolean;
  onClick?: () => void;
}

/**
 * Props for live match card components.
 */
export interface LiveMatchCardProps<TLiveMatch = unknown> {
  match: TLiveMatch;
}

/**
 * Resource loading state for a game pack.
 */
export interface ResourceState {
  loading: boolean;
  ready: boolean;
  progress?: number;
  currentItem?: string;
  version?: string | null;
}

/**
 * Resource management API for game packs.
 */
export interface GamePackResources {
  isReady: () => boolean;
  init: () => Promise<void>;
  getState: () => ResourceState;
  onStateChange: (callback: (state: ResourceState) => void) => () => void;
}

/**
 * Interface that each game pack must implement.
 */
export interface GamePack<TMatch extends BaseMatch = BaseMatch, TLiveMatch = unknown> {
  gameId: number;
  slug: string;
  MatchCard: ComponentType<MatchCardProps<TMatch>>;
  LiveMatchCard?: ComponentType<LiveMatchCardProps<TLiveMatch>>;
  resources?: GamePackResources;
  AssetsStatus?: ComponentType;
  isMatch: (match: BaseMatch) => match is TMatch;
  initAssets?: () => Promise<void>;
}
