import { LiveMatchCard as LeagueLiveMatchCard } from "./LiveMatchCard";
import { TFTLiveMatchCard, isTFTLiveMatch } from "./TFTLiveMatchCard";
import type { LeagueLiveMatch } from "./types";

interface SmartLiveMatchCardProps {
  match: LeagueLiveMatch | unknown;
}

/**
 * Smart LiveMatchCard that renders the appropriate card based on game mode
 */
export function SmartLiveMatchCard({ match }: SmartLiveMatchCardProps) {
  // Check if this is TFT live match data
  if (isTFTLiveMatch(match)) {
    return <TFTLiveMatchCard match={match} />;
  }

  // Default to League live match card
  return <LeagueLiveMatchCard match={match as LeagueLiveMatch} />;
}
