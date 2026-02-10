import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useRef } from "react";
import { adminClient } from "../lib/client";
import { timestampDate } from "@bufbuild/protobuf/wkt";
import type { Timestamp } from "@bufbuild/protobuf/wkt";
import type { Station } from "@client/admin/v1/admin_pb";

export function ContestPage() {
  const queryClient = useQueryClient();
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { data: contest, isLoading } = useQuery({
    queryKey: ["contest"],
    queryFn: () => adminClient.getNextContest(),
  });

  const { data: wallpaper, isLoading: wallpaperLoading } = useQuery({
    queryKey: ["wallpaper"],
    queryFn: () => adminClient.getWallpaper(),
    staleTime: 5 * 60 * 1000, // 5 minutes
  });

  const { data: teamsData } = useQuery({
    queryKey: ["teams"],
    queryFn: () => adminClient.getActiveTeams(),
  });

  const { data: stationsData } = useQuery({
    queryKey: ["stations"],
    queryFn: () => adminClient.getStations(),
  });

  const uploadMutation = useMutation({
    mutationFn: (imageData: Uint8Array) =>
      adminClient.setWallpaper(contest?.id ?? "", imageData),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["wallpaper"] });
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
    return timestampDate(station.connectedAt) > timestampDate(station.diconnectedAt);
  };

  const teams = teamsData?.teams ?? [];
  const stations = stationsData?.stations ?? [];
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

  return (
    <div className="h-full flex">
      <aside className="w-80 bg-surface-800 border-r border-surface-600 p-6 flex flex-col">
        <h2 className="text-2xl font-semibold text-white mb-6">Contest Details</h2>
        {isLoading ? (
          <div className="text-gray-400">Loading...</div>
        ) : contest ? (
          <div className="space-y-6 flex-1">
            <div>
              <h3 className="text-xl font-medium text-white">{contest.name}</h3>
            </div>

            <div className="grid grid-cols-2 gap-3">
              <div className="p-4 rounded-lg bg-surface-700 border border-surface-600">
                <p className="text-2xl font-bold text-white">{totalTeams}</p>
                <p className="text-xs text-gray-400 uppercase tracking-wide">Teams</p>
              </div>
              <div className="p-4 rounded-lg bg-gradient-to-br from-success-500/20 to-emerald-500/10 border border-success-500/30">
                <p className="text-2xl font-bold text-success-500">{connectedTeams}</p>
                <p className="text-xs text-success-500/80 uppercase tracking-wide">Connected</p>
              </div>
            </div>

            <div className="p-4 rounded-lg bg-gradient-to-br from-primary-500/10 to-emerald-500/10 border border-primary-500/20">
              <p className="text-xs text-primary-400 mb-1 uppercase tracking-wide">Start Time</p>
              <p className="text-gray-200">{formatDate(contest.startTime)}</p>
            </div>
            <div className="p-4 rounded-lg bg-gradient-to-br from-orange-500/10 to-rose-500/10 border border-orange-500/20">
              <p className="text-xs text-orange-400 mb-1 uppercase tracking-wide">End Time</p>
              <p className="text-gray-200">{formatDate(contest.endTime)}</p>
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
            className="w-full px-4 py-3 bg-gradient-to-r from-primary-500 to-primary-600 hover:from-primary-600 hover:to-primary-700 disabled:from-surface-600 disabled:to-surface-600 disabled:text-gray-500 text-white rounded-lg transition-all font-medium flex items-center justify-center gap-2"
          >
            {uploadMutation.isPending ? (
              <>
                <svg className="w-5 h-5 animate-spin" fill="none" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
                Uploading...
              </>
            ) : (
              <>
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                </svg>
                Upload Wallpaper
              </>
            )}
          </button>
          {uploadMutation.isError && (
            <p className="text-danger-500 text-sm mt-2">Failed to upload image</p>
          )}
        </div>
      </aside>
      <div className="flex-1 bg-surface-700 flex items-center justify-center relative">
        {wallpaperLoading ? (
          <p className="text-gray-400 text-lg">Loading wallpaper...</p>
        ) : wallpaperUrl ? (
          <img
            src={wallpaperUrl}
            alt="Contest wallpaper"
            className="max-w-full max-h-full object-contain"
          />
        ) : (
          <div className="text-center">
            <div className="w-24 h-24 mx-auto mb-4 rounded-full bg-surface-600 flex items-center justify-center">
              <svg className="w-12 h-12 text-gray-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
              </svg>
            </div>
            <p className="text-gray-500 text-lg">No wallpaper set</p>
            <p className="text-gray-600 text-sm mt-1">Upload an image to set the contest wallpaper</p>
          </div>
        )}
      </div>
    </div>
  );
}
