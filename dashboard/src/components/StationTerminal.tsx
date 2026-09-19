import { useEffect, useRef, useState } from "react";
import { create } from "@bufbuild/protobuf";
import { adminClient } from "../lib/client";
import { useCommandStore } from "../context/command-store";
import { useStationState } from "../context/station-state";
import { CustomCommandSchema } from "@client/v1/admin/station_pb";

type StationTerminalProps = {
  ip: string;
};

export function StationTerminal({ ip }: StationTerminalProps) {
  const { register, getHistory } = useCommandStore();
  const { getState } = useStationState();
  const [draft, setDraft] = useState("");
  const [recalledIndex, setRecalledIndex] = useState<number | null>(null);
  const scrollRef = useRef<HTMLDivElement>(null);
  const history = getHistory(ip);
  const connected = getState(ip).connected;

  const sentCommands = history.map((entry) => entry.command);
  const input =
    recalledIndex === null
      ? draft
      : sentCommands[sentCommands.length - 1 - recalledIndex];

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [history]);

  const handleSubmit = () => {
    const command = input.trim();
    if (!command || !connected) return;

    const id = crypto.randomUUID();
    register(id, [ip], command);
    adminClient.sendEvent([ip], {
      case: "custom",
      value: create(CustomCommandSchema, { id, command }),
    });
    setDraft("");
    setRecalledIndex(null);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") {
      handleSubmit();
      return;
    }

    if (e.key === "ArrowUp") {
      e.preventDefault();
      if (sentCommands.length === 0) return;
      setRecalledIndex((prev) =>
        prev === null ? 0 : Math.min(prev + 1, sentCommands.length - 1),
      );
      return;
    }

    if (e.key === "ArrowDown") {
      e.preventDefault();
      setRecalledIndex((prev) =>
        prev === null || prev === 0 ? null : prev - 1,
      );
    }
  };

  return (
    <div className="bg-surface-900 border-t border-surface-700 font-mono text-sm">
      <div ref={scrollRef} className="max-h-64 overflow-y-auto p-4 space-y-2">
        {history.length === 0 && (
          <p className="text-gray-600">No commands yet</p>
        )}
        {history.map((entry) => (
          <div key={entry.id}>
            <div className="flex gap-2">
              <span className="text-emerald-400 shrink-0">&gt;</span>
              <span className="text-gray-200">{entry.command}</span>
            </div>
            {entry.output !== null ? (
              <pre className="text-gray-400 pl-5 whitespace-pre-wrap break-all">
                {entry.output}
              </pre>
            ) : (
              <span className="text-gray-600 pl-5 animate-pulse">
                waiting for output...
              </span>
            )}
          </div>
        ))}
      </div>
      <div className="flex items-center gap-2 border-t border-surface-700 px-4 py-2">
        <span
          className={`shrink-0 ${connected ? "text-emerald-400" : "text-gray-600"}`}
        >
          &gt;
        </span>
        <input
          type="text"
          value={input}
          onChange={(e) => {
            setDraft(e.target.value);
            setRecalledIndex(null);
          }}
          onKeyDown={handleKeyDown}
          disabled={!connected}
          placeholder={
            connected
              ? "Type a command..."
              : "Station is offline — commands can't be sent"
          }
          className="flex-1 bg-transparent text-gray-200 placeholder-gray-600 focus:outline-none disabled:cursor-not-allowed"
          autoFocus={connected}
        />
      </div>
    </div>
  );
}
