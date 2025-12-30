import { useEffect } from "react";
import { useAppStore } from "@/stores/appStore";

export function useLeagueConnection() {
  const { leagueConnection, fetchLeagueConnection } = useAppStore();

  useEffect(() => {
    fetchLeagueConnection();

    const interval = setInterval(fetchLeagueConnection, 5000);
    return () => clearInterval(interval);
  }, [fetchLeagueConnection]);

  return leagueConnection;
}
