import {
  createContext,
  useState,
  useEffect,
  useCallback,
  use,
  useMemo,
} from "react";
import { adminClient } from "../lib/client";

type State = {
  getState: (ip: string) => { connected: boolean; loggedIn: boolean };
  connectedCount: number;
};

const StationsContext = createContext<State | undefined>(undefined);

export function StationsProvider({ children }: { children: React.ReactNode }) {
  const [state, setState] = useState<Record<string, boolean>>({});

  useEffect(() => {
    const controller = new AbortController();

    const stream = adminClient.subscribe(controller.signal);

    async function processStream() {
      try {
        for await (const update of stream) {
          if (controller.signal.aborted) break;

          if (update.message.case === "statusUpdate") {
            const newUpdates = Object.fromEntries(
              update.message.value.status.map((s) => [s.ip, s.loggedIn]),
            );
            setState(newUpdates);
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
      return { connected: true, loggedIn: logged_in };
    },
    [state],
  );

  const value: State = useMemo(
    () => ({
      getState: getter,
      connectedCount: Object.keys(state).length,
    }),
    [getter, state],
  );

  return <StationsContext value={value}>{children}</StationsContext>;
}

export function useStationState() {
  const context = use(StationsContext);
  if (!context) {
    throw new Error("useStationState must be used within a StationsProvider");
  }
  return context;
}
