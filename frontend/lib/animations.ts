/**
 * Animation constants for framer-motion
 */

import type { Variants } from 'framer-motion';

export const springGentle = {
  type: 'spring',
  stiffness: 300,
  damping: 30,
} as const;

export const springBouncy = {
  type: 'spring',
  stiffness: 400,
  damping: 15,
} as const;

/**
 * Card hover/tap animation variants
 */
export const cardHover: Variants = {
  initial: {},
  hover: {
    scale: 1.02,
    y: -2,
    transition: springGentle,
  },
  tap: {
    scale: 0.98,
    transition: { duration: 0.1 },
  },
};

// Keep these for backwards compatibility if used elsewhere
export const cardTap = {
  scale: 0.98,
  transition: { duration: 0.1 },
} as const;

export const fadeIn = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
} as const;

export const slideUp = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
} as const;
