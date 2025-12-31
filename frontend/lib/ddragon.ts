// Data Dragon CDN utilities for League of Legends assets
// Dynamically loads all data from Riot's API with persistent caching
//
// All data is fetched from DDragon and cached to disk via Electron IPC.
// This means the app survives game updates without code changes.

const DDRAGON_BASE = "https://ddragon.leagueoflegends.com/cdn";

// ============================================
// Cache State
// ============================================

interface CacheMetadata {
  version: string;
  fetchedAt: number;
}

interface SpellData {
  id: string;      // e.g., "SummonerFlash"
  name: string;    // e.g., "Flash"
  key: string;     // e.g., "4" (spell ID)
}

interface RuneData {
  id: number;
  key: string;
  name: string;
  icon: string;
}

interface RuneTreeData {
  id: number;
  key: string;
  name: string;
  icon: string;
  slots: Array<{ runes: RuneData[] }>;
}

// In-memory caches (populated from disk or network)
let cachedVersion: string | null = null;
let cachedItems: Map<string, number> | null = null;
let cachedChampions: Map<string, string> | null = null;
let cachedSpells: Map<string, SpellData> | null = null;  // key: name or ID
let cachedRunes: Map<string, RuneData> | null = null;    // key: name or ID
let cachedRuneTrees: Map<string, RuneTreeData> | null = null; // key: name or ID
let initPromise: Promise<void> | null = null;

// Loading state for UI indicator
let isLoading = false;
let isReady = false;
let loadProgress = { total: 4, loaded: 0, currentItem: '' };

interface DDragonState {
  loading: boolean;
  ready: boolean;
  progress: { total: number; loaded: number; currentItem: string };
  version: string | null;
}

const stateListeners = new Set<(state: DDragonState) => void>();

function getState(): DDragonState {
  return {
    loading: isLoading,
    ready: isReady,
    progress: { ...loadProgress },
    version: cachedVersion,
  };
}

function notifyListeners() {
  const state = getState();
  stateListeners.forEach(cb => cb(state));
}

export function onStateChange(callback: (state: DDragonState) => void): () => void {
  stateListeners.add(callback);
  callback(getState());
  return () => stateListeners.delete(callback);
}

// Legacy loading change listener
export function onLoadingChange(callback: (loading: boolean) => void): () => void {
  return onStateChange(state => callback(state.loading));
}

function setLoading(loading: boolean) {
  isLoading = loading;
  notifyListeners();
}

function setProgress(loaded: number, currentItem: string) {
  loadProgress = { ...loadProgress, loaded, currentItem };
  notifyListeners();
}

function setReady(ready: boolean) {
  isReady = ready;
  notifyListeners();
}

// ============================================
// Static Mappings (for backwards compatibility)
// ============================================

// Champion name to Data Dragon key mapping (handles special cases not in API)
const CHAMPION_KEY_OVERRIDES: Record<string, string> = {
  // Display names → DDragon ID
  "Lee Sin": "LeeSin",
  "Kai'Sa": "Kaisa",
  "Kha'Zix": "Khazix",
  "Cho'Gath": "Chogath",
  "Vel'Koz": "Velkoz",
  "Rek'Sai": "RekSai",
  "Kog'Maw": "KogMaw",
  "Bel'Veth": "Belveth",
  "K'Sante": "KSante",
  "Wukong": "MonkeyKing",
  "Renata Glasc": "Renata",
  "Nunu & Willump": "Nunu",
  "Jarvan IV": "JarvanIV",
  "Jarvan": "JarvanIV",
  "Dr. Mundo": "DrMundo",
  "Master Yi": "MasterYi",
  "Miss Fortune": "MissFortune",
  "Tahm Kench": "TahmKench",
  "Twisted Fate": "TwistedFate",
  "Xin Zhao": "XinZhao",
  "Aurelion Sol": "AurelionSol",
  "LeBlanc": "Leblanc",
  // Seed data format (no apostrophes, mixed case) → DDragon ID
  "ChoGath": "Chogath",
  "VelKoz": "Velkoz",
  "KogMaw": "KogMaw",
  "BelVeth": "Belveth",
  "RekSai": "RekSai",
};

