import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef, useState, useEffect } from "react";
import { adminClient } from "../lib/client";
import { timestampDate } from "@bufbuild/protobuf/wkt";
import type { Timestamp } from "@bufbuild/protobuf/wkt";
import type { Station } from "@client/v1/admin/admin_pb";

export function ContestPage() {
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedMapId, setSelectedMapId] = useState<number | null>(null);
  const [textColor, setTextColor] = useState<string>("#ffffff");
  const [showPreview, setShowPreview] = useState<boolean>(false);

  const { data: contest, isLoading } = useQuery({
    queryKey: ["contest"],
    queryFn: () => adminClient.getNextContest(),
  });

  const { data: wallpaper, isLoading: wallpaperLoading } = useQuery({
    queryKey: ["wallpaper"],
    queryFn: () => adminClient.getWallpaper(),
    staleTime: 10 * 60 * 1000, // 10 minutes - wallpaper rarely changes
    gcTime: 30 * 60 * 1000, // 30 minutes - keep in cache longer
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

  const uploadMutation = useMutation({
    mutationFn: (imageData: Uint8Array) =>
      adminClient.setWallpaper(contest?.id ?? "", imageData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallpaper"] });
    },
  });

  const textColorMutation = useMutation({
    mutationFn: (color: string) =>
      adminClient.setWallpaperTextColor(contest?.id ?? "", color),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallpaper"] });
    },
  });

  // Sync text color with server state
  useEffect(() => {
    if (wallpaper?.color) {
      setTextColor(wallpaper.color);
    }
  }, [wallpaper?.color]);

  // Sync local state with server state
  useEffect(() => {
    if (contest?.mapId !== undefined) {
      setSelectedMapId(contest.mapId);
    }
  }, [contest?.mapId]);

  const setMapMutation = useMutation({
    mutationFn: (mapId: number) => adminClient.setMap(contest?.id ?? "", mapId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["contest"] });
    },
    onError: (error) => {
      console.error("Failed to set map:", error);
      // Reset to server value on error
      setSelectedMapId(contest?.mapId ?? null);
    },
  });

  const formatDate = (timestamp: Timestamp | undefined) => {
    if (!timestamp) return "N/A";
    const date = timestampDate(timestamp);
    return new Intl.DateTimeFormat("en-US", {
      dateStyle: "long",
      timeStyle: "short",
      timeZone: "UTC",
    }).format(date);
  };

  const isStationConnected = (station: Station) => {
    if (!station.diconnectedAt) return true;
    if (!station.connectedAt) return false;
    return (
      timestampDate(station.connectedAt) > timestampDate(station.diconnectedAt)
    );
  };

  const teams = teamsData?.teams ?? [];
  const stations = stationsData?.stations ?? [];
  const maps = mapsData?.maps ?? [];
  const totalTeams = teams.length;

  const connectedTeams = teams.filter((team) => {
    if (!team.ip) return false;
    const station = stations.find((s) => s.ip === team.ip);
    return station && isStationConnected(station);
  }).length;

  const wallpaperUrl = wallpaper?.imageData?.length
    ? URL.createObjectURL(new Blob([new Uint8Array(wallpaper.imageData)]))
    : null;

  const handleFileSelect = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    const arrayBuffer = await file.arrayBuffer();
    uploadMutation.mutate(new Uint8Array(arrayBuffer));
    e.target.value = "";
  };

  const openFilePicker = () => {
    fileInputRef.current?.click();
  };

  const handleMapChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const value = e.target.value;
    if (value === "") {
      setSelectedMapId(null);
      return;
    }
    const mapId = parseInt(value, 10);
    if (!isNaN(mapId)) {
      setSelectedMapId(mapId); // Optimistic update
      setMapMutation.mutate(mapId);
    }
  };

  const handleTextColorChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const color = e.target.value;
    setTextColor(color);
  };

  const handleTextColorSave = () => {
    textColorMutation.mutate(textColor);
  };

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
                <p className="text-gray-200">{formatDate(contest.startTime)}</p>
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
                  value={selectedMapId ?? ""}
                  onChange={handleMapChange}
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
            </div>
          ) : (
            <p className="text-gray-400">No active contest</p>
          )}

          <div className="mt-6 pt-6 border-t border-surface-600">
            <input
              ref={fileInputRef}
              type="file"
              accept="image/png,image/jpeg,image/gif,image/webp,image/bmp,image/svg+xml"
              onChange={handleFileSelect}
              className="hidden"
            />
            <button
              onClick={openFilePicker}
              disabled={!contest || uploadMutation.isPending}
              className="w-full px-4 py-3 bg-linear-to-r from-primary-500 to-primary-600 hover:from-primary-600 hover:to-primary-700 disabled:from-surface-600 disabled:to-surface-600 disabled:text-gray-500 text-white rounded-lg transition-all font-medium flex items-center justify-center gap-2"
            >
              {uploadMutation.isPending ? (
                <>
                  <svg
                    className="w-5 h-5 animate-spin"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      className="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      strokeWidth="4"
                    />
                    <path
                      className="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    />
                  </svg>
                  Uploading...
                </>
              ) : (
                <>
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
                      d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                    />
                  </svg>
                  Upload Wallpaper
                </>
              )}
            </button>
            {uploadMutation.isError && (
              <p className="text-danger-500 text-sm mt-2">
                Failed to upload image
              </p>
            )}

            {wallpaperUrl && (
              <div className="mt-4 p-4 rounded-lg bg-surface-700 border border-surface-600">
                <p className="text-xs text-gray-400 mb-2 uppercase tracking-wide">
                  Text Color
                </p>
                <div className="flex items-center gap-3">
                  <input
                    type="color"
                    value={textColor}
                    onChange={handleTextColorChange}
                    className="w-10 h-10 rounded cursor-pointer border-0 bg-transparent"
                  />
                  <input
                    type="text"
                    value={textColor}
                    onChange={handleTextColorChange}
                    className="flex-1 bg-surface-600 border border-surface-500 rounded px-3 py-2 text-gray-200 text-sm font-mono"
                    placeholder="#ffffff"
                  />
                </div>
                <div className="flex items-center gap-2 mt-3">
                  <button
                    onClick={handleTextColorSave}
                    disabled={
                      textColorMutation.isPending ||
                      textColor === wallpaper?.color
                    }
                    className="flex-1 px-3 py-2 bg-primary-500 hover:bg-primary-600 disabled:bg-surface-600 disabled:text-gray-500 text-white rounded text-sm transition-colors"
                  >
                    {textColorMutation.isPending ? "Saving..." : "Save Color"}
                  </button>
                  <button
                    onClick={() => setShowPreview(!showPreview)}
                    className={`px-3 py-2 rounded text-sm transition-colors ${
                      showPreview
                        ? "bg-primary-500 text-white"
                        : "bg-surface-600 text-gray-300 hover:bg-surface-500"
                    }`}
                  >
                    Preview
                  </button>
                </div>
                {textColorMutation.isError && (
                  <p className="text-danger-500 text-sm mt-2">
                    Failed to save color
                  </p>
                )}
              </div>
            )}
          </div>
        </div>
      </aside>
      <div className="flex-1 bg-surface-700 flex items-center justify-center relative overflow-hidden">
        {wallpaperLoading ? (
          <p className="text-gray-400 text-lg">Loading wallpaper...</p>
        ) : wallpaperUrl ? (
          <div className="absolute inset-0 flex items-center justify-center p-4">
            <div className="relative inline-flex items-center justify-center max-w-full max-h-full">
              <img
                src={wallpaperUrl}
                alt="Contest wallpaper"
                className="max-w-full max-h-full w-auto h-auto object-contain block"
              />
              {showPreview && (
                <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
                  <span
                    className="text-4xl font-bold drop-shadow-lg text-center px-4"
                    style={{ color: textColor }}
                  >
                    Team Name
                  </span>
                </div>
              )}
            </div>
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
              Upload an image to set the contest wallpaper
            </p>
          </div>
        )}
      </div>
    </div>
  );
}
