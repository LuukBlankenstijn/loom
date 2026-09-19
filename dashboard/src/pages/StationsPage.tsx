import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { Fragment, useEffect, useRef, useState } from "react";
import { adminClient } from "../lib/client";
import { AssignModal } from "../components/AssignModal";
import { StationActinoModal } from "../components/StationActionModal";
import { StationTerminal } from "../components/StationTerminal";
import { StationLabel } from "../components/StationLabel";
import type { StationTarget } from "../lib/actions";
import { useStationState } from "../context/station-state";
import type { Station } from "@client/v1/admin/station_pb";

type SortKey = "connected" | "loggedIn";
type ConnectionKey = "online" | "offline";
type SessionKey = "in" | "out";
type Selection<K extends string> = Record<K, boolean>;

const CONNECTION_OPTIONS: { key: ConnectionKey; label: string }[] = [
  { key: "online", label: "Online" },
  { key: "offline", label: "Offline" },
];

const SESSION_OPTIONS: { key: SessionKey; label: string }[] = [
  { key: "in", label: "Logged in" },
  { key: "out", label: "Logged out" },
];

const DEFAULT_CONNECTION: Selection<ConnectionKey> = {
  online: true,
  offline: false,
};

const DEFAULT_SESSION: Selection<SessionKey> = { in: true, out: true };

const ALL_LABELS = "all";
const NO_LABEL = "none";
const LABEL_PREFIX = "label:";

function matchesLabel(station: Station, filter: string) {
  if (filter === ALL_LABELS) return true;
  if (filter === NO_LABEL) return !station.label;
  return !!station.label && filter === `${LABEL_PREFIX}${station.label}`;
}

function FilterMenu({
  connection,
  session,
  onConnectionToggle,
  onSessionToggle,
  onReset,
}: {
  connection: Selection<ConnectionKey>;
  session: Selection<SessionKey>;
  onConnectionToggle: (key: ConnectionKey) => void;
  onSessionToggle: (key: SessionKey) => void;
  onReset: () => void;
}) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const hiddenCount = [
    ...Object.values(connection),
    ...Object.values(session),
  ].filter((selected) => !selected).length;

  useEffect(() => {
    if (!open) return;

    const closeOnOutsideClick = (event: MouseEvent) => {
      if (!containerRef.current?.contains(event.target as Node)) {
        setOpen(false);
      }
    };

    document.addEventListener("mousedown", closeOnOutsideClick);
    return () => document.removeEventListener("mousedown", closeOnOutsideClick);
  }, [open]);

  return (
    <div ref={containerRef} className="relative">
      <button
        onClick={() => setOpen(!open)}
        className={`flex items-center gap-2 px-3 py-2 rounded-lg border text-sm transition-colors ${
          hiddenCount > 0
            ? "bg-primary-500/10 border-primary-500/30 text-primary-400"
            : "bg-surface-800 border-surface-600 text-gray-300 hover:bg-surface-700"
        }`}
      >
        Filters
        {hiddenCount > 0 && (
          <span className="px-1.5 rounded-full bg-primary-500/20 text-xs">
            {hiddenCount}
          </span>
        )}
        <svg
          className={`w-3 h-3 transition-transform ${open ? "rotate-180" : ""}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>
      {open && (
        <div className="absolute right-0 z-10 mt-2 w-60 p-3 space-y-3 bg-surface-800 border border-surface-600 rounded-xl shadow-xl">
          <div>
            <p className="text-xs uppercase tracking-wide text-gray-500 mb-1.5">
              Connection
            </p>
            <div className="flex gap-1">
              {CONNECTION_OPTIONS.map((option) => (
                <button
                  key={option.key}
                  onClick={() => onConnectionToggle(option.key)}
                  className={`flex-1 px-2 py-1.5 rounded-md text-xs transition-colors ${
                    connection[option.key]
                      ? "bg-primary-500 text-white"
                      : "bg-surface-700 text-gray-400 hover:bg-surface-600"
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>
          <div>
            <p className="text-xs uppercase tracking-wide text-gray-500 mb-1.5">
              Session
            </p>
            <div className="flex gap-1">
              {SESSION_OPTIONS.map((option) => (
                <button
                  key={option.key}
                  onClick={() => onSessionToggle(option.key)}
                  className={`flex-1 px-2 py-1.5 rounded-md text-xs transition-colors ${
                    session[option.key]
                      ? "bg-primary-500 text-white"
                      : "bg-surface-700 text-gray-400 hover:bg-surface-600"
                  }`}
                >
                  {option.label}
                </button>
              ))}
            </div>
          </div>
          <button
            onClick={onReset}
            className="w-full py-1.5 rounded-md text-xs text-gray-400 hover:text-white hover:bg-surface-700 transition-colors"
          >
            Reset to online stations
          </button>
        </div>
      )}
    </div>
  );
}

