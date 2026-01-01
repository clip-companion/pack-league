import { motion } from "framer-motion";
import { cn } from "./lib/cn";
import { springGentle } from "./lib/animations";
import type { GameModeContext } from "./types";

interface TFTLiveMatchData {
  gameMode: GameModeContext;
  isTft: boolean;
  queueName: string;
  isRanked: boolean;
}

interface TFTLiveMatchCardProps {
  match: TFTLiveMatchData;
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

export function TFTLiveMatchCard({ match }: TFTLiveMatchCardProps) {
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

        {/* Game info */}
        <div className="flex-1 p-4 flex items-center gap-4">
          {/* TFT Icon/Badge */}
          <div className="w-16 h-16 rounded-lg bg-gradient-to-br from-purple-500/20 to-blue-500/20 flex items-center justify-center border border-purple-500/30">
            <span className="text-2xl font-bold bg-gradient-to-r from-purple-400 to-blue-400 bg-clip-text text-transparent">
              TFT
            </span>
          </div>

          {/* Game details */}
          <div className="flex-1">
            <div className="text-lg font-semibold text-foreground">
              Teamfight Tactics
            </div>
            <div className="text-sm text-muted-foreground">
              {match.queueName || "TFT Game"}
            </div>
            {match.isRanked && (
              <div className="mt-1 inline-flex items-center gap-1 px-2 py-0.5 bg-yellow-500/20 rounded text-xs text-yellow-400 font-medium">
                Ranked
              </div>
            )}
          </div>

          {/* Live indicator */}
          <div className="shrink-0">
            <LiveIndicator />
          </div>
        </div>
      </div>
    </motion.div>
  );
}

/**
 * Type guard to check if live match data is TFT
 */
export function isTFTLiveMatch(data: unknown): data is TFTLiveMatchData {
  if (typeof data !== "object" || data === null) return false;
  return (data as TFTLiveMatchData).isTft === true;
}
