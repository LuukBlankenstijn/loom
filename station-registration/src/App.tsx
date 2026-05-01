import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import {
  MapViewer,
  SeatOverlay,
  fromProtoElements,
  type MapElement,
  type Seat,
} from "@loom/map-react";
import { createBackendClient } from "./client";
import { getStationConfig, type StationConfig } from "./tauri";

const MAP_ID = 1;

type SeatMeta = {
  connected: boolean;
  teamName: string | null;
};

export function App() {
  const [config, setConfig] = useState<StationConfig | null>(null);
  const [elements, setElements] = useState<MapElement[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [seatMeta, setSeatMeta] = useState<Map<string, SeatMeta>>(new Map());

  const ipToSeatRef = useRef<Map<string, string>>(new Map());
  const connectedIpsRef = useRef<Set<string>>(new Set());

  const client = useMemo(
    () => (config ? createBackendClient(config) : null),
    [config],
  );

  const myAssignedSeat = useMemo(() => {
    if (!config) return null;
    return elements.find(
      (el) => el.kind === "seat" && el.ip === config.ip,
    ) as Seat | undefined;
  }, [elements, config]);

  useEffect(() => {
    getStationConfig()
      .then(setConfig)
      .catch((e: unknown) =>
        setError(`Failed to load config: ${formatErr(e)}`),
      );
  }, []);

  const loadMap = useCallback(async () => {
    if (!client) return;
    setLoading(true);
    setError(null);
    try {
      const res = await client.getMap(MAP_ID);
      const els = fromProtoElements(res.elements);
      setElements(els);
      const ipMap = new Map<string, string>();
      const meta = new Map<string, SeatMeta>();
      for (const el of els) {
        if (el.kind === "seat") {
          const connected = el.ip ? connectedIpsRef.current.has(el.ip) : false;
          meta.set(el.id, { connected, teamName: null });
          if (el.ip) ipMap.set(el.ip, el.id);
        }
      }
      ipToSeatRef.current = ipMap;
      setSeatMeta(meta);
    } catch (e: unknown) {
      setError(`Failed to load map: ${formatErr(e)}`);
    } finally {
      setLoading(false);
    }
  }, [client]);

  useEffect(() => {
    void loadMap();
  }, [loadMap]);

  useEffect(() => {
    if (!client) return;
    const controller = new AbortController();
    (async () => {
      try {
        for await (const ev of client.subscribe(controller.signal)) {
          const msg = ev.message;
          if (msg.case === "stationsState") {
            const states = msg.value.state;
            for (const s of states) {
              if (s.connected) connectedIpsRef.current.add(s.ip);
              else connectedIpsRef.current.delete(s.ip);
            }
            setSeatMeta((prev) => {
              const next = new Map(prev);
              for (const s of states) {
                const seatId = ipToSeatRef.current.get(s.ip);
                if (!seatId) continue;
                const cur = next.get(seatId) ?? {
                  connected: false,
                  teamName: null,
                };
                next.set(seatId, { ...cur, connected: s.connected });
              }
              return next;
            });
          } else if (msg.case === "stationAssignments") {
            const updates = msg.value.updates;
            setElements((prev) => updateSeatIps(prev, updates));
            for (const a of updates) {
              for (const [ip, sid] of ipToSeatRef.current) {
                if (sid === a.seatId) ipToSeatRef.current.delete(ip);
              }
              if (a.ip) ipToSeatRef.current.set(a.ip, a.seatId);
            }
            setSeatMeta((prev) => {
              const next = new Map(prev);
              for (const a of updates) {
                const cur = next.get(a.seatId) ?? {
                  connected: false,
                  teamName: null,
                };
                const connected = a.ip
                  ? connectedIpsRef.current.has(a.ip)
                  : false;
                next.set(a.seatId, {
                  ...cur,
                  teamName: a.teamName ?? null,
                  connected,
                });
              }
              return next;
            });
          }
        }
      } catch (e: unknown) {
        if (!controller.signal.aborted) {
          setError(`Subscription error: ${formatErr(e)}`);
        }
      }
    })();
    return () => controller.abort();
  }, [client]);

  const handleClick = async (el: MapElement) => {
    if (el.kind !== "seat" || !client || !config) return;
    try {
      await client.assignStationToSeat(el.id, config.ip);
    } catch (e: unknown) {
      setError(`Assignment failed: ${formatErr(e)}`);
    }
  };

  const seatOverlay = (seat: Seat) => {
    const meta = seatMeta.get(seat.id);
    return (
      <SeatOverlay
        seat={seat}
        connected={meta?.connected ?? false}
        teamName={meta?.teamName ?? null}
      />
    );
  };

  return (
    <div
      style={{
        height: "100vh",
        display: "grid",
        gridTemplateRows: "auto 1fr",
      }}
      className="bg-surface-900 text-gray-100"
    >
      <header
        data-tauri-drag-region
        className="bg-surface-800 border-b border-surface-600 px-6 py-3 flex items-center gap-4 shrink-0 select-none"
      >
        <div className="w-9 h-9 rounded-lg bg-linear-to-br from-primary-500/20 to-primary-400/20 border border-primary-500/30 flex items-center justify-center">
          <svg
            className="w-5 h-5 text-primary-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"
            />
          </svg>
        </div>
        <div className="flex-1">
          <h1 className="font-medium text-white">Station Registration</h1>
          <p className="text-xs text-gray-500">
            Click a seat to claim it for this machine
          </p>
        </div>
        <button
          type="button"
          onClick={() => void loadMap()}
          disabled={loading}
          title="Reload map"
          className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-surface-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <svg
            className={"w-5 h-5 " + (loading ? "animate-spin" : "")}
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.8}
              d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
            />
          </svg>
        </button>
        <StatusPill
          ip={config?.ip ?? null}
          assignedSeat={myAssignedSeat ?? null}
          elementCount={elements.length}
          loading={loading}
        />
      </header>

      <main
        className="relative"
        style={{ minHeight: 0, position: "relative" }}
      >
        {error && (
          <Banner
            kind="error"
            message={error}
            onClose={() => setError(null)}
          />
        )}
        {loading ? (
          <div className="absolute inset-0 flex items-center justify-center text-gray-400">
            Loading map…
          </div>
        ) : elements.length === 0 ? (
          <EmptyMapState />
        ) : (
          <MapViewer
            elements={elements}
            onElementClick={handleClick}
            seatOverlay={seatOverlay}
            fitToContent
          />
        )}
      </main>
    </div>
  );
}