function SortHeader({
  label,
  active,
  direction,
  onClick,
}: {
  label: string;
  active: boolean;
  direction: "asc" | "desc";
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-1.5 text-sm font-semibold text-gray-300 hover:text-white transition-colors"
    >
      {label}
      <svg
        className={`w-3 h-3 transition-transform ${active ? "text-primary-400" : "text-gray-600"} ${active && direction === "asc" ? "rotate-180" : ""}`}
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M19 9l-7 7-7-7"
        />
      </svg>
    </button>
  );
}

export function StationsPage() {
  const queryClient = useQueryClient();
  const [modalOpen, setModalOpen] = useState(false);
  const [selectedStation, setSelectedStation] = useState<Station | null>(null);
  const [selectedIps, setSelectedIps] = useState<Set<string>>(new Set());
  const [actionTargetStations, setActionTargetStations] = useState<
    StationTarget[] | null
  >(null);
  const [expandedIps, setExpandedIps] = useState<Set<string>>(new Set());
  const { getState: getStationsState, connectedCount } = useStationState();
  const [search, setSearch] = useState("");
  const [labelFilter, setLabelFilter] = useState(ALL_LABELS);
  const [connection, setConnection] =
    useState<Selection<ConnectionKey>>(DEFAULT_CONNECTION);
  const [session, setSession] =
    useState<Selection<SessionKey>>(DEFAULT_SESSION);
  const [sort, setSort] = useState<{
    key: SortKey;
    direction: "asc" | "desc";
  } | null>(null);

  const { data: stationsData, isLoading } = useQuery({
    queryKey: ["stations"],
    queryFn: () => adminClient.getStations(),
  });

  const { data: teamsData } = useQuery({
    queryKey: ["teams"],
    queryFn: () => adminClient.getActiveTeams(),
  });

  const stations = stationsData?.stations ?? [];

  const connectedKey = `${connectedCount}:${stations
    .filter((station) => getStationsState(station.ip).connected)
    .map((station) => station.ip)
    .sort()
    .join(",")}`;
  const lastConnectedKey = useRef<string | null>(null);

  useEffect(() => {
    if (!stationsData) return;
    if (
      lastConnectedKey.current !== null &&
      lastConnectedKey.current !== connectedKey
    ) {
      queryClient.invalidateQueries({ queryKey: ["stations"] });
    }
    lastConnectedKey.current = connectedKey;
  }, [connectedKey, stationsData, queryClient]);

  const teams = teamsData?.teams ?? [];

  const ipToTeam = new Map(teams.filter((t) => t.ip).map((t) => [t.ip!, t]));

  const query = search.trim().toLowerCase();

  const matchedStations = stations.filter((station) => {
    const { connected, loggedIn } = getStationsState(station.ip);
    if (
      query &&
      !station.ip.toLowerCase().includes(query) &&
      !station.label?.includes(query)
    )
      return false;
    if (!matchesLabel(station, labelFilter)) return false;
    if (!connection[connected ? "online" : "offline"]) return false;
    return session[loggedIn ? "in" : "out"];
  });

  const selectedLabel = labelFilter.startsWith(LABEL_PREFIX)
    ? [labelFilter.slice(LABEL_PREFIX.length)]
    : [];
  const labels = [
    ...new Set([
      ...stations.flatMap((station) => (station.label ? [station.label] : [])),
      ...selectedLabel,
    ]),
  ].sort();

  const visibleStations = sort
    ? [...matchedStations].sort(
        (a, b) =>
          (Number(getStationsState(b.ip)[sort.key]) -
            Number(getStationsState(a.ip)[sort.key])) *
          (sort.direction === "desc" ? 1 : -1),
      )
    : matchedStations;

  const allSelected =
    visibleStations.length > 0 &&
    visibleStations.every((station) => selectedIps.has(station.ip));
  const someSelected = selectedIps.size > 0 && !allSelected;

  const toggleAll = () => {
    setSelectedIps(
      allSelected
        ? new Set()
        : new Set(visibleStations.map((station) => station.ip)),
    );
  };

  const toggleSort = (key: SortKey) => {
    setSort((prev) => {
      if (prev?.key !== key) return { key, direction: "desc" };
      return prev.direction === "desc" ? { key, direction: "asc" } : null;
    });
  };

  const toggleOne = (ip: string) => {
    setSelectedIps((prev) => {
      const next = new Set(prev);
      if (next.has(ip)) {
        next.delete(ip);
      } else {
        next.add(ip);
      }
      return next;
    });
  };

  const unassignMutation = useMutation({
    mutationFn: (teamId: string) => adminClient.setIp(teamId, undefined),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
      queryClient.invalidateQueries({ queryKey: ["stations"] });
    },
  });

  const toggleTerminal = (ip: string) => {
    setExpandedIps((prev) => {
      const next = new Set(prev);
      if (next.has(ip)) {
        next.delete(ip);
      } else {
        next.add(ip);
      }
      return next;
    });
  };

  const openAssignModal = (station: Station) => {
    setSelectedStation(station);
    setModalOpen(true);
  };

  return (
    <div className="max-w-7xl mx-auto px-6 py-10">
      <div className="flex items-center gap-3 mb-8">
        <div className="w-2 h-8 bg-linear-to-b from-emerald-400 to-emerald-600 rounded-full" />
        <h1 className="text-3xl font-semibold text-white">Stations</h1>
        <div className="ml-auto flex gap-2">
          <button
            onClick={() =>
              setActionTargetStations(
                stations
                  .filter((s) => selectedIps.has(s.ip))
                  .map((s) => ({ ...s, team: ipToTeam.get(s.ip) })),
              )
            }
            disabled={selectedIps.size === 0}
            className="px-3 py-1 bg-primary-500 hover:bg-primary-600 disabled:bg-surface-600 disabled:text-gray-500 text-white text-sm rounded-full transition-colors"
          >
            Actions ({selectedIps.size})
          </button>
          <span className="px-3 py-1 bg-success-500/20 text-success-500 rounded-full text-sm">
            {connectedCount} online
          </span>
          <span className="px-3 py-1 bg-surface-600 text-gray-400 rounded-full text-sm">
            {stations.length} total
          </span>
        </div>
      </div>
      <div className="flex items-center justify-end gap-2 mb-4">
        <div className="relative">
          <svg
            className="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M21 21l-4.35-4.35M17 11a6 6 0 11-12 0 6 6 0 0112 0z"
            />
          </svg>
          <input
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search IP or label"
            className="w-56 bg-surface-800 border border-surface-600 rounded-lg pl-9 pr-3 py-2 text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:border-primary-500"
          />
        </div>
        <select
          value={labelFilter}
          onChange={(e) => setLabelFilter(e.target.value)}
          className={`px-3 py-2 rounded-lg border text-sm focus:outline-none focus:border-primary-500 transition-colors ${
            labelFilter === ALL_LABELS
              ? "bg-surface-800 border-surface-600 text-gray-300"
              : "bg-primary-500/10 border-primary-500/30 text-primary-400"
          }`}
        >
          <option value={ALL_LABELS}>All labels</option>
          {labels.map((label) => (
            <option key={label} value={`${LABEL_PREFIX}${label}`}>
              {label}
            </option>
          ))}
          <option value={NO_LABEL}>No label</option>
        </select>
        <FilterMenu
          connection={connection}
          session={session}
          onConnectionToggle={(key) =>
            setConnection((prev) => ({ ...prev, [key]: !prev[key] }))
          }
          onSessionToggle={(key) =>
            setSession((prev) => ({ ...prev, [key]: !prev[key] }))
          }
          onReset={() => {
            setConnection(DEFAULT_CONNECTION);
            setSession(DEFAULT_SESSION);
          }}
        />
      </div>
      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : stations.length === 0 ? (
        <div className="text-center py-20 text-gray-400">No stations found</div>
      ) : visibleStations.length === 0 ? (
        <div className="text-center py-20">
          <p className="text-gray-400">No stations match your filters</p>
          <p className="text-gray-600 text-sm mt-1">
            Widen the search or turn a filter back on to see stations again
          </p>
        </div>
      ) : (
        <div className="bg-surface-800 rounded-xl border border-surface-600 overflow-hidden shadow-xl">
          <table className="w-full">
            <thead>
              <tr className="border-b border-surface-600 bg-surface-800/50">
                <th className="px-6 py-4 w-12">
                  <button
                    onClick={toggleAll}
                    className={`p-1.5 rounded-md transition-all border active:scale-90 ${
                      allSelected
                        ? "bg-primary-500/10 text-primary-500 border-primary-500/20 hover:bg-primary-500/20"
                        : someSelected
                          ? "bg-gray-500/10 text-gray-400 border-gray-500/20 hover:bg-gray-500/20"
                          : "bg-gray-500/10 text-transparent border-gray-500/20 hover:bg-gray-500/20 hover:text-gray-500"
                    }`}
                  >
                    <svg
                      className="w-3.5 h-3.5"
                      fill="none"
                      stroke="currentColor"
                      viewBox="0 0 24 24"
                    >
                      {someSelected ? (
                        <path
                          strokeLinecap="round"
                          strokeWidth={2}
                          d="M6 12h12"
                        />
                      ) : (
                        <path
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          strokeWidth={2}
                          d="M5 13l4 4L19 7"
                        />
                      )}
                    </svg>
                  </button>
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-44">
                  IP Address
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-44">
                  Label
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300">
                  Team
                </th>
                <th className="text-left px-6 py-4 w-36">
                  <SortHeader
                    label="Logged In"
                    active={sort?.key === "loggedIn"}
                    direction={sort?.direction ?? "desc"}
                    onClick={() => toggleSort("loggedIn")}
                  />
                </th>
                <th className="text-left px-6 py-4 w-36">
                  <SortHeader
                    label="Status"
                    active={sort?.key === "connected"}
                    direction={sort?.direction ?? "desc"}
                    onClick={() => toggleSort("connected")}
                  />
                </th>
                <th className="text-left px-6 py-4 text-sm font-semibold text-gray-300 w-32">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-surface-700">
              {visibleStations.map((station) => {
                const team = ipToTeam.get(station.ip);
                const connectionState = getStationsState(station.ip);

                return (
                  <Fragment key={station.ip}>
                    <tr className="hover:bg-surface-700/50 transition-colors">
                      <td className="px-6 py-4">
                        <button
                          onClick={() => toggleOne(station.ip)}
                          className={`p-1.5 rounded-md transition-all border active:scale-90 ${
                            selectedIps.has(station.ip)
                              ? "bg-primary-500/10 text-primary-500 border-primary-500/20 hover:bg-primary-500/20"
                              : "bg-gray-500/10 text-transparent border-gray-500/20 hover:bg-gray-500/20 hover:text-gray-500"
                          }`}
                        >
                          <svg
                            className="w-3.5 h-3.5"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                          >
                            <path
                              strokeLinecap="round"
                              strokeLinejoin="round"
                              strokeWidth={2}
                              d="M5 13l4 4L19 7"
                            />
                          </svg>
                        </button>
                      </td>
                      <td className="px-6 py-4 font-mono text-gray-200">
                        {station.ip}
                      </td>
                      <td className="px-6 py-4">
                        <StationLabel station={station} />
                      </td>
                      <td className="px-6 py-4">
                        {team ? (
                          <div className="flex items-center gap-3">
                            <span className="text-gray-200 bg-purple-500/10 px-2 py-1 rounded">
                              {team.name}
                            </span>
                            <button
                              onClick={() => unassignMutation.mutate(team.id)}
                              disabled={unassignMutation.isPending}
                              className="p-1.5 rounded-md bg-danger-500/10 text-danger-500 hover:bg-danger-500/20 hover:text-danger-400 transition-colors border border-danger-500/20"
                              title="Unassign team"
                            >
                              <svg
                                className="w-3.5 h-3.5"
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
                        ) : (
                          <button
                            onClick={() => openAssignModal(station)}
                            className="px-3 py-1.5 bg-primary-500 hover:bg-primary-600 text-white text-sm rounded-lg transition-colors whitespace-nowrap"
                          >
                            Assign Team
                          </button>
                        )}
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-2">
                          <span
                            className={`w-2.5 h-2.5 rounded-full ${
                              connectionState.loggedIn
                                ? "bg-success-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"
                                : "bg-danger-500"
                            }`}
                          />
                          <span
                            className={`text-sm ${connectionState.loggedIn ? "text-success-500" : "text-gray-500"}`}
                          >
                            {connectionState.loggedIn ? "Yes" : "No"}
                          </span>
                        </div>
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex items-center gap-2">
                          <span
                            className={`w-2.5 h-2.5 rounded-full ${
                              connectionState.connected
                                ? "bg-success-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]"
                                : "bg-danger-500"
                            }`}
                          />
                          <span
                            className={`text-sm ${connectionState.connected ? "text-success-500" : "text-gray-500"}`}
                          >
                            {connectionState.connected ? "Online" : "Offline"}
                          </span>
                        </div>
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex">
                          <button
                            onClick={() =>
                              setActionTargetStations([{ ...station, team }])
                            }
                            className="px-3 py-1.5 bg-surface-600 hover:bg-surface-500 text-gray-300 text-sm rounded-l-lg transition-colors"
                          >
                            Actions
                          </button>
                          <button
                            onClick={() => toggleTerminal(station.ip)}
                            title="Toggle console"
                            className={`px-1.5 py-1.5 text-sm rounded-r-lg border-l border-surface-700 transition-all ${
                              expandedIps.has(station.ip)
                                ? "bg-emerald-500/20 text-emerald-400 hover:bg-emerald-500/30"
                                : "bg-surface-600 hover:bg-surface-500 text-gray-400"
                            }`}
                          >
                            <svg
                              className={`w-3.5 h-3.5 transition-transform ${expandedIps.has(station.ip) ? "rotate-180" : ""}`}
                              fill="none"
                              stroke="currentColor"
                              viewBox="0 0 24 24"
                            >
                              <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M19 9l-7 7-7-7"
                              />
                            </svg>
                          </button>
                        </div>
                      </td>
                    </tr>
                    {expandedIps.has(station.ip) && (
                      <tr>
                        <td colSpan={7} className="p-0">
                          <StationTerminal ip={station.ip} />
                        </td>
                      </tr>
                    )}
                  </Fragment>
                );
              })}
            </tbody>
          </table>
        </div>
      )}

      {modalOpen && selectedStation && (
        <AssignModal
          mode="station"
          stationIp={selectedStation.ip}
          stations={stations}
          teams={teams}
          onClose={() => {
            setModalOpen(false);
            setSelectedStation(null);
          }}
        />
      )}

      {actionTargetStations && (
        <StationActinoModal
          stations={actionTargetStations}
          onClose={() => setActionTargetStations(null)}
        />
      )}
    </div>
  );
}
