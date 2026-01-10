import { motion } from "framer-motion";
import { cn } from "./lib/cn";
import { formatTimeAgo, formatGameDurationFull, formatLPChange } from "./lib/formatters";
import { cardHover } from "./lib/animations";
import type { AnyTFTMatch, TFTPlacement, TFTTraitTier, TFTTrait, TFTUnit, TFTAugment } from "./types";
import { isTopFour, getMatchCore } from "./types";

interface TFTMatchCardProps {
  match: AnyTFTMatch;
  clipCount?: number;
  isSelected?: boolean;
  onClick?: () => void;
}

/**
 * Get placement display with ordinal suffix (1st, 2nd, 3rd, etc.)
 */
function getPlacementDisplay(placement: TFTPlacement): string {
  const suffixes: Record<number, string> = { 1: "st", 2: "nd", 3: "rd" };
  const suffix = suffixes[placement] || "th";
  return `${placement}${suffix}`;
}

/**
 * Get color class based on placement
 */
function getPlacementColorClass(placement: TFTPlacement): string {
  switch (placement) {
    case 1:
      return "text-yellow-400"; // Gold for 1st
    case 2:
      return "text-slate-300"; // Silver for 2nd
    case 3:
      return "text-amber-600"; // Bronze for 3rd
    case 4:
      return "text-emerald-400"; // Green for 4th (still top 4)
    default:
      return "text-loss"; // Red for bottom 4
  }
}

/**
 * Get background accent color based on placement
 */
function getPlacementBgClass(placement: TFTPlacement): string {
  if (placement === 1) return "bg-yellow-500";
  if (placement <= 4) return "bg-win";
  return "bg-loss";
}

/**
 * Get trait tier background color
 */
function getTraitTierBg(tier: TFTTraitTier): string {
  switch (tier) {
    case "chromatic":
      return "bg-gradient-to-r from-pink-500 via-purple-500 to-blue-500";
    case "gold":
      return "bg-yellow-500";
    case "silver":
      return "bg-slate-400";
    case "bronze":
      return "bg-amber-700";
    default:
      return "bg-zinc-700";
  }
}

/**
 * Get trait tier text color
 */
function getTraitTierText(tier: TFTTraitTier): string {
  switch (tier) {
    case "chromatic":
      return "text-white";
    case "gold":
      return "text-yellow-900";
    case "silver":
      return "text-slate-900";
    case "bronze":
      return "text-amber-100";
    default:
      return "text-zinc-400";
  }
}

/**
 * Get augment tier color
 */
function getAugmentTierColor(tier: TFTAugment["tier"]): string {
  switch (tier) {
    case "prismatic":
      return "text-pink-400 bg-pink-500/20";
    case "gold":
      return "text-yellow-400 bg-yellow-500/20";
    case "silver":
      return "text-slate-300 bg-slate-500/20";
    default:
      return "text-zinc-400 bg-zinc-500/20";
  }
}

/**
 * Star display for unit tier
 */
function UnitStars({ tier }: { tier: 1 | 2 | 3 }) {
  const starColor = tier === 3 ? "text-yellow-400" : tier === 2 ? "text-slate-300" : "text-amber-700";
  return (
    <span className={cn("text-[8px] leading-none", starColor)}>
      {"★".repeat(tier)}
    </span>
  );
}

/**
 * Single trait badge
 */
function TraitBadge({ trait }: { trait: TFTTrait }) {
  return (
    <div
      className={cn(
        "flex items-center gap-1 px-1.5 py-0.5 rounded text-[10px] font-medium",
        getTraitTierBg(trait.style),
        getTraitTierText(trait.style)
      )}
      title={`${trait.name} (${trait.tierCurrent}/${trait.tierTotal})`}
    >
      <span className="truncate max-w-[60px]">{trait.name}</span>
      <span className="opacity-75">{trait.numUnits}</span>
    </div>
  );
}

/**
 * Single unit on the board
 */