// Item name aliases (maps old/shortened names to current DDragon names)
const ITEM_ALIASES: Record<string, string> = {
  "Luden's": "Luden's Companion",
  "Ludens": "Luden's Companion",
  "Zhonya's": "Zhonya's Hourglass",
  "Banshee's": "Banshee's Veil",
  "Rabadon's": "Rabadon's Deathcap",
  "Youmuu's": "Youmuu's Ghostblade",
  "Serylda's": "Serylda's Grudge",
  "Lord Dominik's": "Lord Dominik's Regards",
  "Ionian Boots": "Ionian Boots of Lucidity",
  "Sorc Shoes": "Sorcerer's Shoes",
  "Oracle": "Oracle Lens",
  "Sweeper": "Oracle Lens",
  "Farsight": "Farsight Alteration",
  "Luden's Tempest": "Luden's Companion",
  "Luden's Echo": "Luden's Companion",
  // Case variations
  "Blade of the Ruined King": "Blade of The Ruined King",
  // Collector variant
  "Collector": "The Collector",
};

// Rune tree colors (static, unlikely to change)
export const RUNE_TREES: Record<string, { color: string; abbrev: string }> = {
  Precision: { color: "#C8AA6E", abbrev: "P" },
  Domination: { color: "#D44242", abbrev: "D" },
  Sorcery: { color: "#9FAAFC", abbrev: "S" },
  Resolve: { color: "#A1D586", abbrev: "R" },
  Inspiration: { color: "#49AAB8", abbrev: "I" },
};

// Keystone abbreviations (for compact display)
export const KEYSTONE_MAP: Record<string, { tree: string; abbrev: string }> = {
  "Press the Attack": { tree: "Precision", abbrev: "PtA" },
  "Lethal Tempo": { tree: "Precision", abbrev: "LT" },
  "Fleet Footwork": { tree: "Precision", abbrev: "FF" },
  Conqueror: { tree: "Precision", abbrev: "Conq" },
  Electrocute: { tree: "Domination", abbrev: "Elec" },
  Predator: { tree: "Domination", abbrev: "Pred" },
  "Dark Harvest": { tree: "Domination", abbrev: "DH" },
  "Hail of Blades": { tree: "Domination", abbrev: "HoB" },
  "Summon Aery": { tree: "Sorcery", abbrev: "Aery" },
  "Arcane Comet": { tree: "Sorcery", abbrev: "Comet" },
  "Phase Rush": { tree: "Sorcery", abbrev: "PR" },
  "Grasp of the Undying": { tree: "Resolve", abbrev: "Grasp" },
  Aftershock: { tree: "Resolve", abbrev: "AS" },
  Guardian: { tree: "Resolve", abbrev: "Guard" },
  "Glacial Augment": { tree: "Inspiration", abbrev: "Glacial" },
  "Unsealed Spellbook": { tree: "Inspiration", abbrev: "Spell" },
  "First Strike": { tree: "Inspiration", abbrev: "FS" },
};

// Spell name to DDragon ID mapping (for spells with non-standard IDs)
const SPELL_ID_OVERRIDES: Record<string, string> = {
  Cleanse: "SummonerBoost",
  Ghost: "SummonerHaste",
  Heal: "SummonerHeal",
  Barrier: "SummonerBarrier",
  Exhaust: "SummonerExhaust",
  Flash: "SummonerFlash",
  Teleport: "SummonerTeleport",
  Smite: "SummonerSmite",
  Ignite: "SummonerDot",
  Clarity: "SummonerMana",
  Snowball: "SummonerSnowball",
};

// Keystone name aliases (maps camelCase/compact names to DDragon names)
const KEYSTONE_ALIASES: Record<string, string> = {
  DarkHarvest: "Dark Harvest",
  HailOfBlades: "Hail of Blades",
  PressTheAttack: "Press the Attack",
  LethalTempo: "Lethal Tempo",
  FleetFootwork: "Fleet Footwork",
  SummonAery: "Summon Aery",
  ArcaneComet: "Arcane Comet",
  PhaseRush: "Phase Rush",
  GraspOfTheUndying: "Grasp of the Undying",
  GlacialAugment: "Glacial Augment",
  UnsealedSpellbook: "Unsealed Spellbook",
  FirstStrike: "First Strike",
};

