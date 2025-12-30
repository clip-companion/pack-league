/**
 * DDragon download status indicator for sidebar
 *
 * Shows a download icon that indicates when League data is being fetched.
 * Hover to see progress details and a preview of loading status.
 */

import { motion, AnimatePresence } from 'framer-motion';
import { Download, Check, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useDDragon } from '../hooks/useDDragon';
import { cn } from '@/lib/cn';

export function DDragonStatus() {
  const { loading, ready, progress, version } = useDDragon();

  // Don't show anything if ready and not loading
  if (ready && !loading) {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              className="h-10 w-10 text-muted-foreground hover:text-foreground"
            >
              <Check className="h-4 w-4 text-green-500" />
            </Button>
          </TooltipTrigger>
          <TooltipContent side="right" className="w-48">
            <div className="space-y-1">
              <p className="font-medium text-green-500">League Data Ready</p>
              <p className="text-xs text-muted-foreground">
                Version {version || 'unknown'}
              </p>
              <p className="text-xs text-muted-foreground">
                All icons cached locally
              </p>
            </div>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  const progressPercent = (progress.loaded / progress.total) * 100;

  return (
    <TooltipProvider>
      <Tooltip>
        <TooltipTrigger asChild>
          <Button
            variant="ghost"
            size="icon"
            className="h-10 w-10 relative"
          >
            <AnimatePresence mode="wait">
              {loading ? (
                <motion.div
                  key="loading"
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                >
                  <Loader2 className="h-5 w-5 animate-spin text-primary" />
                </motion.div>
              ) : (
                <motion.div
                  key="download"
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                >
                  <Download className="h-5 w-5 text-muted-foreground" />
                </motion.div>
              )}
            </AnimatePresence>

            {/* Progress ring */}
            {loading && (
              <svg
                className="absolute inset-0 h-10 w-10 -rotate-90"
                viewBox="0 0 40 40"
              >
                <circle
                  cx="20"
                  cy="20"
                  r="16"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  className="text-muted/30"
                />
                <motion.circle
                  cx="20"
                  cy="20"
                  r="16"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  className="text-primary"
                  strokeDasharray={100}
                  initial={{ strokeDashoffset: 100 }}
                  animate={{ strokeDashoffset: 100 - progressPercent }}
                  transition={{ duration: 0.3 }}
                />
              </svg>
            )}
          </Button>
        </TooltipTrigger>
        <TooltipContent side="right" className="w-56">
          <div className="space-y-2">
            <p className="font-medium">
              {loading ? 'Downloading League Data' : 'League Data'}
            </p>

            {/* Progress bar */}
            <div className="space-y-1">
              <div className="flex justify-between text-xs text-muted-foreground">
                <span>{progress.currentItem || 'Initializing...'}</span>
                <span>{progress.loaded}/{progress.total}</span>
              </div>
              <div className="h-1.5 w-full rounded-full bg-muted">
                <motion.div
                  className={cn(
                    "h-full rounded-full",
                    ready ? "bg-green-500" : "bg-primary"
                  )}
                  initial={{ width: 0 }}
                  animate={{ width: `${progressPercent}%` }}
                  transition={{ duration: 0.3 }}
                />
              </div>
            </div>

            {version && (
              <p className="text-xs text-muted-foreground">
                Version: {version}
              </p>
            )}
          </div>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
