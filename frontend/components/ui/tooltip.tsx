import * as React from 'react';
import { cn } from '../../lib/cn';

const TooltipProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  return <>{children}</>;
};

interface TooltipProps {
  children: React.ReactNode;
}

const Tooltip: React.FC<TooltipProps> = ({ children }) => {
  const [isOpen, setIsOpen] = React.useState(false);

  return (
    <div
      className="relative inline-block"
      onMouseEnter={() => setIsOpen(true)}
      onMouseLeave={() => setIsOpen(false)}
    >
      {React.Children.map(children, (child) => {
        if (React.isValidElement(child)) {
          if (child.type === TooltipTrigger) {
            return child;
          }
          if (child.type === TooltipContent) {
            return isOpen ? child : null;
          }
        }
        return child;
      })}
    </div>
  );
};

interface TooltipTriggerProps {
  children: React.ReactNode;
  asChild?: boolean;
}

const TooltipTrigger: React.FC<TooltipTriggerProps> = ({ children, asChild }) => {
  if (asChild && React.isValidElement(children)) {
    return children;
  }
  return <span>{children}</span>;
};

interface TooltipContentProps {
  children: React.ReactNode;
  side?: 'top' | 'right' | 'bottom' | 'left';
  className?: string;
}

const TooltipContent: React.FC<TooltipContentProps> = ({
  children,
  side = 'top',
  className,
}) => {
  const sideClasses = {
    top: 'bottom-full left-1/2 -translate-x-1/2 mb-2',
    right: 'left-full top-1/2 -translate-y-1/2 ml-2',
    bottom: 'top-full left-1/2 -translate-x-1/2 mt-2',
    left: 'right-full top-1/2 -translate-y-1/2 mr-2',
  };

  return (
    <div
      className={cn(
        'absolute z-50 overflow-hidden rounded-md border bg-popover px-3 py-1.5 text-sm text-popover-foreground shadow-md',
        'animate-in fade-in-0 zoom-in-95',
        sideClasses[side],
        className
      )}
    >
      {children}
    </div>
  );
};

export { Tooltip, TooltipContent, TooltipProvider, TooltipTrigger };
