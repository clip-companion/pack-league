import type { BaseMatch } from "@companion/pack-protocol";

/**
 * League of Legends participant in a match
 */
export interface Participant {
  summonerName: string;
  champion: string;
  team: "blue" | "red";
}

/**
 * League of Legends live item data
 */
export interface LiveItem {
  itemId: number;
  name: string;
  slot: number;
}

/**
 * League of Legends live spell data
 */
export interface LiveSpell {
  name: string;
}

/**
 * League of Legends live runes data
 */
export interface LiveRunes {
  keystoneId: number;
  keystoneName: string;
  primaryTreeId: number;
  primaryTreeName: string;
  secondaryTreeId: number;
  secondaryTreeName: string;
}

/**
 * League of Legends live player data
 */
export interface LivePlayer {
  summonerName: string;
  champion: string;
  team: "blue" | "red";
  kills: number;
  deaths: number;
  assists: number;
  cs: number;
  level: number;
  isDead: boolean;
}

/**
 * League of Legends match result
 */
export type MatchResult = "win" | "loss" | "remake";

/**
 * League of Legends match data
 */
export interface LeagueMatch extends BaseMatch {
  // LeagueMatch gameId is always 1
  gameId: 1;
  summonerName: string;
  champion: string;
  championLevel: number;
  result: MatchResult;
  kills: number;
  deaths: number;
  assists: number;
  cs: number;
  csPerMin: number;
  visionScore: number;
  killParticipation: number;
  damageDealt: number;
  gameMode: string;
  lpChange: number | null;
  rank: string | null;
  summonerSpell1: string;
  summonerSpell2: string;
  keystoneRune: string;
  secondaryTree: string;
  items: string[];
  trinket: string | null;
  participants: Participant[];
  badges: string[];
}

/**
 * League of Legends live match data
 */
export interface LeagueLiveMatch {
  summonerName: string;
  champion: string;
  level: number;
  kills: number;
  deaths: number;
  assists: number;
  cs: number;
  currentGold: number;
  gameTimeSecs: number;
  gameMode: string;
  team: "blue" | "red";
  items: LiveItem[];
  trinket: LiveItem | null;
  spell1: LiveSpell | null;
  spell2: LiveSpell | null;
  runes: LiveRunes | null;
  participants: LivePlayer[];
  isDead: boolean;
}

// ============================================================================
// TFT Types
// ============================================================================

/**
 * Game mode context from the backend
 */
export interface GameModeContext {
  modeGuid: string;
  modeKey: string;
  displayName: string;
  queueId: number;
  queueName: string;
  isRanked: boolean;
}

/**
 * TFT match result (placement-based)
 */
export type TFTPlacement = 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8;

/**
 * TFT match data
 */
export interface TFTMatch extends BaseMatch {
  gameId: 1;
  summonerName: string;
  placement: TFTPlacement;
  result: MatchResult; // Still win/loss for top 4 vs bottom 4
  gameMode: GameModeContext;
  lpChange: number | null;
  rank: string | null;
  durationSecs: number;
  badges: string[];
}

/**
 * Union type for any match from this pack
 */
export type PackMatch = LeagueMatch | TFTMatch;

/**
 * Type guard to check if a match is a TFT match
 */
export function isTFTMatch(match: PackMatch): match is TFTMatch {
  const gameMode = (match as any).gameMode;
  if (typeof gameMode === "object" && gameMode !== null) {
    return gameMode.modeKey === "TFT";
  }
  if (typeof gameMode === "string") {
    return gameMode.toUpperCase() === "TFT";
  }
  return false;
}

/**
 * Helper to check if placement is top 4 (considered a "win" in TFT)
 */
export function isTopFour(placement: TFTPlacement): boolean {
  return placement <= 4;
}
