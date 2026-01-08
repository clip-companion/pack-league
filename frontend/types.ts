/**
 * Type Strategy for Match Data
 * =============================
 *
 * This pack supports two API formats for backwards compatibility:
 *
 * LEGACY FORMAT (get_matches):
 *   - `LeagueMatch` / `TFTMatch` - flat objects extending `BaseMatch`
 *   - Match fields at top level (id, playedAt, result, etc.)
 *   - Pack details in `details` field
 *
 * NEW FORMAT (get_matches_v2):
 *   - `LeagueMatchV2` / `TFTMatchV2` - uses `MatchWithDetails<TDetails>`
 *   - Core match data in `core` field (from host matches table)
 *   - Pack details in `details` field (parsed, not JSON strings)
 *
 * UNION TYPES (accept either format):
 *   - `AnyLeagueMatch` = `LeagueMatch | LeagueMatchV2`
 *   - `AnyTFTMatch` = `TFTMatch | TFTMatchV2`
 *   - `AnyPackMatch` = `AnyLeagueMatch | AnyTFTMatch`
 *
 * HELPERS:
 *   - `getMatchCore(match)` - Extract CoreMatchData from either format
 *   - `isLeagueMatchV2(match)` - Type guard for new format
 *   - `isTFTMatch(match)` - Check if match is TFT (either format)
 *
 * Components should use `AnyLeagueMatch` / `AnyTFTMatch` to accept both formats.
 */

import type {
  BaseMatch,
  CoreMatchData,
  MatchWithDetails,
} from "@companion/pack-protocol";

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
 * League of Legends match details (game-specific fields)
 * Stored in the `details` field of BaseMatch
 */
export interface LeagueMatchDetails {
  summonerName: string;
  champion: string;
  championLevel: number;
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
 * League of Legends match data (legacy format)
 * Extends BaseMatch with League-specific details
 * @deprecated Use LeagueMatchV2 for new code
 */
export interface LeagueMatch extends BaseMatch<LeagueMatchDetails> {
  // LeagueMatch gameId is always 1
  gameId: 1;
}

/**
 * League of Legends match data (new format with core/details separation)
 * Uses MatchWithDetails from pack-protocol
 */
export type LeagueMatchV2 = MatchWithDetails<LeagueMatchDetails>;

/**
 * Union type supporting both legacy and new match formats
 */
export type AnyLeagueMatch = LeagueMatch | LeagueMatchV2;

/**
 * Type guard to check if match uses new format (has `core` field)
 */
export function isLeagueMatchV2(match: AnyLeagueMatch): match is LeagueMatchV2 {
  return "core" in match && match.core !== undefined;
}

/**
 * Check if match uses new format (has `core` field)
 * Works for any pack match type
 */
function hasCore(match: unknown): match is { core: CoreMatchData } {
  return (
    typeof match === "object" &&
    match !== null &&
    "core" in match &&
    (match as { core?: unknown }).core !== undefined
  );
}

/**
 * Get core data from either match format
 * Works with both League and TFT matches, legacy and new formats
 */
export function getMatchCore(match: AnyPackMatch | AnyLeagueMatch | AnyTFTMatch): CoreMatchData {
  if (hasCore(match)) {
    return match.core;
  }
  // Legacy format - construct core from flat fields
  const legacy = match as unknown as BaseMatch;
  return {
    id: legacy.id,
    packId: legacy.packId || "",
    subpack: 0, // Pack determines subpack at a higher level
    externalMatchId: null,
    playedAt: legacy.playedAt,
    durationSecs: legacy.durationSecs,
    result: legacy.result,
    isInProgress: false,
    summarySource: null,
    createdAt: legacy.createdAt,
  };
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
 * TFT match details (game-specific fields)
 * Stored in the `details` field of BaseMatch
 */
export interface TFTMatchDetails {
  summonerName: string;
  placement: TFTPlacement;
  gameMode: GameModeContext;
  lpChange: number | null;
  rank: string | null;
  badges: string[];
}

/**
 * TFT match data (legacy format)
 * Extends BaseMatch with TFT-specific details
 * @deprecated Use TFTMatchV2 for new code
 */
export interface TFTMatch extends BaseMatch<TFTMatchDetails> {
  gameId: 1;
}

/**
 * TFT match data (new format with core/details separation)
 * Uses MatchWithDetails from pack-protocol
 */
export type TFTMatchV2 = MatchWithDetails<TFTMatchDetails>;

/**
 * Union type supporting both legacy and new TFT match formats
 */
export type AnyTFTMatch = TFTMatch | TFTMatchV2;

/**
 * Union type for any match from this pack (legacy format)
 */
export type PackMatch = LeagueMatch | TFTMatch;

/**
 * Union type for any match from this pack (supports both formats)
 */
export type AnyPackMatch = AnyLeagueMatch | AnyTFTMatch;

/**
 * Type guard to check if a match is a TFT match
 * Works with both legacy (PackMatch) and new (AnyPackMatch) formats
 */
export function isTFTMatch(match: PackMatch | AnyPackMatch): match is TFTMatch | TFTMatchV2 {
  const gameMode = match.details?.gameMode;
  if (typeof gameMode === "object" && gameMode !== null) {
    return (gameMode as GameModeContext).modeKey === "TFT";
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
