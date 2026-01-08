import { MatchCard as LeagueMatchCard } from "./MatchCard";
import { TFTMatchCard } from "./TFTMatchCard";
import { isTFTMatch } from "./types";
import type { AnyPackMatch, AnyLeagueMatch } from "./types";

interface SmartMatchCardProps {
  match: AnyPackMatch;
  clipCount?: number;
  isSelected?: boolean;
  onClick?: () => void;
}

/**
 * Smart MatchCard that renders the appropriate card based on game mode
 * Supports both legacy and new (core/details) match formats
 */
export function SmartMatchCard(props: SmartMatchCardProps) {
  // Check if this is a TFT match
  if (isTFTMatch(props.match)) {
    return <TFTMatchCard {...props} match={props.match} />;
  }
  // Default to League match card
  return <LeagueMatchCard {...props} match={props.match as AnyLeagueMatch} />;
}
