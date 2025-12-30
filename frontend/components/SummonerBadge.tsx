import { cn } from "@/lib/cn";
import { User } from "lucide-react";

interface SummonerBadgeProps {
  name: string | null;
  className?: string;
}

export function SummonerBadge({ name, className }: SummonerBadgeProps) {
  if (!name) return null;

  return (
    <div
      className={cn(
        "flex items-center gap-2 rounded-full bg-muted px-3 py-1.5",
        className
      )}
    >
      <User className="h-4 w-4 text-muted-foreground" />
      <span className="text-sm font-medium">{name}</span>
    </div>
  );
}