function EmptyMapState() {
  return (
    <div className="absolute inset-0 flex items-center justify-center p-8">
      <div className="max-w-md text-center space-y-4">
        <div className="mx-auto w-14 h-14 rounded-full bg-surface-800 border border-surface-600 flex items-center justify-center">
          <svg
            className="w-7 h-7 text-gray-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9 20l-5.447-2.724A1 1 0 013 16.382V5.618a1 1 0 011.447-.894L9 7m0 13l6-3m-6 3V7m6 10l4.553 2.276A1 1 0 0021 18.382V7.618a1 1 0 00-.553-.894L15 4m0 13V4m0 0L9 7"
            />
          </svg>
        </div>
        <div>
          <h2 className="text-lg font-medium text-gray-200">
            No map available
          </h2>
          <p className="mt-2 text-sm text-gray-400">
            This contest does not have a map drawn yet. Ask an organizer to
            create one in the dashboard, then reopen this app to claim a seat.
          </p>
        </div>
      </div>
    </div>
  );
}

function StatusPill({
  ip,
  assignedSeat,
  elementCount,
  loading,
}: {
  ip: string | null;
  assignedSeat: Seat | null;
  elementCount: number;
  loading: boolean;
}) {
  const claimed = assignedSeat !== null;
  return (
    <div className="flex items-center gap-3">
      <div className="text-right">
        <div className="text-xs text-gray-500">
          {loading ? "Loading…" : `${elementCount} map elements`}
        </div>
        <div className="text-sm font-mono text-gray-200">{ip ?? "…"}</div>
      </div>
      <div
        className={
          "px-3 py-1 rounded-full text-xs border " +
          (claimed
            ? "bg-success-500/15 border-success-500/40 text-success-500"
            : "bg-surface-700 border-surface-600 text-gray-400")
        }
      >
        {claimed ? "Claimed" : "Unclaimed"}
      </div>
    </div>
  );
}

function Banner({
  kind,
  message,
  onClose,
}: {
  kind: "error" | "info";
  message: string;
  onClose: () => void;
}) {
  const styles =
    kind === "error"
      ? "bg-danger-600/90 border-danger-500 text-white"
      : "bg-primary-600/90 border-primary-400 text-white";
  return (
    <div
      className={`absolute top-3 left-1/2 -translate-x-1/2 z-10 px-4 py-2 rounded-md border shadow-lg flex items-center gap-3 text-sm ${styles}`}
    >
      <span>{message}</span>
      <button
        type="button"
        onClick={onClose}
        className="px-2 py-0.5 rounded bg-white/10 hover:bg-white/20 transition-colors text-xs"
      >
        Close
      </button>
    </div>
  );
}

function updateSeatIps(
  prev: MapElement[],
  updates: { seatId: string; ip?: string }[],
): MapElement[] {
  const ipBySeat = new Map<string, string | null>();
  for (const u of updates) ipBySeat.set(u.seatId, u.ip ?? null);
  return prev.map((el) => {
    if (el.kind !== "seat") return el;
    if (!ipBySeat.has(el.id)) return el;
    return { ...el, ip: ipBySeat.get(el.id) ?? null };
  });
}

function formatErr(e: unknown): string {
  if (e instanceof Error) return e.message;
  return String(e);
}
