import { cn } from '../lib/cn';

interface GameIconProps {
  gameId: number;
  size?: 'sm' | 'md' | 'lg';
  className?: string;
}

/**
 * Game icon component - displays League of Legends icon
 */
export function GameIcon({ gameId, size = 'md', className }: GameIconProps) {
  const sizeClasses = {
    sm: 'w-4 h-4',
    md: 'w-6 h-6',
    lg: 'w-8 h-8',
  };

  // For League of Legends (gameId 1), use the LoL icon
  // This is a placeholder - in production this would come from DDragon or assets
  return (
    <div
      className={cn(
        'rounded-sm overflow-hidden bg-slate-700 flex items-center justify-center text-xs font-bold text-white',
        sizeClasses[size],
        className
      )}
      title="League of Legends"
    >
      L
    </div>
  );
}
