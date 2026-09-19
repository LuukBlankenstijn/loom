import { createContext, use } from "react";

export type CommandEntry = {
  id: string;
  command: string;
  timestamp: number;
  output: string | null;
};

export type CommandStore = {
  register: (id: string, ips: string[], command: string) => void;
  setOutput: (id: string, output: string) => void;
  getHistory: (ip: string) => CommandEntry[];
};

export const CommandContext = createContext<CommandStore | undefined>(
  undefined,
);

export function useCommandStore() {
  const context = use(CommandContext);
  if (!context) {
    throw new Error("useCommandStore must be used within a CommandProvider");
  }
  return context;
}
