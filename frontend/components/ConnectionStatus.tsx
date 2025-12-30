/**
 * ConnectionStatus - Full connection status display for settings page
 *
 * Re-exports the ConnectionIndicator component with full variant.
 */

import { ConnectionIndicator } from "@/components/ConnectionIndicator";

export function ConnectionStatus() {
  return <ConnectionIndicator variant="full" />;
}
