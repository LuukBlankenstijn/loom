import { createContext, use } from "react";

export type StationsState = {
  getState: (ip: string) => { connected: boolean; loggedIn: boolean };
  connectedCount: number;
};

export const StationsContext = createContext<StationsState | undefined>(
  undefined,
);

export function useStationState() {
  const context = use(StationsContext);
  if (!context) {
    throw new Error("useStationState must be used within a StationsProvider");
  }
  return context;
}
