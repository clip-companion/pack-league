/**
 * Live Client API Client
 *
 * TypeScript client for the League of Legends Live Client Data API.
 * This API runs on https://127.0.0.1:2999 when a game is in progress
 * and provides real-time game data.
 *
 * NOTE: The Live Client API uses a self-signed certificate. In Electron,
 * this is typically handled by the app's certificate bypass configuration.
 * In browsers, the certificate must be manually accepted or the requests
 * will fail.
 */

const LIVE_CLIENT_URL = "https://127.0.0.1:2999";

// ============================================================================
// Types
// ============================================================================

export interface ActivePlayer {
  summonerName: string;
  level: number;
  currentGold: number;
  championStats: ChampionStats;
  fullRunes?: FullRunes;
}

export interface ChampionStats {
  abilityPower: number;
  armor: number;
  attackDamage: number;
  attackSpeed: number;
  healthRegenRate: number;
  maxHealth: number;
}

export interface FullRunes {
  keystone: Rune;
  primaryRuneTree: Rune;
  secondaryRuneTree: Rune;
}

export interface Rune {
  id: number;
  displayName: string;
}

export interface Player {
  summonerName: string;
  championName: string;
  team: string;
  level: number;
  scores: PlayerScores;
  isDead: boolean;
  items: Item[];
  summonerSpells?: SummonerSpells;
  runes?: PlayerRunes;
}

export interface PlayerScores {
  kills: number;
  deaths: number;
  assists: number;
  creepScore: number;
}

export interface Item {
  itemID: number;
  displayName: string;
  slot: number;
  count: number;
}

export interface SummonerSpells {
  summonerSpellOne: SpellInfo;
  summonerSpellTwo: SpellInfo;
}

export interface SpellInfo {
  displayName: string;
}

export interface PlayerRunes {
  keystone: Rune;
  primaryRuneTree: Rune;
  secondaryRuneTree: Rune;
}

export interface GameEvents {
  Events: GameEvent[];
}

export interface GameEvent {
  EventID: number;
  EventName: string;
  EventTime: number;
  KillerName?: string;
  VictimName?: string;
  Assisters?: string[];
}

export interface GameInfo {
  gameMode: string;
  gameTime: number;
  mapName: string;
  mapNumber: number;
  mapTerrain: string;
}

export interface AllGameData {
  activePlayer: ActivePlayer;
  allPlayers: Player[];
  events: GameEvents;
  gameData: GameInfo;
}

// ============================================================================
// Client
// ============================================================================

export class LiveClientApi {
  private baseUrl: string;
  private timeout: number;

  constructor(options?: { baseUrl?: string; timeoutMs?: number }) {
    this.baseUrl = options?.baseUrl ?? LIVE_CLIENT_URL;
    this.timeout = options?.timeoutMs ?? 2000;
  }

  private async fetch<T>(endpoint: string): Promise<T> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(`${this.baseUrl}${endpoint}`, {
        signal: controller.signal,
      });

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } finally {
      clearTimeout(timeoutId);
    }
  }

  /**
   * Get all game data in a single request.
   * This is the most efficient way to get all data.
   */
  async getAllGameData(): Promise<AllGameData> {
    return this.fetch<AllGameData>("/liveclientdata/allgamedata");
  }

  /**
   * Get data about the active player (local user).
   */
  async getActivePlayer(): Promise<ActivePlayer> {
    return this.fetch<ActivePlayer>("/liveclientdata/activeplayer");
  }

  /**
   * Get all game events (kills, objectives, etc).
   */
  async getEvents(): Promise<GameEvents> {
    return this.fetch<GameEvents>("/liveclientdata/eventdata");
  }

  /**
   * Check if a game is currently active.
   * Returns true if the Live Client API is responsive.
   */
  async isGameActive(): Promise<boolean> {
    try {
      await this.getActivePlayer();
      return true;
    } catch {
      return false;
    }
  }
}

// ============================================================================
// Singleton instance
// ============================================================================

let defaultClient: LiveClientApi | null = null;

/**
 * Get the default Live Client API instance.
 */
export function getLiveClient(): LiveClientApi {
  if (!defaultClient) {
    defaultClient = new LiveClientApi();
  }
  return defaultClient;
}

// ============================================================================
// Convenience functions
// ============================================================================

/**
 * Get all game data.
 * @throws Error if game is not active or request fails.
 */
export async function getAllGameData(): Promise<AllGameData> {
  return getLiveClient().getAllGameData();
}

/**
 * Get the active player data.
 * @throws Error if game is not active or request fails.
 */
export async function getActivePlayer(): Promise<ActivePlayer> {
  return getLiveClient().getActivePlayer();
}

/**
 * Get all game events.
 * @throws Error if game is not active or request fails.
 */
export async function getEvents(): Promise<GameEvents> {
  return getLiveClient().getEvents();
}

/**
 * Check if a game is currently active.
 */
export async function isGameActive(): Promise<boolean> {
  return getLiveClient().isGameActive();
}