function UnitIcon({ unit }: { unit: TFTUnit }) {
  const hasItems = unit.itemNames && unit.itemNames.length > 0;
  return (
    <div
      className="relative flex flex-col items-center"
      title={`${unit.character}${hasItems ? ` - ${unit.itemNames.join(", ")}` : ""}`}
    >
      {/* Unit icon placeholder - in a real app this would be an image */}
      <div className={cn(
        "w-8 h-8 rounded bg-zinc-800 border flex items-center justify-center text-[10px] font-medium",
        unit.tier === 3 && "border-yellow-500 ring-1 ring-yellow-500/50",
        unit.tier === 2 && "border-slate-400",
        unit.tier === 1 && "border-zinc-600"
      )}>
        {unit.character.slice(0, 2)}
      </div>
      {/* Star level */}
      <UnitStars tier={unit.tier} />
      {/* Item indicator */}
      {hasItems && (
        <div className="absolute -top-0.5 -right-0.5 w-3 h-3 bg-blue-500 rounded-full text-[8px] flex items-center justify-center text-white">
          {unit.itemNames.length}
        </div>
      )}
    </div>
  );
}

/**
 * Augment display
 */
function AugmentBadge({ augment }: { augment: TFTAugment }) {
  return (
    <div
      className={cn(
        "px-2 py-0.5 rounded text-[10px] font-medium truncate",
        getAugmentTierColor(augment.tier)
      )}
      title={`${augment.name} (${augment.tier})`}
    >
      {augment.name}
    </div>
  );
}

