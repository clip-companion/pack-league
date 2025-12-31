import { motion } from "framer-motion";
import { cn } from "./lib/cn";
import {
  formatTimeAgo,
  formatGameDurationFull,
  formatKDARatio,
  getKDALabel,
  formatLPChange,
} from "./lib/formatters";
import {
  getChampionIconUrl,
  getSpellIconUrl,
  getItemIconUrl,
  getKeystoneIconUrl,
  getRuneTreeIconUrl,
  KEYSTONE_MAP,
  normalizeSpellName,
  normalizeKeystoneName,
  normalizeRuneTreeName,
} from "./lib/ddragon";
import { cardHover, springGentle } from "./lib/animations";
import { GameIcon } from "./components/GameIcon";
import { useDDragonReady } from "./hooks/useDDragon";
import type { LeagueMatch } from "./types";

interface MatchCardProps {
  match: LeagueMatch;
  clipCount?: number;
  isSelected?: boolean;
  onClick?: () => void;
}

export function MatchCard({ match, isSelected, onClick }: MatchCardProps) {
  // Subscribe to DDragon ready state to re-render when icons become available
  const ddReady = useDDragonReady();

  const isWin = match.result === "win";
  const kdaLabel = getKDALabel(match.kills, match.deaths, match.assists);
  const lpChange = formatLPChange(match.lpChange);

  // Show loading skeleton while DDragon initializes
  if (!ddReady) {
    return (
      <div className={cn(
        "w-full rounded-lg border overflow-hidden bg-card animate-pulse",
        isSelected && "border-primary/50"
      )}>
        <div className="flex h-[88px]">
          <div className="w-1 shrink-0 bg-muted" />
          <div className="flex-1 p-3 flex items-center gap-4">
            <div className="w-12 h-12 rounded-full bg-muted" />
            <div className="flex-1 space-y-2">
              <div className="h-4 w-24 bg-muted rounded" />
              <div className="h-3 w-16 bg-muted rounded" />
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Normalize names to handle both IDs and names (backwards compatibility)
  const spell1Name = normalizeSpellName(match.summonerSpell1);
  const spell2Name = normalizeSpellName(match.summonerSpell2);
  const keystoneName = normalizeKeystoneName(match.keystoneRune);
  const secondaryTreeName = normalizeRuneTreeName(match.secondaryTree);

  const blueTeam = match.participants.filter((p) => p.team === "blue");
  const redTeam = match.participants.filter((p) => p.team === "red");

  return (
    <motion.button
      onClick={onClick}
      variants={cardHover}
      initial="initial"
      whileHover="hover"
      whileTap="tap"
      transition={springGentle}
      className={cn(
        "w-full text-left rounded-lg border overflow-hidden",
        "hover:border-primary/30 transition-colors",
        isSelected && "border-primary/50 shadow-md",
        !isSelected && "bg-card"
      )}
    >
      <div className="flex">
        {/* Win/Loss indicator bar - left side */}
        <div className={cn("w-1 shrink-0", isWin ? "bg-win" : "bg-loss")} />

        {/* Game info column - content centered vertically */}
        <div className="w-24 shrink-0 p-2 flex flex-col justify-center overflow-hidden">
          <div className="text-xs font-medium text-muted-foreground truncate">
            {match.gameMode}
          </div>
          <div className="text-xs text-muted-foreground">
            {formatTimeAgo(match.playedAt)}
          </div>
          <div className="mt-1 flex items-baseline gap-1">
            <span
              className={cn(
                "text-sm font-semibold",
                isWin ? "text-win" : "text-loss"
              )}
            >
              {isWin ? "Victory" : "Defeat"}
            </span>
            {lpChange && (
              <span
                className={cn(
                  "text-xs font-medium",
                  match.lpChange && match.lpChange > 0
                    ? "text-win"
                    : "text-loss"
                )}
              >
                {lpChange}
              </span>
            )}
          </div>
          <div className="text-xs text-muted-foreground">
            {formatGameDurationFull(match.durationSecs)}
          </div>
        </div>

        {/* Center content area - vertically centers champ+kda+items */}
        <div className="py-2 pl-2 flex items-center border-l border-border/30 shrink-0">
          {/* Content box: champ+kda on top, items below */}
          <div className="flex flex-col">
            {/* Top row: Champ, KDA */}
            <div className="flex items-start gap-2">

            {/* Champion portrait with summoners/runes */}
            <div className="flex items-center gap-1.5 shrink-0">
              {/* Champion icon */}
              <div className="relative">
                <div
                  className={cn(
                    "ring-2",
                    isWin ? "ring-win/60" : "ring-loss/60"
                  )}
                  style={{ borderRadius: '50%' }}
                >
                  <GameIcon
                    src={getChampionIconUrl(match.champion)}
                    alt={match.champion}
                    size={48}
                    shape="circle"
                    className="scale-110"
                  />
                </div>
                {/* Level badge */}
                <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 bg-background border rounded px-1 text-[9px] font-bold">
                  {match.championLevel}
                </div>
              </div>

              {/* Summoner spells & Runes in 2x2 grid */}
              <div className="flex flex-col gap-0.5">
                <div className="flex gap-0.5">
                  <GameIcon
                    src={getSpellIconUrl(spell1Name)}
                    alt={spell1Name}
                    size={20}
                    title={spell1Name}
                  />
                  <GameIcon
                    src={getKeystoneIconUrl(keystoneName)}
                    alt={keystoneName}
                    size={20}
                    title={keystoneName}
                    fallback={KEYSTONE_MAP[keystoneName]?.abbrev || keystoneName.slice(0, 1)}
                  />
                </div>
                <div className="flex gap-0.5">
                  <GameIcon
                    src={getSpellIconUrl(spell2Name)}
                    alt={spell2Name}
                    size={20}
                    title={spell2Name}
                  />
                  <GameIcon
                    src={getRuneTreeIconUrl(secondaryTreeName)}
                    alt={secondaryTreeName}
                    size={20}
                    title={secondaryTreeName}
                  />
                </div>
              </div>
            </div>

            {/* KDA */}
            <div className="shrink-0 text-center">
              <div className="text-base font-bold tracking-tight">
                <span className="text-foreground">{match.kills}</span>
                <span className="text-muted-foreground/60"> / </span>
                <span className="text-loss">{match.deaths}</span>
                <span className="text-muted-foreground/60"> / </span>
                <span className="text-foreground">{match.assists}</span>
              </div>
              <div className="text-[10px] text-muted-foreground">
                {formatKDARatio(match.kills, match.deaths, match.assists)}:1 KDA
              </div>
              {kdaLabel && (
                <div
                  className={cn(
                    "text-[10px] font-semibold",
                    kdaLabel === "Perfect" && "text-yellow-500",
                    kdaLabel === "Legendary" && "text-orange-500",
                    kdaLabel !== "Perfect" && kdaLabel !== "Legendary" && "text-blue-500"
                  )}
                >
                  {kdaLabel}
                </div>
              )}
            </div>

            </div>

            {/* Items row - snaps to bottom of content box */}
            <div className="flex items-center mt-1">
            <div className="flex gap-0.5">
              {[0, 1, 2, 3, 4, 5].map((i) => {
                const itemName = match.items[i] || "";
                const hasItem = itemName.length > 0;
                const itemUrl = hasItem ? getItemIconUrl(itemName) : null;
                return hasItem ? (
                  <GameIcon
                    key={i}
                    src={itemUrl}
                    alt={itemName}
                    size={20}
                    title={itemName}
                  />
                ) : (
                  <div
                    key={i}
                    className="w-5 h-5 rounded bg-black/40 border border-white/10"
                    title="Empty"
                  />
                );
              })}
              {/* Trinket */}
              {match.trinket ? (
                <GameIcon
                  src={getItemIconUrl(match.trinket)}
                  alt={match.trinket}
                  size={20}
                  shape="circle"
                  title={match.trinket}
                />
              ) : (
                <div
                  className="w-5 h-5 rounded-full bg-black/40 border border-white/10"
                  title="No trinket"
                />
              )}
            </div>
          </div>
          </div>
        </div>

        {/* Stats + Badges column - content centered vertically */}
        <div className="shrink-0 py-2 px-2 text-[10px] space-y-0.5 flex flex-col justify-center">
          <div className="text-muted-foreground">
            P/Kill{" "}
            <span className="text-foreground font-medium">
              {match.killParticipation}%
            </span>
          </div>
          <div className="text-muted-foreground">
            CS{" "}
            <span className="text-foreground font-medium">{match.cs}</span>
            <span className="text-muted-foreground/70">
              {" "}({match.csPerMin.toFixed(1)})
            </span>
          </div>
          {match.rank && (
            <div className="text-primary font-medium">{match.rank}</div>
          )}
          {/* Badges */}
          {match.badges.length > 0 && (
            <div className="flex flex-col gap-0.5 pt-1">
              {match.badges.slice(0, 2).map((badge) => (
                <span
                  key={badge}
                  className={cn(
                    "px-1.5 py-0.5 rounded text-[9px] font-medium whitespace-nowrap",
                    badge.includes("Penta") && "bg-yellow-500/20 text-yellow-500",
                    badge.includes("Quadra") && "bg-orange-500/20 text-orange-500",
                    badge.includes("Triple") && "bg-purple-500/20 text-purple-500",
                    badge.includes("Double") && "bg-blue-500/20 text-blue-500",
                    badge === "MVP" && "bg-yellow-500/20 text-yellow-500",
                    badge === "Legendary" && "bg-orange-500/20 text-orange-500",
                    badge === "Perfect" && "bg-emerald-500/20 text-emerald-500",
                    !badge.includes("Penta") &&
                      !badge.includes("Quadra") &&
                      !badge.includes("Triple") &&
                      !badge.includes("Double") &&
                      badge !== "MVP" &&
                      badge !== "Legendary" &&
                      badge !== "Perfect" &&
                      "bg-slate-500/20 text-slate-400"
                  )}
                >
                  {badge}
                </span>
              ))}
            </div>
          )}
        </div>

        {/* Right side: Team Compositions - content centered vertically */}
        <div className="flex gap-2 p-2 pl-0 shrink-0 items-center">
          {/* Blue Team */}
          <div className="flex flex-col gap-0.5">
            {blueTeam.slice(0, 5).map((p, i) => (
              <div
                key={i}
                className={cn(
                  "flex items-center gap-1",
                  p.summonerName === match.summonerName && "bg-primary/10 rounded-sm px-1 -mx-1"
                )}
              >
                <GameIcon
                  src={getChampionIconUrl(p.champion)}
                  alt={p.champion}
                  size={16}
                  className={cn(
                    p.summonerName === match.summonerName && "ring-1 ring-primary"
                  )}
                />
                <span
                  className={cn(
                    "text-[10px] leading-none",
                    p.summonerName === match.summonerName
                      ? "text-foreground font-medium"
                      : "text-muted-foreground"
                  )}
                >
                  {p.summonerName}
                </span>
              </div>
            ))}
          </div>

          {/* Red Team */}
          <div className="flex flex-col gap-0.5">
            {redTeam.slice(0, 5).map((p, i) => (
              <div
                key={i}
                className={cn(
                  "flex items-center gap-1",
                  p.summonerName === match.summonerName && "bg-primary/10 rounded-sm px-1 -mx-1"
                )}
              >
                <GameIcon
                  src={getChampionIconUrl(p.champion)}
                  alt={p.champion}
                  size={16}
                  className={cn(
                    p.summonerName === match.summonerName && "ring-1 ring-primary"
                  )}
                />
                <span
                  className={cn(
                    "text-[10px] leading-none",
                    p.summonerName === match.summonerName
                      ? "text-foreground font-medium"
                      : "text-muted-foreground"
                  )}
                >
                  {p.summonerName}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </motion.button>
  );
}
