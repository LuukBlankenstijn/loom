import { useEffect, useMemo, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import {
  MapViewer,
  SeatOverlay,
  fromProtoElements,
  type MapElement,
  type Seat,
} from "@loom/map-react";
import { BroadcastType } from "@client/v1/broadcast/broadcast_pb";
import { adminClient } from "../lib/client";

type SeatMeta = { connected: boolean; teamName: string | null };

type StationItem = {
  ip: string;
  connected: boolean;
  assignedSeatId: string | null;
};

export function MapViewerPage() {
  const { mapId } = useParams<{ mapId: string }>();
  const id = Number(mapId);

  const { data: mapsData } = useQuery({
    queryKey: ["maps"],
    queryFn: () => adminClient.getAllMaps(),
  });
  const map = mapsData?.maps.find((m) => m.id === id);

  const { data: mapData, isLoading } = useQuery({
    queryKey: ["map", id],
    queryFn: () => adminClient.getMap(id),
    enabled: Number.isFinite(id),
  });

  const { data: stationsData } = useQuery({
    queryKey: ["stations"],
    queryFn: () => adminClient.getStations(),
  });

  const [elements, setElements] = useState<MapElement[]>([]);
  const [connectedIps, setConnectedIps] = useState<Set<string>>(new Set());
  const [teamByIp, setTeamByIp] = useState<Map<string, string>>(new Map());
  const [activeSeatId, setActiveSeatId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!mapData) return;
    setElements(fromProtoElements(mapData.elements));
  }, [mapData]);

  useEffect(() => {
    const controller = new AbortController();
    (async () => {
      try {
        for await (const ev of adminClient.subscribe(controller.signal, [
          BroadcastType.CONNECTION_STATE,
          BroadcastType.STATION_ASSIGNMENTS,
        ])) {
          const msg = ev.message;
          if (msg.case === "stationsState") {
            setConnectedIps((prev) => {
              const next = new Set(prev);
              for (const s of msg.value.state) {
                if (s.connected) next.add(s.ip);
                else next.delete(s.ip);
              }
              return next;
            });
          } else if (msg.case === "stationAssignments") {
            const updates = msg.value.updates;
            setElements((prev) => updateSeatIps(prev, updates));
            setTeamByIp((prev) => {
              const next = new Map(prev);
              for (const a of updates) {
                if (!a.ip) continue;
                if (a.teamName) next.set(a.ip, a.teamName);
                else next.delete(a.ip);
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
  }, []);

  const seatMeta = useMemo(() => {
    const meta = new Map<string, SeatMeta>();
    for (const el of elements) {
      if (el.kind !== "seat") continue;
      const connected = el.ip ? connectedIps.has(el.ip) : false;
      const teamName = el.ip ? teamByIp.get(el.ip) ?? null : null;
      meta.set(el.id, { connected, teamName });
    }
    return meta;
  }, [elements, connectedIps, teamByIp]);

  const seatByIp = useMemo(() => {
    const m = new Map<string, string>();
    for (const el of elements) {
      if (el.kind === "seat" && el.ip) m.set(el.ip, el.id);
    }
    return m;
  }, [elements]);

  const stationItems: StationItem[] = useMemo(() => {
    const list = stationsData?.stations ?? [];
    return list.map((s) => ({
      ip: s.ip,
      connected: connectedIps.has(s.ip),
      assignedSeatId: seatByIp.get(s.ip) ?? null,
    }));
  }, [stationsData, connectedIps, seatByIp]);

  const activeSeat = useMemo<Seat | null>(() => {
    if (!activeSeatId) return null;
    const seat = elements.find(
      (e) => e.kind === "seat" && e.id === activeSeatId,
    );
    return (seat as Seat) ?? null;
  }, [activeSeatId, elements]);

  const activeMeta = activeSeat ? seatMeta.get(activeSeat.id) ?? null : null;

  const handleSeatClick = (el: MapElement) => {
    if (el.kind === "seat") setActiveSeatId(el.id);
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

  const handleAssign = async (stationIp: string | undefined) => {
    if (!activeSeat) return;
    try {
      await adminClient.assignStationToSeat(activeSeat.id, stationIp);
      setActiveSeatId(null);
    } catch (e: unknown) {
      setError(`Assignment failed: ${formatErr(e)}`);
    }
  };

  return (
    <div className="h-screen flex flex-col bg-surface-900">
      <div className="bg-surface-800 border-b border-surface-600 px-6 py-3 flex items-center gap-4 shrink-0">
        <Link
          to="/maps"
          className="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-surface-700 transition-colors"
        >
          <svg
            className="w-5 h-5"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M15 19l-7-7 7-7"
            />
          </svg>
        </Link>
        <div className="w-px h-6 bg-surface-600" />
        <div className="flex items-center gap-3 flex-1">
          <div className="w-8 h-8 rounded-lg bg-linear-to-br from-violet-500/20 to-purple-500/20 border border-violet-500/30 flex items-center justify-center">
            <svg
              className="w-4 h-4 text-violet-400"
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
            <h1 className="text-white font-medium">
              {map?.name ?? `Map #${mapId}`}
            </h1>
            <p className="text-xs text-gray-500">
              Click a seat to assign or unassign a station
            </p>
          </div>
        </div>
        <Link
          to={`/maps/${id}/edit`}
          className="px-3 py-1.5 text-sm bg-surface-700 hover:bg-surface-600 text-gray-300 rounded-lg"
        >
          Edit
        </Link>
      </div>

      <div className="flex-1 relative">
        {error && (
          <div className="absolute top-3 left-1/2 -translate-x-1/2 z-10 px-4 py-2 rounded-md border bg-danger-600/90 border-danger-500 text-white text-sm flex items-center gap-3 shadow-lg">
            <span>{error}</span>
            <button
              type="button"
              onClick={() => setError(null)}
              className="px-2 py-0.5 rounded bg-white/10 hover:bg-white/20 text-xs"
            >
              Close
            </button>
          </div>
        )}

        {isLoading ? (
          <div className="absolute inset-0 flex items-center justify-center text-gray-400">
            Loading map…
          </div>
        ) : elements.length === 0 ? (
          <div className="absolute inset-0 flex items-center justify-center text-gray-400">
            This map has no elements yet.
          </div>
        ) : (
          <MapViewer
            elements={elements}
            onElementClick={handleSeatClick}
            seatOverlay={seatOverlay}
            fitToContent
          />
        )}

        {activeSeat && (
          <AssignPanel
            seat={activeSeat}
            currentIp={activeSeat.ip ?? null}
            currentTeamName={activeMeta?.teamName ?? null}
            stations={stationItems}
            onAssign={handleAssign}
            onClose={() => setActiveSeatId(null)}
          />
        )}
      </div>
    </div>
  );
}

function AssignPanel({
  seat,
  currentIp,
  currentTeamName,
  stations,
  onAssign,
  onClose,
}: {
  seat: Seat;
  currentIp: string | null;
  currentTeamName: string | null;
  stations: StationItem[];
  onAssign: (ip: string | undefined) => void;
  onClose: () => void;
}) {
  return (
    <div className="absolute right-4 top-4 bottom-4 w-80 bg-surface-800 border border-surface-600 rounded-xl shadow-2xl flex flex-col">
      <div className="px-4 py-3 border-b border-surface-700 flex items-center justify-between">
        <div>
          <div className="text-xs text-gray-500 font-mono">
            seat {seat.id.slice(0, 8)}…
          </div>
          <div className="text-sm text-gray-200">
            {currentTeamName ?? "Unassigned"}
          </div>
        </div>
        <button
          type="button"
          onClick={onClose}
          className="p-1 rounded hover:bg-surface-700 text-gray-400 hover:text-white"
          aria-label="Close"
        >
          <svg
            className="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      {currentIp && (
        <div className="px-4 py-3 border-b border-surface-700">
          <button
            type="button"
            onClick={() => onAssign(undefined)}
            className="w-full px-3 py-2 text-sm bg-danger-500/15 hover:bg-danger-500/25 border border-danger-500/40 text-danger-500 rounded-lg"
          >
            Unassign
          </button>
        </div>
      )}

      <div className="flex-1 overflow-y-auto">
        <div className="px-4 py-2 text-xs uppercase tracking-wide text-gray-500">
          Stations
        </div>
        {stations.length === 0 && (
          <div className="px-4 py-3 text-sm text-gray-500">
            No stations registered yet.
          </div>
        )}
        {stations.map((s) => {
          const selected = currentIp === s.ip;
          const assignedElsewhere =
            s.assignedSeatId !== null && s.assignedSeatId !== seat.id;
          return (
            <button
              key={s.ip}
              type="button"
              onClick={() => onAssign(s.ip)}
              disabled={selected}
              className={
                "w-full px-4 py-2 flex items-center justify-between text-sm transition-colors gap-2 " +
                (selected
                  ? "bg-violet-500/15 text-violet-300 cursor-default"
                  : "hover:bg-surface-700 text-gray-200")
              }
            >
              <span className="font-mono truncate">{s.ip}</span>
              <span className="flex items-center gap-1.5 shrink-0">
                <span
                  className={
                    "text-xs px-2 py-0.5 rounded-full " +
                    (assignedElsewhere
                      ? "bg-amber-500/20 text-amber-400"
                      : selected
                        ? "bg-violet-500/20 text-violet-300"
                        : "bg-surface-700 text-gray-500")
                  }
                >
                  {assignedElsewhere
                    ? "assigned"
                    : selected
                      ? "this seat"
                      : "unassigned"}
                </span>
                <span
                  className={
                    "text-xs px-2 py-0.5 rounded-full " +
                    (s.connected
                      ? "bg-success-500/20 text-success-500"
                      : "bg-surface-700 text-gray-500")
                  }
                >
                  {s.connected ? "online" : "offline"}
                </span>
              </span>
            </button>
          );
        })}
      </div>
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