// ============================================
// Cache Utilities
// ============================================

// Game slug for cache namespacing
const GAME_SLUG = 'league';

async function readCache<T>(filename: string): Promise<T | null> {
  if (typeof window !== 'undefined' && window.electronAPI?.cache) {
    try {
      const data = await window.electronAPI.cache.read<T>(GAME_SLUG, filename);
      if (!data) {
        console.log(`[DDragon] Cache miss for: ${filename}`);
      }
      return data;
    } catch (err) {
      console.warn(`[DDragon] Cache read error for ${filename}:`, err);
      return null;
    }
  }
  console.log('[DDragon] Cache API not available (not in Electron?)');
  return null;
}

async function writeCache(filename: string, data: unknown): Promise<void> {
  if (typeof window !== 'undefined' && window.electronAPI?.cache) {
    try {
      await window.electronAPI.cache.write(GAME_SLUG, filename, data);
      console.log(`[DDragon] Cached: ${filename}`);
    } catch (err) {
      console.warn(`[DDragon] Cache write error for ${filename}:`, err);
    }
  }
}

async function isCacheValid(): Promise<{ valid: boolean; version?: string }> {
  const metadata = await readCache<CacheMetadata>('version.json');
  if (!metadata) return { valid: false };

  // Cache is valid for 24 hours
  const age = Date.now() - metadata.fetchedAt;
  const maxAge = 24 * 60 * 60 * 1000;

  if (age > maxAge) {
    return { valid: false };
  }

  return { valid: true, version: metadata.version };
}

// ============================================
// Data Loaders
// ============================================

async function getLatestVersion(): Promise<string> {
  if (cachedVersion) return cachedVersion;

  // Check disk cache first
  const cacheStatus = await isCacheValid();
  if (cacheStatus.valid && cacheStatus.version) {
    cachedVersion = cacheStatus.version;
    return cachedVersion;
  }

  try {
    const response = await fetch("https://ddragon.leagueoflegends.com/api/versions.json");
    const versions = await response.json() as string[];
    cachedVersion = versions[0];
    console.log(`[DDragon] Using version: ${cachedVersion}`);
    return cachedVersion;
  } catch (error) {
    console.warn("[DDragon] Failed to fetch versions, using fallback:", error);
    cachedVersion = "14.24.1";
    return cachedVersion;
  }
}

async function loadItems(): Promise<void> {
  // Try disk cache first
  const cached = await readCache<Record<string, number>>('items.json');
  if (cached) {
    cachedItems = new Map(Object.entries(cached));
    // Add aliases
    for (const [alias, canonical] of Object.entries(ITEM_ALIASES)) {
      const id = cachedItems.get(canonical);
      if (id) cachedItems.set(alias, id);
    }
    console.log(`[DDragon] Loaded ${cachedItems.size} items from cache`);
    return;
  }

  // Fetch from network
  const version = await getLatestVersion();
  const url = `${DDRAGON_BASE}/${version}/data/en_US/item.json`;

  try {
    const response = await fetch(url);
    const data = await response.json() as { data: Record<string, { name: string }> };

    cachedItems = new Map();
    const toCache: Record<string, number> = {};

    for (const [id, item] of Object.entries(data.data)) {
      const numId = parseInt(id, 10);
      cachedItems.set(item.name, numId);
      toCache[item.name] = numId;
    }

    // Add aliases
    for (const [alias, canonical] of Object.entries(ITEM_ALIASES)) {
      const id = cachedItems.get(canonical);
      if (id) cachedItems.set(alias, id);
    }

    await writeCache('items.json', toCache);
    console.log(`[DDragon] Loaded ${cachedItems.size} items from network`);
  } catch (error) {
    console.error("[DDragon] Failed to load items:", error);
    cachedItems = new Map();
  }
}

