import { MatchCard as LeagueMatchCard } from "./MatchCard";
import { TFTMatchCard } from "./TFTMatchCard";
import { isTFTMatch } from "./types";
import type { PackMatch, LeagueMatch } from "./types";

interface SmartMatchCardProps {
  match: PackMatch;
  clipCount?: number;
  isSelected?: boolean;
  onClick?: () => void;
}

/**
 * Smart MatchCard that renders the appropriate card based on game mode
 */
export function SmartMatchCard(props: SmartMatchCardProps) {
  // Check if this is a TFT match
  if (isTFTMatch(props.match)) {
    return <TFTMatchCard {...props} match={props.match} />;
  }
  // Default to League match card
  return <LeagueMatchCard {...props} match={props.match as LeagueMatch} />;
}
