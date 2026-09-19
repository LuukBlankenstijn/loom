import { useState, useEffect, useCallback, useMemo } from "react";
import { adminClient } from "../lib/client";
import { StationsContext, type StationsState } from "./station-state";

type ConnectionStatus = {
  connected: boolean;
  loggedIn: boolean;
};

export function StationsProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<Record<string, ConnectionStatus>>({});

  useEffect(() => {
    const controller = new AbortController();

    const stream = adminClient.subscribe(controller.signal);

    async function processStream() {
      try {
        for await (const update of stream) {
          if (controller.signal.aborted) break;

          if (update.message.case === "stationsState") {
            setState((old) => {
              const nextState = { ...old };
              if (update.message.case !== "stationsState") {
                // this will never happen, but the lsp server has trouble seeing that
                return nextState;
              }
              update.message.value?.state.forEach((value) => {
                nextState[value.ip] = {
                  connected: value.connected,
                  loggedIn: value.loggedIn,
                };
              });
              return nextState;
            });
          }
        }
      } catch (error) {
        console.error("Station Stream Error:", error);
      }
    }

    processStream();

    return () => {
      controller.abort();
    };
  }, []);

  const getter = useCallback(
    (ip: string): { connected: boolean; loggedIn: boolean } => {
      const logged_in = state[ip];
      if (logged_in === undefined) {
        return { connected: false, loggedIn: false };
      }
      return { connected: logged_in.connected, loggedIn: logged_in.loggedIn };
    },
    [state],
  );

  const value: StationsState = useMemo(
    () => ({
      getState: getter,
      connectedCount: Object.keys(state).length,
    }),
    [getter, state],
  );

  return <StationsContext value={value}>{children}</StationsContext>;
}