async function loadChampions(): Promise<void> {
  // Try disk cache first
  const cached = await readCache<Record<string, string>>('champions.json');
  if (cached) {
    cachedChampions = new Map(Object.entries(cached));
    // Add overrides
    for (const [name, key] of Object.entries(CHAMPION_KEY_OVERRIDES)) {
      cachedChampions.set(name, key);
    }
    console.log(`[DDragon] Loaded ${cachedChampions.size} champions from cache`);
    return;
  }

  // Fetch from network
  const version = await getLatestVersion();
  const url = `${DDRAGON_BASE}/${version}/data/en_US/champion.json`;

  try {
    const response = await fetch(url);
    const data = await response.json() as { data: Record<string, { name: string; id: string }> };

    cachedChampions = new Map();
    const toCache: Record<string, string> = {};

    for (const champion of Object.values(data.data)) {
      cachedChampions.set(champion.name, champion.id);
      toCache[champion.name] = champion.id;
    }

    // Add overrides
    for (const [name, key] of Object.entries(CHAMPION_KEY_OVERRIDES)) {
      cachedChampions.set(name, key);
    }

    await writeCache('champions.json', toCache);
    console.log(`[DDragon] Loaded ${cachedChampions.size} champions from network`);
  } catch (error) {
    console.error("[DDragon] Failed to load champions:", error);
    cachedChampions = new Map();
  }
}

async function loadSummonerSpells(): Promise<void> {
  // Try disk cache first
  const cached = await readCache<Record<string, SpellData>>('spells.json');
  if (cached) {
    cachedSpells = new Map();
    for (const [key, spell] of Object.entries(cached)) {
      cachedSpells.set(key, spell);
    }
    console.log(`[DDragon] Loaded ${cachedSpells.size} spells from cache`);
    return;
  }

  // Fetch from network
  const version = await getLatestVersion();
  const url = `${DDRAGON_BASE}/${version}/data/en_US/summoner.json`;

  try {
    const response = await fetch(url);
    const data = await response.json() as {
      data: Record<string, { id: string; name: string; key: string }>
    };

    cachedSpells = new Map();
    const toCache: Record<string, SpellData> = {};

    for (const spell of Object.values(data.data)) {
      const spellData: SpellData = {
        id: spell.id,
        name: spell.name,
        key: spell.key,
      };
      // Index by multiple keys for easy lookup
      cachedSpells.set(spell.name, spellData);      // "Flash"
      cachedSpells.set(spell.id, spellData);         // "SummonerFlash"
      cachedSpells.set(spell.key, spellData);        // "4"
      toCache[spell.name] = spellData;
    }

    await writeCache('spells.json', toCache);
    console.log(`[DDragon] Loaded ${Object.keys(toCache).length} spells from network`);
  } catch (error) {
    console.error("[DDragon] Failed to load summoner spells:", error);
    cachedSpells = new Map();
  }
}

async function loadRunes(): Promise<void> {
  // Try disk cache first
  const cachedRunesData = await readCache<Record<string, RuneData>>('runes.json');
  const cachedTreesData = await readCache<Record<string, RuneTreeData>>('rune-trees.json');

  if (cachedRunesData && cachedTreesData) {
    cachedRunes = new Map(Object.entries(cachedRunesData));
    cachedRuneTrees = new Map(Object.entries(cachedTreesData));
    console.log(`[DDragon] Loaded ${cachedRunes.size} runes from cache`);
    return;
  }

  // Fetch from network
  const version = await getLatestVersion();
  const url = `${DDRAGON_BASE}/${version}/data/en_US/runesReforged.json`;

  try {
    const response = await fetch(url);
    const data = await response.json() as RuneTreeData[];

    cachedRunes = new Map();
    cachedRuneTrees = new Map();
    const runesToCache: Record<string, RuneData> = {};
    const treesToCache: Record<string, RuneTreeData> = {};

    for (const tree of data) {
      // Store tree data
      cachedRuneTrees.set(tree.name, tree);
      cachedRuneTrees.set(tree.key, tree);
      cachedRuneTrees.set(String(tree.id), tree);
      treesToCache[tree.name] = tree;

      // Store individual runes
      for (const slot of tree.slots) {
        for (const rune of slot.runes) {
          cachedRunes.set(rune.name, rune);
          cachedRunes.set(rune.key, rune);
          cachedRunes.set(String(rune.id), rune);
          runesToCache[rune.name] = rune;
        }
      }
    }

    await writeCache('runes.json', runesToCache);
    await writeCache('rune-trees.json', treesToCache);
    console.log(`[DDragon] Loaded ${Object.keys(runesToCache).length} runes from network`);
  } catch (error) {
    console.error("[DDragon] Failed to load runes:", error);
    cachedRunes = new Map();
    cachedRuneTrees = new Map();
  }
}

