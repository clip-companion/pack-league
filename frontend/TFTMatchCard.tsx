import { motion } from "framer-motion";
import { cn } from "./lib/cn";
import { formatTimeAgo, formatGameDurationFull, formatLPChange } from "./lib/formatters";
import { cardHover } from "./lib/animations";
import type { TFTMatch, TFTPlacement } from "./types";
import { isTopFour } from "./types";

interface TFTMatchCardProps {
  match: TFTMatch;
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

export function TFTMatchCard({ match, isSelected, onClick }: TFTMatchCardProps) {
  const isTop4 = isTopFour(match.placement);
  const lpChange = formatLPChange(match.lpChange);
  const placementDisplay = getPlacementDisplay(match.placement);
  const queueName = typeof match.gameMode === "object"
    ? match.gameMode.queueName || "TFT"
    : "TFT";

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
        <div className={cn("w-1 shrink-0", getPlacementBgClass(match.placement))} />

        {/* Game info column */}
        <div className="w-28 shrink-0 p-3 flex flex-col justify-center overflow-hidden">
          <div className="text-xs font-medium text-muted-foreground truncate">
            {queueName}
          </div>
          <div className="text-xs text-muted-foreground">
            {formatTimeAgo(match.playedAt)}
          </div>
          <div className="mt-1">
            <span className={cn("text-xs", isTop4 ? "text-win" : "text-loss")}>
              {isTop4 ? "Top 4" : "Bottom 4"}
            </span>
          </div>
          <div className="text-xs text-muted-foreground">
            {formatGameDurationFull(match.durationSecs)}
          </div>
        </div>

        {/* Placement display - main focus */}
        <div className="flex-1 py-3 px-4 flex items-center justify-center border-l border-border/30">
          <div className="text-center">
            <div className={cn(
              "text-4xl font-bold tracking-tight",
              getPlacementColorClass(match.placement)
            )}>
              {placementDisplay}
            </div>
            <div className="text-sm text-muted-foreground mt-1">
              Place
            </div>
            {lpChange && (
              <div className={cn(
                "text-sm font-medium mt-1",
                match.lpChange && match.lpChange > 0 ? "text-win" : "text-loss"
              )}>
                {lpChange}
              </div>
            )}
          </div>
        </div>

        {/* Rank info - right side */}
        <div className="shrink-0 py-3 px-4 flex flex-col justify-center items-end border-l border-border/30">
          {match.rank && (
            <div className="text-sm font-medium text-primary">
              {match.rank}
            </div>
          )}
          {/* Badges */}
          {match.badges && match.badges.length > 0 && (
            <div className="flex flex-col gap-1 mt-2">
              {match.badges.slice(0, 2).map((badge) => (
                <span
                  key={badge}
                  className={cn(
                    "px-2 py-0.5 rounded text-xs font-medium whitespace-nowrap",
                    badge === "1st Place" && "bg-yellow-500/20 text-yellow-400",
                    badge === "Top 4" && "bg-emerald-500/20 text-emerald-400",
                    !["1st Place", "Top 4"].includes(badge) && "bg-slate-500/20 text-slate-400"
                  )}
                >
                  {badge}
                </span>
              ))}
            </div>
          )}
        </div>
      </div>
    </motion.button>
  );
}
