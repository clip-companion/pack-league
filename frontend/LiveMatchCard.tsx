import { useState, useEffect } from "react";
import { motion } from "framer-motion";
import { cn } from "@/lib/cn";
import { formatDuration, formatKDARatio, getKDALabel } from "@/lib/formatters";
import {
  getChampionIconUrl,
  getItemIconUrlById,
  getSpellIconUrl,
  getKeystoneIconUrl,
  getRuneTreeIconUrl,
  getKeystoneColor,
} from "./lib/ddragon";
import { springGentle } from "@/lib/animations";
import type { LeagueLiveMatch, LiveItem } from "./types";

interface LiveMatchCardProps {
  match: LeagueLiveMatch;
}

// Get items sorted by slot (0-5 for regular items)
function getItemsForSlots(items: LiveItem[]): (LiveItem | null)[] {
  const slots: (LiveItem | null)[] = [null, null, null, null, null, null];
  for (const item of items) {
    if (item.slot >= 0 && item.slot < 6) {
      slots[item.slot] = item;
    }
  }
  return slots;
}

// Fallback component for failed images
function ImageWithFallback({
  src,
  alt,
  fallback,
  className,
}: {
  src: string;
  alt: string;
  fallback: string;
  className?: string;
}) {
  const [error, setError] = useState(false);

  if (error) {
    return (
      <div
        className={cn(
          "flex items-center justify-center bg-muted text-[10px] font-bold text-muted-foreground",
          className
        )}
        title={alt}
      >
        {fallback}
      </div>
    );
  }

  return (
    <img
      src={src}
      alt={alt}
      className={className}
      onError={() => setError(true)}
      loading="lazy"
    />
  );
}

