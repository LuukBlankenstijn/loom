import { useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState } from "react";
import type { Station } from "@client/v1/admin/station_pb";
import { adminClient } from "../lib/client";

export function StationLabel({ station }: { station: Station }) {
  const queryClient = useQueryClient();
  const [draft, setDraft] = useState<string | null>(null);
  const cancelled = useRef(false);

  const mutation = useMutation({
    mutationFn: (label?: string) =>
      adminClient.setStationLabel(station.ip, label),
    onSettled: () =>
      queryClient.invalidateQueries({ queryKey: ["stations"] }),
  });

  const label = mutation.isPending
    ? (mutation.variables ?? station.reportedLabel)
    : station.label;
  const overridden = mutation.isPending
    ? mutation.variables !== undefined
    : station.labelOverridden;

  const commit = (value: string) => {
    setDraft(null);
    const next = value.trim().toLowerCase();
    if (next === (station.label ?? "")) return;
    mutation.mutate(next || undefined);
  };

  if (draft !== null) {
    return (
      <input
        value={draft}
        onChange={(e) => setDraft(e.target.value)}
        onKeyDown={(e) => {
          if (e.key === "Enter") {
            e.currentTarget.blur();
          } else if (e.key === "Escape") {
            cancelled.current = true;
            e.currentTarget.blur();
          }
        }}
        onBlur={() => {
          if (cancelled.current) {
            cancelled.current = false;
            setDraft(null);
          } else {
            commit(draft);
          }
        }}
        placeholder={station.reportedLabel ?? "label"}
        className="w-28 bg-surface-700 border border-primary-500 rounded-md px-2 py-1 text-xs font-mono text-gray-200 placeholder-gray-500 focus:outline-none"
        autoFocus
      />
    );
  }

  return (
    <div className="flex items-center gap-1.5">
      <button
        onClick={() => setDraft(label ?? "")}
        title={
          mutation.isError
            ? `Failed to save label: ${mutation.error.message}`
            : "Edit label"
        }
        className={`rounded-md transition-colors ${
          mutation.isError
            ? "px-2 py-0.5 text-xs font-mono bg-danger-500/10 text-danger-500 border border-danger-500/30"
            : label
              ? "px-2 py-0.5 text-xs font-mono bg-sky-500/10 text-sky-300 border border-sky-500/20 hover:bg-sky-500/20"
              : "px-2 py-0.5 text-sm text-gray-600 hover:text-gray-300 hover:bg-surface-700"
        }`}
      >
        {label ?? "—"}
      </button>
      {overridden && (
        <>
          <span
            title={`reported by agent: ${station.reportedLabel ?? "none"}`}
            className="w-1.5 h-1.5 rounded-full bg-amber-400"
          />
          <button
            onClick={() => mutation.mutate(undefined)}
            disabled={mutation.isPending}
            title={`Reset to agent label (${station.reportedLabel ?? "none"})`}
            className="text-xs text-gray-500 hover:text-gray-200 disabled:opacity-50 transition-colors"
          >
            reset
          </button>
        </>
      )}
    </div>
  );
}
