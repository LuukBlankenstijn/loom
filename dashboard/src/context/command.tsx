import { createContext, useCallback, use, useRef, useState } from "react";

export type CommandEntry = {
  id: string;
  command: string;
  timestamp: number;
  output: string | null;
};

type CommandStore = {
  register: (id: string, ips: string[], command: string) => void;
  setOutput: (id: string, output: string) => void;
  getHistory: (ip: string) => CommandEntry[];
};

const CommandContext = createContext<CommandStore | undefined>(undefined);

export function CommandProvider({ children }: { children: React.ReactNode }) {
  const [history, setHistory] = useState<Record<string, CommandEntry[]>>({});
  const idToIps = useRef(new Map<string, string[]>());

  const register = useCallback(
    (id: string, ips: string[], command: string) => {
      idToIps.current.set(id, ips);
      const entry: CommandEntry = {
        id,
        command,
        timestamp: Date.now(),
        output: null,
      };
      setHistory((prev) => {
        const next = { ...prev };
        for (const ip of ips) {
          next[ip] = [...(prev[ip] ?? []), entry];
        }
        return next;
      });
    },
    [],
  );

  const setOutput = useCallback((id: string, output: string) => {
    const ips = idToIps.current.get(id);
    if (!ips) return;
    setHistory((prev) => {
      const next = { ...prev };
      for (const ip of ips) {
        next[ip] = (prev[ip] ?? []).map((entry) =>
          entry.id === id ? { ...entry, output } : entry,
        );
      }
      return next;
    });
  }, []);

  const getHistory = useCallback(
    (ip: string): CommandEntry[] => history[ip] ?? [],
    [history],
  );

  return (
    <CommandContext value={{ register, setOutput, getHistory }}>
      {children}
    </CommandContext>
  );
}

export function useCommandStore() {
  const context = use(CommandContext);
  if (!context) {
    throw new Error("useCommandStore must be used within a CommandProvider");
  }
  return context;
}
