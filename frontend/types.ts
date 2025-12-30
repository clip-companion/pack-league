import type { BaseMatch } from "./packs/types";

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
