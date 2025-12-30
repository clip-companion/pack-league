/**
 * Format a timestamp as a relative time (e.g., "2 hours ago")
 */
export function formatTimeAgo(date: Date | string): string {
  const now = new Date();
  const then = typeof date === 'string' ? new Date(date) : date;
  const diffMs = now.getTime() - then.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return 'just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  return then.toLocaleDateString();
}

/**
 * Format game duration in full format (e.g., "32:15")
 */
export function formatGameDurationFull(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

/**
 * Format duration in short format (e.g., "32m")
 */
export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  return `${mins}m`;
}

/**
 * Format KDA ratio
 */
export function formatKDARatio(kills: number, deaths: number, assists: number): string {
  if (deaths === 0) {
    return 'Perfect';
  }
  const kda = ((kills + assists) / deaths).toFixed(2);
  return kda;
}

/**
 * Get KDA label (e.g., "Legendary", "Good", etc.)
 */
export function getKDALabel(kills: number, deaths: number, assists: number): string {
  const kda = deaths === 0 ? kills + assists : (kills + assists) / deaths;

  if (deaths === 0 && (kills > 0 || assists > 0)) return 'Perfect';
  if (kda >= 5) return 'Legendary';
  if (kda >= 4) return 'Excellent';
  if (kda >= 3) return 'Great';
  if (kda >= 2) return 'Good';
  if (kda >= 1) return 'Average';
  return 'Poor';
}

/**
 * Format LP change with sign
 */
export function formatLPChange(lp: number | null | undefined): string | null {
  if (lp === null || lp === undefined) return null;
  return lp >= 0 ? `+${lp}` : `${lp}`;
}