// ============================================
// Initialization
// ============================================

export async function initDDragon(): Promise<void> {
  if (initPromise) {
    console.log('[DDragon] Init already in progress, returning existing promise');
    return initPromise;
  }

  console.log('[DDragon] Starting initialization...');
  setLoading(true);
  setReady(false);

  initPromise = (async () => {
    try {
      setProgress(0, 'Checking version...');
      const version = await getLatestVersion();

      // Update cache metadata
      await writeCache('version.json', {
        version,
        fetchedAt: Date.now(),
      } as CacheMetadata);

      // Load data sequentially to track progress
      setProgress(1, 'Loading items...');
      await loadItems();

      setProgress(2, 'Loading champions...');
      await loadChampions();

      setProgress(3, 'Loading spells...');
      await loadSummonerSpells();

      setProgress(4, 'Loading runes...');
      await loadRunes();

      console.log('[DDragon] Initialization complete');
      setReady(true);
    } catch (err) {
      console.error('[DDragon] Initialization failed:', err);
      throw err;
    } finally {
      setLoading(false);
    }
  })();

  return initPromise;
}

// NOTE: Lazy loading - initDDragon() must be called explicitly when needed.
// This prevents loading assets for games the user hasn't played.

// ============================================
// Public API - Icon URLs
// ============================================