export function TFTMatchCard({ match, isSelected, onClick }: TFTMatchCardProps) {
  // Get core data (works with both legacy and new API formats)
  const core = getMatchCore(match);

  // Destructure from match.details (game-specific fields)
  const {
    placement,
    lpChange: lpChangeValue,
    gameMode,
    rank,
    badges,
    level,
    playersEliminated,
    totalDamageToPlayers,
    traits,
    units,
    augments,
  } = match.details;

  const isTop4 = isTopFour(placement);
  const lpChange = formatLPChange(lpChangeValue);
  const placementDisplay = getPlacementDisplay(placement);
  const queueName = typeof gameMode === "object"
    ? gameMode.queueName || "TFT"
    : "TFT";

  // Sort traits by tier (chromatic > gold > silver > bronze)
  const sortedTraits = [...(traits || [])].sort((a, b) => {
    const tierOrder: Record<TFTTraitTier, number> = {
      chromatic: 4,
      gold: 3,
      silver: 2,
      bronze: 1,
      inactive: 0,
    };
    return tierOrder[b.style] - tierOrder[a.style];
  });

  // Sort units by tier (3-star first)
  const sortedUnits = [...(units || [])].sort((a, b) => b.tier - a.tier);

  return (
    <motion.button
      onClick={onClick}
      variants={cardHover}
      initial="initial"
      whileHover="hover"
      whileTap="tap"
      className={cn(
        "w-full text-left rounded-lg border overflow-hidden",
        "hover:border-primary/30 transition-colors",
        isSelected && "border-primary/50 shadow-md",
        !isSelected && "bg-card"
      )}
    >
      <div className="flex">
        {/* Placement indicator bar - left side */}
        <div className={cn("w-1 shrink-0", getPlacementBgClass(placement))} />

        {/* Main content */}
        <div className="flex-1 min-w-0">
          {/* Top row: placement, game info, rank */}
          <div className="flex items-center gap-3 p-2 pb-1">
            {/* Placement - prominent display */}
            <div className="flex flex-col items-center shrink-0">
              <div className={cn(
                "text-3xl font-bold tracking-tight leading-none",
                getPlacementColorClass(placement)
              )}>
                {placementDisplay}
              </div>
              <div className={cn(
                "text-[10px] font-medium mt-0.5",
                isTop4 ? "text-win" : "text-loss"
              )}>
                {isTop4 ? "Top 4" : "Bot 4"}
              </div>
            </div>

            {/* Game info */}
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="text-xs font-medium text-muted-foreground truncate">
                  {queueName}
                </span>
                <span className="text-xs text-muted-foreground">•</span>
                <span className="text-xs text-muted-foreground">
                  {formatTimeAgo(core.playedAt)}
                </span>
              </div>
              <div className="flex items-center gap-2 mt-0.5">
                <span className="text-xs text-muted-foreground">
                  {formatGameDurationFull(core.durationSecs || 0)}
                </span>
                {level && (
                  <>
                    <span className="text-xs text-muted-foreground">•</span>
                    <span className="text-xs text-muted-foreground">
                      Lvl <span className="text-foreground font-medium">{level}</span>
                    </span>
                  </>
                )}
              </div>
            </div>

            {/* LP change and rank */}
            <div className="flex flex-col items-end shrink-0">
              {lpChange && (
                <div className={cn(
                  "text-sm font-semibold",
                  lpChangeValue && lpChangeValue > 0 ? "text-win" : "text-loss"
                )}>
                  {lpChange}
                </div>
              )}
              {rank && (
                <div className="text-xs text-primary font-medium">
                  {rank}
                </div>
              )}
            </div>
          </div>

          {/* Traits row */}
          {sortedTraits.length > 0 && (
            <div className="px-2 py-1 flex flex-wrap gap-1 border-t border-border/30">
              {sortedTraits.slice(0, 6).map((trait, i) => (
                <TraitBadge key={`${trait.name}-${i}`} trait={trait} />
              ))}
              {sortedTraits.length > 6 && (
                <span className="text-[10px] text-muted-foreground self-center">
                  +{sortedTraits.length - 6}
                </span>
              )}
            </div>
          )}

          {/* Bottom row: units and augments */}
          <div className="flex items-center gap-2 px-2 py-1.5 border-t border-border/30">
            {/* Units (board composition) */}
            {sortedUnits.length > 0 && (
              <div className="flex gap-0.5 overflow-hidden">
                {sortedUnits.slice(0, 8).map((unit, i) => (
                  <UnitIcon key={`${unit.character}-${i}`} unit={unit} />
                ))}
                {sortedUnits.length > 8 && (
                  <div className="w-8 h-8 rounded bg-zinc-800 border border-zinc-600 flex items-center justify-center text-[10px] text-muted-foreground">
                    +{sortedUnits.length - 8}
                  </div>
                )}
              </div>
            )}

            {/* Spacer */}
            <div className="flex-1" />

            {/* Augments */}
            {augments && augments.length > 0 && (
              <div className="flex flex-col gap-0.5 shrink-0">
                {augments.slice(0, 3).map((aug, i) => (
                  <AugmentBadge key={`${aug.name}-${i}`} augment={aug} />
                ))}
              </div>
            )}
          </div>

          {/* Stats row (if we have damage/eliminations data) */}
          {(playersEliminated !== undefined || totalDamageToPlayers !== undefined || (badges && badges.length > 0)) && (
            <div className="flex items-center gap-3 px-2 py-1 border-t border-border/30 text-[10px]">
              {playersEliminated !== undefined && playersEliminated > 0 && (
                <span className="text-muted-foreground">
                  Eliminated <span className="text-foreground font-medium">{playersEliminated}</span>
                </span>
              )}
              {totalDamageToPlayers !== undefined && (
                <span className="text-muted-foreground">
                  Damage <span className="text-foreground font-medium">{totalDamageToPlayers}</span>
                </span>
              )}
              {/* Badges */}
              {badges && badges.length > 0 && (
                <div className="flex gap-1 ml-auto">
                  {badges.slice(0, 2).map((badge) => (
                    <span
                      key={badge}
                      className={cn(
                        "px-1.5 py-0.5 rounded text-[9px] font-medium",
                        badge === "First Place" && "bg-yellow-500/20 text-yellow-400",
                        badge === "Top 4" && "bg-emerald-500/20 text-emerald-400",
                        !["First Place", "Top 4"].includes(badge) && "bg-slate-500/20 text-slate-400"
                      )}
                    >
                      {badge}
                    </span>
                  ))}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </motion.button>
  );
}
