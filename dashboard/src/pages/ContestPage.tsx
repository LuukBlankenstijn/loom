import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";
import { adminClient } from "../lib/client";
import { WallpaperMedia } from "../components/WallpaperMedia";
import { timestampDate } from "@bufbuild/protobuf/wkt";
import type { Timestamp } from "@bufbuild/protobuf/wkt";
import { useStationState } from "../context/station-state";
import type { Station } from "@client/v1/admin/station_pb";

export function ContestPage() {
  const queryClient = useQueryClient();
  const [showPreview, setShowPreview] = useState<boolean>(false);
  const { getState: getConnectedState } = useStationState();

  const { data: contest, isLoading } = useQuery({
    queryKey: ["contest"],
    queryFn: () => adminClient.getNextContest(),
  });

  const { data: wallpapersData, isLoading: wallpapersLoading } = useQuery({
    queryKey: ["wallpapers"],
    queryFn: () => adminClient.listWallpapers(),
  });

  const { data: teamsData } = useQuery({
    queryKey: ["teams"],
    queryFn: () => adminClient.getActiveTeams(),
  });

  const { data: stationsData } = useQuery({
    queryKey: ["stations"],
    queryFn: () => adminClient.getStations(),
  });

  const { data: mapsData } = useQuery({
    queryKey: ["maps"],
    queryFn: () => adminClient.getAllMaps(),
  });

  const setWallpaperMutation = useMutation({
    mutationFn: (wallpaperId?: number) =>
      adminClient.setWallpaper(contest?.id ?? "", wallpaperId),
    onSuccess: () =>
      Promise.all([
        queryClient.invalidateQueries({ queryKey: ["contest"] }),
        queryClient.invalidateQueries({ queryKey: ["wallpapers"] }),
      ]),
  });

  const setMapMutation = useMutation({
    mutationFn: (mapId: number) => adminClient.setMap(contest?.id ?? "", mapId),
    onSuccess: () =>
      queryClient.invalidateQueries({ queryKey: ["contest"] }),
    onError: (error) => {
      console.error("Failed to set map:", error);
    },
  });

  const formatDate = (timestamp: Timestamp | undefined) => {
    if (!timestamp) return "N/A";
    const date = timestampDate(timestamp);
    return new Intl.DateTimeFormat(undefined, {
      dateStyle: "long",
      timeStyle: "short",
    }).format(date);
  };

  const isStationConnected = (station: Station) =>
    getConnectedState(station.ip).connected;

  const teams = teamsData?.teams ?? [];
  const stations = stationsData?.stations ?? [];
  const maps = mapsData?.maps ?? [];
  const wallpapers = wallpapersData?.wallpapers ?? [];
  const assignedWallpaper = wallpapers.find(
    (w) => w.id === contest?.wallpaperId,
  );
  const defaultWallpaper = wallpapers.find((w) => w.isDefault);
  const shownWallpaper = assignedWallpaper ?? defaultWallpaper;
  const selectedWallpaperId = setWallpaperMutation.isPending
    ? setWallpaperMutation.variables
    : contest?.wallpaperId;
  const totalTeams = teams.length;

  const connectedTeams = teams.filter((team) => {
    if (!team.ip) return false;
    const station = stations.find((s) => s.ip === team.ip);
    return station && isStationConnected(station);
  }).length;

  return (
    <div className="h-full flex">
      <aside className="w-80 bg-surface-800 border-r border-surface-600 flex flex-col overflow-hidden">
        <h2 className="text-2xl font-semibold text-white px-6 pt-6 pb-4 shrink-0">
          Contest Details
        </h2>
        <div
          className="flex-1 overflow-y-auto min-h-0 px-6 pb-6 scrollbar-none"
          style={{ scrollbarWidth: "none", msOverflowStyle: "none" }}
        >
          {isLoading ? (
            <div className="text-gray-400">Loading...</div>
          ) : contest ? (
            <div className="space-y-6 flex-1">
              <div>
                <h3 className="text-xl font-medium text-white">
                  {contest.name}
                </h3>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="p-4 rounded-lg bg-surface-700 border border-surface-600">
                  <p className="text-2xl font-bold text-white">{totalTeams}</p>
                  <p className="text-xs text-gray-400 uppercase tracking-wide">
                    Teams
                  </p>
                </div>
                <div className="p-4 rounded-lg bg-linear-to-br from-success-500/20 to-emerald-500/10 border border-success-500/30">
                  <p className="text-2xl font-bold text-success-500">
                    {connectedTeams}
                  </p>
                  <p className="text-xs text-success-500/80 uppercase tracking-wide">
                    Connected
                  </p>
                </div>
              </div>

              <div className="p-4 rounded-lg bg-linear-to-br from-primary-500/10 to-emerald-500/10 border border-primary-500/20">
                <p className="text-xs text-primary-400 mb-1 uppercase tracking-wide">
                  Start Time
                </p>
                <p className="text-gray-200">
                  {contest.startTime
                    ? formatDate(contest.startTime)
                    : "Delayed"}
                </p>
              </div>
              <div className="p-4 rounded-lg bg-linear-to-br from-orange-500/10 to-rose-500/10 border border-orange-500/20">
                <p className="text-xs text-orange-400 mb-1 uppercase tracking-wide">
                  End Time
                </p>
                <p className="text-gray-200">{formatDate(contest.endTime)}</p>
              </div>

              <div className="p-4 rounded-lg bg-linear-to-br from-violet-500/10 to-purple-500/10 border border-violet-500/20">
                <p className="text-xs text-violet-400 mb-2 uppercase tracking-wide">
                  Map
                </p>
                <select
                  value={
                    (setMapMutation.isPending
                      ? setMapMutation.variables
                      : contest.mapId) ?? ""
                  }
                  onChange={(e) => {
                    if (e.target.value !== "") {
                      setMapMutation.mutate(Number(e.target.value));
                    }
                  }}
                  disabled={setMapMutation.isPending}
                  className="w-full bg-surface-700 border border-surface-500 rounded-lg px-3 py-2 text-gray-200 focus:outline-none focus:ring-2 focus:ring-violet-500 text-sm"
                >
                  <option value="">No map selected</option>
                  {maps.map((map) => (
                    <option key={map.id} value={map.id}>
                      {map.name}
                    </option>
                  ))}
                </select>
                {setMapMutation.isError && (
                  <p className="text-danger-500 text-sm mt-2">
                    Failed to set map
                  </p>
                )}
                {setMapMutation.isPending && (
                  <p className="text-gray-400 text-sm mt-2">Saving...</p>
                )}
              </div>

              <div className="p-4 rounded-lg bg-linear-to-br from-sky-500/10 to-cyan-500/10 border border-sky-500/20">
                <p className="text-xs text-sky-400 mb-2 uppercase tracking-wide">
                  Wallpaper
                </p>
                <select
                  value={selectedWallpaperId ?? ""}
                  onChange={(e) =>
                    setWallpaperMutation.mutate(
                      e.target.value === ""
                        ? undefined
                        : Number(e.target.value),
                    )
                  }
                  disabled={setWallpaperMutation.isPending}
                  className="w-full bg-surface-700 border border-surface-500 rounded-lg px-3 py-2 text-gray-200 focus:outline-none focus:ring-2 focus:ring-sky-500 text-sm"
                >
                  <option value="">None (use default)</option>
                  {wallpapers.map((wallpaper) => (
                    <option key={wallpaper.id} value={wallpaper.id}>
                      {wallpaper.name}
                    </option>
                  ))}
                </select>
                {setWallpaperMutation.isError && (
                  <p className="text-danger-500 text-sm mt-2">
                    Failed to set wallpaper
                  </p>
                )}
                {setWallpaperMutation.isPending && (
                  <p className="text-gray-400 text-sm mt-2">Saving...</p>
                )}
                {!assignedWallpaper && defaultWallpaper && (
                  <p className="text-gray-400 text-sm mt-2">
                    Using the system default. Manage it under{" "}
                    <Link
                      to="/wallpapers"
                      className="text-sky-400 hover:text-sky-300 underline"
                    >
                      Wallpapers
                    </Link>
                    .
                  </p>
                )}
                {shownWallpaper && (
                  <button
                    onClick={() => setShowPreview(!showPreview)}
                    className={`w-full mt-3 px-3 py-2 rounded text-sm transition-colors ${
                      showPreview
                        ? "bg-primary-500 text-white"
                        : "bg-surface-600 text-gray-300 hover:bg-surface-500"
                    }`}
                  >
                    Preview team name
                  </button>
                )}
              </div>
            </div>
          ) : (
            <p className="text-gray-400">No active contest</p>
          )}

        </div>
      </aside>
      <div className="flex-1 bg-surface-700 flex items-center justify-center relative overflow-hidden">
        {isLoading || wallpapersLoading ? (
          <p className="text-gray-400 text-lg">Loading wallpaper...</p>
        ) : shownWallpaper ? (
          <div className="absolute inset-0 flex items-center justify-center p-4">
            <div className="relative inline-flex items-center justify-center max-w-full max-h-full">
              <WallpaperMedia
                wallpaper={shownWallpaper}
                className="max-w-full max-h-full w-auto h-auto object-contain block"
              />
              {showPreview && (
                <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                  <span
                    className="text-4xl font-bold drop-shadow-lg text-center px-4"
                    style={{ color: shownWallpaper.textColor }}
                  >
                    Team Name
                  </span>
                </div>
              )}
            </div>
            {!assignedWallpaper && (
              <div className="absolute top-4 left-4 px-3 py-1.5 rounded-lg bg-surface-900/80 backdrop-blur text-sm text-gray-300">
                Using the system default ·{" "}
                <Link
                  to="/wallpapers"
                  className="text-sky-400 hover:text-sky-300 underline"
                >
                  Manage wallpapers
                </Link>
              </div>
            )}
          </div>
        ) : (
          <div className="text-center">
            <div className="w-24 h-24 mx-auto mb-4 rounded-full bg-surface-600 flex items-center justify-center">
              <svg
                className="w-12 h-12 text-gray-500"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={1.5}
                  d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                />
              </svg>
            </div>
            <p className="text-gray-500 text-lg">No wallpaper set</p>
            <p className="text-gray-600 text-sm mt-1">
              Pick a wallpaper for this contest or set a system default under{" "}
              <Link
                to="/wallpapers"
                className="text-sky-400 hover:text-sky-300 underline"
              >
                Wallpapers
              </Link>
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