export function getChampionKey(championName: string): string {
  if (CHAMPION_KEY_OVERRIDES[championName]) {
    return CHAMPION_KEY_OVERRIDES[championName];
  }
  if (cachedChampions?.has(championName)) {
    return cachedChampions.get(championName)!;
  }
  return championName.replace(/['\s]/g, "");
}

export function getChampionIconUrl(championName: string): string {
  const version = cachedVersion || "14.24.1";
  const key = getChampionKey(championName);
  return `${DDRAGON_BASE}/${version}/img/champion/${key}.png`;
}

export function getSpellIconUrl(spellNameOrId: string): string {
  const version = cachedVersion || "14.24.1";

  // Try to find in cached spells
  const spell = cachedSpells?.get(spellNameOrId);
  if (spell) {
    return `${DDRAGON_BASE}/${version}/img/spell/${spell.id}.png`;
  }

  // Use override map for known spells (handles non-standard IDs like Cleanse→SummonerBoost)
  const overrideId = SPELL_ID_OVERRIDES[spellNameOrId];
  if (overrideId) {
    return `${DDRAGON_BASE}/${version}/img/spell/${overrideId}.png`;
  }

  // Fallback: guess the ID format
  const id = spellNameOrId.startsWith("Summoner")
    ? spellNameOrId
    : `Summoner${spellNameOrId}`;
  return `${DDRAGON_BASE}/${version}/img/spell/${id}.png`;
}

export function getItemIconUrl(itemName: string): string | null {
  const canonical = ITEM_ALIASES[itemName] || itemName;
  const version = cachedVersion || "14.24.1";

  const itemId = cachedItems?.get(canonical);
  if (itemId) {
    return `${DDRAGON_BASE}/${version}/img/item/${itemId}.png`;
  }

  return null;
}

export function getItemIconUrlById(itemId: number): string {
  const version = cachedVersion || "14.24.1";
  return `${DDRAGON_BASE}/${version}/img/item/${itemId}.png`;
}

export function getKeystoneIconUrl(keystoneNameOrId: string): string | null {
  // Try alias first (handles camelCase names like "DarkHarvest")
  const canonicalName = KEYSTONE_ALIASES[keystoneNameOrId] || keystoneNameOrId;

  const rune = cachedRunes?.get(canonicalName);
  if (rune?.icon) {
    return `https://ddragon.leagueoflegends.com/cdn/img/${rune.icon}`;
  }
  return null;
}

export function getRuneTreeIconUrl(treeNameOrId: string): string | null {
  const tree = cachedRuneTrees?.get(treeNameOrId);
  if (tree?.icon) {
    return `https://ddragon.leagueoflegends.com/cdn/img/${tree.icon}`;
  }
  return null;
}

// ============================================
// Public API - Colors & Metadata
// ============================================

export function getKeystoneColor(keystoneName: string): string {
  const canonicalName = KEYSTONE_ALIASES[keystoneName] || keystoneName;
  const keystone = KEYSTONE_MAP[canonicalName];
  if (keystone) {
    return RUNE_TREES[keystone.tree]?.color || "#888888";
  }
  // Try to find tree from cached rune data
  const rune = cachedRunes?.get(canonicalName);
  if (rune) {
    // Find which tree this rune belongs to
    for (const [treeName, tree] of cachedRuneTrees?.entries() || []) {
      if (tree.slots?.some(slot => slot.runes?.some(r => r.name === rune.name))) {
        return RUNE_TREES[treeName]?.color || "#888888";
      }
    }
  }
  return "#888888";
}

export function getRuneTreeColor(treeName: string): string {
  return RUNE_TREES[treeName]?.color || "#888888";
}

// ============================================
// Normalization (ID ↔ Name conversion)
// ============================================

export function normalizeSpellName(spellNameOrId: string): string {
  const spell = cachedSpells?.get(spellNameOrId);
  if (spell) return spell.name;

  // Fallback ID map for backwards compatibility
  const SPELL_ID_MAP: Record<string, string> = {
    "1": "Cleanse", "3": "Exhaust", "4": "Flash", "6": "Ghost",
    "7": "Heal", "11": "Smite", "12": "Teleport", "13": "Clarity",
    "14": "Ignite", "21": "Barrier", "32": "Snowball",
  };
  return SPELL_ID_MAP[spellNameOrId] || spellNameOrId;
}

export function normalizeKeystoneName(keystoneNameOrId: string): string {
  // Check for alias first
  const canonicalName = KEYSTONE_ALIASES[keystoneNameOrId] || keystoneNameOrId;

  if (KEYSTONE_MAP[canonicalName]) return canonicalName;

  const rune = cachedRunes?.get(canonicalName);
  if (rune) return rune.name;

  // Fallback ID map for backwards compatibility
  const KEYSTONE_ID_MAP: Record<string, string> = {
    "8005": "Press the Attack", "8008": "Lethal Tempo",
    "8021": "Fleet Footwork", "8010": "Conqueror",
    "8112": "Electrocute", "8124": "Predator",
    "8128": "Dark Harvest", "9923": "Hail of Blades",
    "8214": "Summon Aery", "8229": "Arcane Comet", "8230": "Phase Rush",
    "8437": "Grasp of the Undying", "8439": "Aftershock", "8465": "Guardian",
    "8351": "Glacial Augment", "8360": "Unsealed Spellbook", "8369": "First Strike",
  };
  return KEYSTONE_ID_MAP[keystoneNameOrId] || keystoneNameOrId;
}

export function normalizeRuneTreeName(treeNameOrId: string): string {
  if (RUNE_TREES[treeNameOrId]) return treeNameOrId;

  const tree = cachedRuneTrees?.get(treeNameOrId);
  if (tree) return tree.name;

  // Fallback ID map
  const RUNE_TREE_ID_MAP: Record<string, string> = {
    "8000": "Precision", "8100": "Domination", "8200": "Sorcery",
    "8300": "Inspiration", "8400": "Resolve",
  };
  return RUNE_TREE_ID_MAP[treeNameOrId] || treeNameOrId;
}
