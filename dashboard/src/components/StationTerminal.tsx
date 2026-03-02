import { useEffect, useRef, useState } from "react";
import { create } from "@bufbuild/protobuf";
import { CustomCommandSchema } from "@client/v1/command/command_pb";
import { adminClient } from "../lib/client";
import { useCommandStore } from "../context/command";

type StationTerminalProps = {
  ip: string;
};

export function StationTerminal({ ip }: StationTerminalProps) {
  const { register, getHistory } = useCommandStore();
  const [input, setInput] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);
  const history = getHistory(ip);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [history]);

  const handleSubmit = () => {
    const command = input.trim();
    if (!command) return;

    const id = crypto.randomUUID();
    register(id, [ip], command);
    adminClient.sendCommand([ip], {
      case: "custom",
      value: create(CustomCommandSchema, { id, command }),
    });
    setInput("");
  };

  return (
    <div className="bg-surface-900 border-t border-surface-700 font-mono text-sm">
      <div
        ref={scrollRef}
        className="max-h-64 overflow-y-auto p-4 space-y-2"
      >
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
        <span className="text-emerald-400 shrink-0">&gt;</span>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={(e) => {
            if (e.key === "Enter") handleSubmit();
          }}
          placeholder="Type a command..."
          className="flex-1 bg-transparent text-gray-200 placeholder-gray-600 focus:outline-none"
          autoFocus
        />
      </div>
    </div>
  );
}