// Animated live indicator with pulsing dot
function LiveIndicator() {
  return (
    <div className="flex items-center gap-1.5 px-2 py-0.5 bg-red-500/20 rounded">
      <motion.div
        className="w-2 h-2 rounded-full bg-red-500"
        animate={{
          scale: [1, 1.2, 1],
          opacity: [1, 0.7, 1],
        }}
        transition={{
          duration: 1.5,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />
      <span className="text-xs font-bold text-red-500 uppercase tracking-wide">
        Live
      </span>
    </div>
  );
}

// Game timer that updates visually
function GameTimer({ gameTimeSecs }: { gameTimeSecs: number }) {
  const [displayTime, setDisplayTime] = useState(gameTimeSecs);

  // Update the display time every second (local interpolation between backend updates)
  useEffect(() => {
    setDisplayTime(gameTimeSecs);
    const interval = setInterval(() => {
      setDisplayTime((t) => t + 1);
    }, 1000);
    return () => clearInterval(interval);
  }, [gameTimeSecs]);

  return (
    <div className="text-sm font-mono text-muted-foreground">
      {formatDuration(Math.floor(displayTime))}
    </div>
  );
}

export function LiveMatchCard({ match }: LiveMatchCardProps) {
  const kdaLabel = getKDALabel(match.kills, match.deaths, match.assists);
  const isBlueTeam = match.team === "blue";

  const blueTeam = match.participants.filter((p) => p.team === "blue");
  const redTeam = match.participants.filter((p) => p.team === "red");

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.98 }}
      animate={{ opacity: 1, scale: 1 }}
      exit={{ opacity: 0, scale: 0.98 }}
      transition={springGentle}
      className={cn(
        "w-full text-left rounded-lg border overflow-hidden",
        "bg-gradient-to-r from-card via-card to-card",
        "border-red-500/40 shadow-[0_0_15px_rgba(239,68,68,0.15)]"
      )}
    >
      {/* Animated border glow effect */}
      <motion.div
        className="absolute inset-0 rounded-lg pointer-events-none"
        animate={{
          boxShadow: [
            "inset 0 0 0 1px rgba(239, 68, 68, 0.2)",
            "inset 0 0 0 1px rgba(239, 68, 68, 0.5)",
            "inset 0 0 0 1px rgba(239, 68, 68, 0.2)",
          ],
        }}
        transition={{
          duration: 2,
          repeat: Infinity,
          ease: "easeInOut",
        }}
      />

      <div className="relative flex">
        {/* Live indicator bar - left side */}
        <motion.div
          className="w-1 shrink-0 bg-red-500"
          animate={{
            opacity: [1, 0.5, 1],
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            ease: "easeInOut",
          }}
        />

        {/* Game info column */}
        <div className="w-24 shrink-0 p-2 flex flex-col justify-center overflow-hidden">
          <div className="text-xs font-medium text-muted-foreground truncate">
            {match.gameMode}
          </div>
          <GameTimer gameTimeSecs={match.gameTimeSecs} />
          <div className="mt-1">
            <LiveIndicator />
          </div>
          <div className="text-xs text-muted-foreground mt-1">
            {isBlueTeam ? "Blue Team" : "Red Team"}
          </div>
        </div>

        {/* Center content area */}
        <div className="py-2 pl-2 flex items-center overflow-hidden border-l border-border/30">
          <div className="flex flex-col">
            {/* Top row: Champ, KDA */}
            <div className="flex items-start gap-2">
              {/* Champion portrait */}
              <div className="flex items-center gap-1.5 shrink-0">
                <div className="relative">
                  <div
                    className={cn(
                      "w-12 h-12 rounded-full overflow-hidden ring-2",
                      "ring-red-500/60"
                    )}
                  >
                    <ImageWithFallback
                      src={getChampionIconUrl(match.champion)}
                      alt={match.champion}
                      fallback={match.champion.slice(0, 2).toUpperCase()}
                      className="w-full h-full object-cover scale-110"
                    />
                  </div>
                  {/* Level badge */}
                  <div className="absolute -bottom-1 left-1/2 -translate-x-1/2 bg-background border rounded px-1 text-[9px] font-bold">
                    {match.level}
                  </div>
                </div>

                {/* Summoner Spells & Runes */}
                <div className="flex flex-col gap-0.5">
                  {/* Summoner Spells */}
                  <div className="flex gap-0.5">
                    {match.spell1 ? (
                      <div className="w-5 h-5 rounded overflow-hidden">
                        <ImageWithFallback
                          src={getSpellIconUrl(match.spell1.name)}
                          alt={match.spell1.name}
                          fallback={match.spell1.name.slice(0, 1)}
                          className="w-full h-full object-cover"
                        />
                      </div>
                    ) : (
                      <div className="w-5 h-5 rounded bg-black/40 border border-white/10" />
                    )}
                    {match.spell2 ? (
                      <div className="w-5 h-5 rounded overflow-hidden">
                        <ImageWithFallback
                          src={getSpellIconUrl(match.spell2.name)}
                          alt={match.spell2.name}
                          fallback={match.spell2.name.slice(0, 1)}
                          className="w-full h-full object-cover"
                        />
                      </div>
                    ) : (
                      <div className="w-5 h-5 rounded bg-black/40 border border-white/10" />
                    )}
                  </div>
                  {/* Runes */}
                  <div className="flex gap-0.5">
                    {match.runes ? (
                      <>
                        <div
                          className="w-5 h-5 rounded overflow-hidden"
                          style={{
                            backgroundColor: getKeystoneColor(match.runes.keystoneName) + "33",
                          }}
                        >
                          <ImageWithFallback
                            src={getKeystoneIconUrl(match.runes.keystoneName) || ""}
                            alt={match.runes.keystoneName}
                            fallback={match.runes.keystoneName.slice(0, 2)}
                            className="w-full h-full object-cover"
                          />
                        </div>
                        <div className="w-5 h-5 rounded overflow-hidden bg-black/40">
                          <ImageWithFallback
                            src={getRuneTreeIconUrl(match.runes.secondaryTreeName) || ""}
                            alt={match.runes.secondaryTreeName}
                            fallback={match.runes.secondaryTreeName.slice(0, 1)}
                            className="w-full h-full object-cover p-0.5"
                          />
                        </div>
                      </>
                    ) : (
                      <>
                        <div className="w-5 h-5 rounded bg-black/40 border border-white/10" />
                        <div className="w-5 h-5 rounded bg-black/40 border border-white/10" />
                      </>
                    )}
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
                      kdaLabel !== "Perfect" &&
                        kdaLabel !== "Legendary" &&
                        "text-blue-500"
                    )}
                  >
                    {kdaLabel}
                  </div>
                )}
              </div>
            </div>

            {/* Items row */}
            <div className="flex items-center mt-1">
              <div className="flex gap-0.5">
                {getItemsForSlots(match.items).map((item, i) => (
                  <div
                    key={i}
                    className={cn(
                      "w-5 h-5 rounded overflow-hidden",
                      item ? "bg-muted" : "bg-black/40 border border-white/10"
                    )}
                    title={item ? item.name : "Empty"}
                  >
                    {item && (
                      <ImageWithFallback
                        src={getItemIconUrlById(item.itemId)}
                        alt={item.name}
                        fallback={item.name.slice(0, 2)}
                        className="w-full h-full object-cover"
                      />
                    )}
                  </div>
                ))}
                {/* Trinket slot */}
                <div
                  className={cn(
                    "w-5 h-5 rounded-full overflow-hidden",
                    match.trinket ? "bg-muted" : "bg-black/40 border border-white/10"
                  )}
                  title={match.trinket ? match.trinket.name : "No trinket"}
                >
                  {match.trinket && (
                    <ImageWithFallback
                      src={getItemIconUrlById(match.trinket.itemId)}
                      alt={match.trinket.name}
                      fallback="W"
                      className="w-full h-full object-cover"
                    />
                  )}
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* Stats column */}
        <div className="shrink-0 py-2 px-2 text-[10px] space-y-0.5 flex flex-col justify-center">
          <div className="text-muted-foreground">
            CS{" "}
            <span className="text-foreground font-medium">{match.cs}</span>
            <span className="text-muted-foreground/70">
              {" "}
              ({(match.cs / Math.max(1, match.gameTimeSecs / 60)).toFixed(1)})
            </span>
          </div>
          <div className="text-muted-foreground">
            Gold{" "}
            <span className="text-yellow-500 font-medium">
              {Math.floor(match.currentGold).toLocaleString()}
            </span>
          </div>
          {match.isDead && (
            <div className="text-red-400 font-medium">Dead</div>
          )}
        </div>

        {/* Team Compositions */}
        <div className="flex gap-2 p-2 pl-0 shrink-0 items-center">
          {/* Blue Team */}
          <div className="flex flex-col gap-0.5">
            {blueTeam.slice(0, 5).map((p, i) => (
              <div
                key={i}
                className={cn(
                  "flex items-center gap-1",
                  p.summonerName === match.summonerName &&
                    "bg-primary/10 rounded-sm px-1 -mx-1"
                )}
              >
                <div
                  className={cn(
                    "w-4 h-4 rounded overflow-hidden shrink-0 relative",
                    p.summonerName === match.summonerName && "ring-1 ring-primary"
                  )}
                >
                  <ImageWithFallback
                    src={getChampionIconUrl(p.champion)}
                    alt={p.champion}
                    fallback={p.champion.slice(0, 2)}
                    className={cn(
                      "w-full h-full object-cover",
                      p.isDead && "grayscale opacity-50"
                    )}
                  />
                </div>
                <span
                  className={cn(
                    "text-[10px] leading-none",
                    p.summonerName === match.summonerName
                      ? "text-foreground font-medium"
                      : "text-muted-foreground",
                    p.isDead && "line-through opacity-50"
                  )}
                >
                  {p.summonerName}
                </span>
                <span className="text-[9px] text-muted-foreground/70">
                  {p.kills}/{p.deaths}/{p.assists}
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
                  p.summonerName === match.summonerName &&
                    "bg-primary/10 rounded-sm px-1 -mx-1"
                )}
              >
                <div
                  className={cn(
                    "w-4 h-4 rounded overflow-hidden shrink-0 relative",
                    p.summonerName === match.summonerName && "ring-1 ring-primary"
                  )}
                >
                  <ImageWithFallback
                    src={getChampionIconUrl(p.champion)}
                    alt={p.champion}
                    fallback={p.champion.slice(0, 2)}
                    className={cn(
                      "w-full h-full object-cover",
                      p.isDead && "grayscale opacity-50"
                    )}
                  />
                </div>
                <span
                  className={cn(
                    "text-[10px] leading-none",
                    p.summonerName === match.summonerName
                      ? "text-foreground font-medium"
                      : "text-muted-foreground",
                    p.isDead && "line-through opacity-50"
                  )}
                >
                  {p.summonerName}
                </span>
                <span className="text-[9px] text-muted-foreground/70">
                  {p.kills}/{p.deaths}/{p.assists}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>
    </motion.div>
  );
}
