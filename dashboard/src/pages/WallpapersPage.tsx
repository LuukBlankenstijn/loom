import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import {
  WallpaperKind,
  type VideoStatus,
  type Wallpaper,
} from "@client/v1/admin/wallpaper_pb";
import { adminClient } from "../lib/client";
import { WallpaperMedia } from "../components/WallpaperMedia";

const MAX_UPLOAD_BYTES = 10 * 1024 * 1024;
const ACCEPTED_IMAGE_TYPES = [
  "image/png",
  "image/jpeg",
  "image/webp",
  "image/gif",
];

function useInvalidateWallpapers() {
  const queryClient = useQueryClient();
  return () =>
    Promise.all([
      queryClient.invalidateQueries({ queryKey: ["wallpapers"] }),
      queryClient.invalidateQueries({ queryKey: ["contest"] }),
    ]);
}

export function WallpapersPage() {
  const invalidateWallpapers = useInvalidateWallpapers();

  const { data, isLoading } = useQuery({
    queryKey: ["wallpapers"],
    queryFn: () => adminClient.listWallpapers(),
    refetchInterval: (query) =>
      query.state.data?.wallpapers.some(
        (wallpaper) => wallpaper.kind === WallpaperKind.YOUTUBE,
      )
        ? 5000
        : false,
  });

  const setDefaultMutation = useMutation({
    mutationFn: (wallpaperId?: number) =>
      adminClient.setDefaultWallpaper(wallpaperId),
    onSuccess: invalidateWallpapers,
  });

  const wallpapers = data?.wallpapers ?? [];
  const defaultWallpaper = wallpapers.find((w) => w.isDefault);
  const selectedDefaultId = setDefaultMutation.isPending
    ? setDefaultMutation.variables
    : defaultWallpaper?.id;

  return (
    <div className="max-w-5xl mx-auto px-6 py-10">
      <div className="flex items-center gap-3 mb-8">
        <div className="w-2 h-8 bg-linear-to-b from-sky-400 to-sky-600 rounded-full" />
        <h1 className="text-3xl font-semibold text-white">Wallpapers</h1>
        <span className="ml-auto px-3 py-1 bg-sky-500/20 text-sky-400 rounded-full text-sm">
          {wallpapers.length} total
        </span>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4 mb-8">
        <div className="p-5 bg-surface-800 rounded-xl border border-surface-600">
          <h3 className="text-sm font-medium text-gray-300 uppercase tracking-wide">
            Default wallpaper
          </h3>
          <p className="text-sm text-gray-400 mt-2">
            Stations show this wallpaper whenever the contest has no wallpaper
            of its own.
          </p>
          <select
            value={selectedDefaultId ?? ""}
            onChange={(e) =>
              setDefaultMutation.mutate(
                e.target.value === "" ? undefined : Number(e.target.value),
              )
            }
            disabled={isLoading || setDefaultMutation.isPending}
            className="w-full mt-4 bg-surface-700 border border-surface-500 rounded-lg px-3 py-2 text-gray-200 focus:outline-none focus:ring-2 focus:ring-sky-500 text-sm"
          >
            <option value="">None</option>
            {wallpapers.map((wallpaper) => (
              <option key={wallpaper.id} value={wallpaper.id}>
                {wallpaper.name}
              </option>
            ))}
          </select>
          {setDefaultMutation.isError && (
            <p className="text-danger-500 text-sm mt-2">
              Failed to set default wallpaper
            </p>
          )}
          {setDefaultMutation.isPending && (
            <p className="text-gray-400 text-sm mt-2">Saving...</p>
          )}
        </div>

        <AddWallpaperForm />
      </div>

      {isLoading ? (
        <div className="text-gray-400">Loading...</div>
      ) : wallpapers.length === 0 ? (
        <div className="text-center py-20">
          <div className="w-20 h-20 mx-auto mb-4 rounded-full bg-surface-700 flex items-center justify-center">
            <svg
              className="w-10 h-10 text-gray-500"
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
          <p className="text-gray-400 text-lg">No wallpapers yet</p>
          <p className="text-gray-500 text-sm mt-1">
            Upload an image, or add an image URL or YouTube video to build the
            library
          </p>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          {wallpapers.map((wallpaper) => (
            <WallpaperCard key={wallpaper.id} wallpaper={wallpaper} />
          ))}
        </div>
      )}
    </div>
  );
}

type SourceMode = "upload" | "imageUrl" | "youtube";

const SOURCE_MODES: [SourceMode, string][] = [
  ["upload", "Upload image"],
  ["imageUrl", "Image URL"],
  ["youtube", "YouTube"],
];

function AddWallpaperForm() {
  const invalidateWallpapers = useInvalidateWallpapers();
  const [name, setName] = useState("");
  const [mode, setMode] = useState<SourceMode>("upload");
  const [file, setFile] = useState<File | null>(null);
  const [fileError, setFileError] = useState<string | null>(null);
  const [url, setUrl] = useState("");
  const [fileInputKey, setFileInputKey] = useState(0);

  const createMutation = useMutation({
    mutationFn: async () => {
      const trimmedName = name.trim();
      if (mode !== "upload") {
        return adminClient.createWallpaperFromUrl(
          trimmedName,
          url.trim(),
          mode === "youtube" ? WallpaperKind.YOUTUBE : WallpaperKind.IMAGE,
        );
      }
      if (!file) throw new Error("No file selected");
      const imageData = new Uint8Array(await file.arrayBuffer());
      return adminClient.createWallpaperFromFile(trimmedName, imageData);
    },
    onSuccess: () => {
      setName("");
      setFile(null);
      setUrl("");
      setFileInputKey((key) => key + 1);
      return invalidateWallpapers();
    },
  });

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const selected = e.target.files?.[0] ?? null;
    setFile(null);
    setFileError(null);
    if (!selected) return;

    if (!ACCEPTED_IMAGE_TYPES.includes(selected.type)) {
      setFileError("Only PNG, JPEG, WebP and GIF images are supported.");
      e.target.value = "";
      return;
    }
    if (selected.size > MAX_UPLOAD_BYTES) {
      const sizeMiB = (selected.size / 1024 / 1024).toFixed(1);
      setFileError(
        `This file is ${sizeMiB} MiB; uploads are limited to 10 MiB. Host larger files elsewhere and add them by URL.`,
      );
      e.target.value = "";
      return;
    }

    setFile(selected);
    if (!name.trim()) {
      setName(selected.name.replace(/\.[^.]+$/, ""));
    }
  };

  const canSubmit =
    name.trim() !== "" &&
    (mode === "upload" ? file !== null : url.trim() !== "") &&
    !createMutation.isPending;

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        if (canSubmit) createMutation.mutate();
      }}
      className="p-5 bg-surface-800 rounded-xl border border-surface-600 space-y-3"
    >
      <h3 className="text-sm font-medium text-gray-300 uppercase tracking-wide">
        Add wallpaper
      </h3>
      <input
        type="text"
        value={name}
        onChange={(e) => setName(e.target.value)}
        placeholder="Name"
        className="w-full bg-surface-700 border border-surface-500 rounded-lg px-3 py-2 text-gray-200 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-sky-500"
      />
      <div className="flex gap-1 p-1 bg-surface-700 rounded-lg">
        {SOURCE_MODES.map(([value, label]) => (
          <button
            key={value}
            type="button"
            onClick={() => {
              setMode(value);
              setFile(null);
              setFileError(null);
              setFileInputKey((key) => key + 1);
            }}
            className={`flex-1 px-3 py-1.5 rounded-md text-sm transition-colors ${
              mode === value
                ? "bg-sky-500 text-white"
                : "text-gray-400 hover:text-white hover:bg-surface-600"
            }`}
          >
            {label}
          </button>
        ))}
      </div>

      {mode === "upload" ? (
        <div>
          <input
            key={fileInputKey}
            type="file"
            accept={ACCEPTED_IMAGE_TYPES.join(",")}
            onChange={handleFileChange}
            className="w-full text-sm text-gray-400 file:mr-3 file:px-3 file:py-2 file:rounded-lg file:border-0 file:bg-surface-600 file:text-gray-200 hover:file:bg-surface-500 file:cursor-pointer"
          />
          <p className="text-xs text-gray-500 mt-1">
            PNG, JPEG, WebP or GIF, up to 10 MiB.
          </p>
          {fileError && (
            <p className="text-danger-500 text-sm mt-2">{fileError}</p>
          )}
        </div>
      ) : (
        <div>
          <input
            type="url"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
            placeholder={
              mode === "youtube"
                ? "https://www.youtube.com/watch?v=…"
                : "https://..."
            }
            className="w-full bg-surface-700 border border-surface-500 rounded-lg px-3 py-2 text-gray-200 placeholder-gray-500 text-sm focus:outline-none focus:ring-2 focus:ring-sky-500"
          />
          {mode === "youtube" && (
            <p className="text-xs text-gray-500 mt-1">
              The server streams the video to stations while it is the current
              wallpaper.
            </p>
          )}
        </div>
      )}

      <button
        type="submit"
        disabled={!canSubmit}
        className="w-full px-4 py-2 bg-sky-500 hover:bg-sky-600 disabled:bg-surface-600 disabled:text-gray-500 text-white rounded-lg text-sm font-medium transition-colors"
      >
        {createMutation.isPending ? "Adding..." : "Add wallpaper"}
      </button>
      {createMutation.isError && (
        <p className="text-danger-500 text-sm">
          Failed to add wallpaper: {createMutation.error.message}
        </p>
      )}
    </form>
  );
}

function WallpaperCard({ wallpaper }: { wallpaper: Wallpaper }) {
  const invalidateWallpapers = useInvalidateWallpapers();
  const [editedName, setEditedName] = useState<string | null>(null);
  const [editedColor, setEditedColor] = useState<string | null>(null);
  const [showPreview, setShowPreview] = useState(false);

  const renameMutation = useMutation({
    mutationFn: (name: string) =>
      adminClient.updateWallpaper(wallpaper.id, { name }),
    onSuccess: () => {
      setEditedName(null);
      return invalidateWallpapers();
    },
  });

  const textColorMutation = useMutation({
    mutationFn: (textColor: string) =>
      adminClient.updateWallpaper(wallpaper.id, { textColor }),
    onSuccess: () => {
      setEditedColor(null);
      return invalidateWallpapers();
    },
  });

  const audioMutation = useMutation({
    mutationFn: (audio: boolean) =>
      adminClient.updateWallpaper(wallpaper.id, { audio }),
    onSuccess: invalidateWallpapers,
  });

  const deleteMutation = useMutation({
    mutationFn: () => adminClient.deleteWallpaper(wallpaper.id),
    onSuccess: invalidateWallpapers,
  });

  const textColor = editedColor ?? wallpaper.textColor;
  const isUpload = wallpaper.source.case === "mimeType";
  const isYoutube = wallpaper.kind === WallpaperKind.YOUTUBE;
  const usage = [
    ...(wallpaper.isDefault ? ["Default"] : []),
    ...wallpaper.contestIds.map((id) => `Contest ${id}`),
  ];

  const commitRename = () => {
    if (renameMutation.isPending) return;
    const name = editedName?.trim();
    if (!name || name === wallpaper.name) {
      setEditedName(null);
      return;
    }
    renameMutation.mutate(name);
  };

  const handleDelete = () => {
    const message =
      usage.length > 0
        ? `"${wallpaper.name}" is in use (${usage.join(", ")}). Deleting it will unassign it. Delete anyway?`
        : `Delete "${wallpaper.name}"?`;
    if (window.confirm(message)) {
      deleteMutation.mutate();
    }
  };

  return (
    <div className="bg-surface-800 rounded-xl border border-surface-600 overflow-hidden hover:border-sky-500/50 transition-colors flex flex-col">
      <div className="relative aspect-video bg-surface-700">
        <WallpaperMedia
          wallpaper={wallpaper}
          className="absolute inset-0 w-full h-full object-cover"
        />
        {showPreview && (
          <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
            <span
              className="text-2xl font-bold drop-shadow-lg text-center px-4"
              style={{ color: textColor }}
            >
              Team Name
            </span>
          </div>
        )}
        <div className="absolute top-2 left-2 flex gap-1.5">
          {isYoutube ? (
            <span className="px-2 py-0.5 rounded-full text-xs font-medium backdrop-blur bg-rose-500/30 text-rose-200">
              YouTube
            </span>
          ) : (
            <>
              <span className="px-2 py-0.5 rounded-full text-xs font-medium backdrop-blur bg-sky-500/30 text-sky-200">
                Image
              </span>
              <span className="px-2 py-0.5 rounded-full text-xs font-medium bg-surface-900/60 text-gray-200 backdrop-blur">
                {isUpload ? "Upload" : "URL"}
              </span>
            </>
          )}
        </div>
        {wallpaper.video?.streaming && (
          <div className="absolute top-2 right-2 flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium bg-danger-500/80 text-white backdrop-blur">
            <span className="w-1.5 h-1.5 rounded-full bg-white animate-pulse" />
            Streaming
          </div>
        )}
      </div>

      <div className="p-4 flex flex-col gap-3 flex-1">
        <div className="flex items-start justify-between gap-2">
          {editedName !== null ? (
            <input
              type="text"
              value={editedName}
              onChange={(e) => setEditedName(e.target.value)}
              onBlur={commitRename}
              onKeyDown={(e) => {
                if (e.key === "Enter") commitRename();
                if (e.key === "Escape") setEditedName(null);
              }}
              disabled={renameMutation.isPending}
              autoFocus
              className="flex-1 min-w-0 bg-surface-700 border border-surface-500 rounded px-2 py-1 text-white text-lg focus:outline-none focus:ring-2 focus:ring-sky-500"
            />
          ) : (
            <button
              type="button"
              onClick={() => setEditedName(wallpaper.name)}
              title="Rename"
              className="flex-1 min-w-0 text-left text-lg font-medium text-white truncate hover:text-sky-300 transition-colors"
            >
              {wallpaper.name}
            </button>
          )}
          <span className="text-xs text-gray-500 font-mono pt-1.5">
            #{wallpaper.id}
          </span>
        </div>
        {renameMutation.isError && (
          <p className="text-danger-500 text-sm">Failed to rename</p>
        )}

        <p className="text-sm text-gray-400">
          {usage.length > 0 ? (
            <>
              Used by <span className="text-gray-200">{usage.join(", ")}</span>
            </>
          ) : (
            "Not in use"
          )}
        </p>
        {wallpaper.source.case === "url" && (
          <p
            className="text-xs text-gray-500 font-mono truncate"
            title={wallpaper.source.value}
          >
            {wallpaper.source.value}
          </p>
        )}
        {wallpaper.video && <VideoStatusPanel video={wallpaper.video} />}
        {isYoutube && (
          <label className="flex items-center justify-between gap-3 p-3 rounded-lg bg-surface-700 border border-surface-600 cursor-pointer">
            <span>
              <span className="block text-xs text-gray-400 uppercase tracking-wide">
                Sound
              </span>
              <span className="block text-sm text-gray-300">
                Stations play the video's audio
              </span>
            </span>
            <input
              type="checkbox"
              checked={wallpaper.audio}
              disabled={audioMutation.isPending}
              onChange={(e) => audioMutation.mutate(e.target.checked)}
              className="w-5 h-5 accent-sky-500 cursor-pointer"
            />
          </label>
        )}
        {audioMutation.isError && (
          <p className="text-danger-500 text-sm">Failed to change sound</p>
        )}

        <div className="p-3 rounded-lg bg-surface-700 border border-surface-600">
          <p className="text-xs text-gray-400 mb-2 uppercase tracking-wide">
            Text color
          </p>
          <div className="flex items-center gap-3">
            <input
              type="color"
              value={textColor}
              onChange={(e) => setEditedColor(e.target.value)}
              className="w-10 h-10 rounded cursor-pointer border-0 bg-transparent"
            />
            <input
              type="text"
              value={textColor}
              onChange={(e) => setEditedColor(e.target.value)}
              className="flex-1 min-w-0 bg-surface-600 border border-surface-500 rounded px-3 py-2 text-gray-200 text-sm font-mono"
              placeholder="#ffffff"
            />
          </div>
          <div className="flex items-center gap-2 mt-3">
            <button
              type="button"
              onClick={() => textColorMutation.mutate(textColor)}
              disabled={
                textColorMutation.isPending ||
                textColor === wallpaper.textColor
              }
              className="flex-1 px-3 py-2 bg-primary-500 hover:bg-primary-600 disabled:bg-surface-600 disabled:text-gray-500 text-white rounded text-sm transition-colors"
            >
              {textColorMutation.isPending ? "Saving..." : "Save color"}
            </button>
            <button
              type="button"
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

        <button
          type="button"
          onClick={handleDelete}
          disabled={deleteMutation.isPending}
          className="mt-auto w-full px-3 py-2 bg-surface-700 hover:bg-danger-500/20 disabled:text-gray-500 text-danger-500 rounded-lg text-sm font-medium transition-colors"
        >
          {deleteMutation.isPending ? "Deleting..." : "Delete"}
        </button>
        {deleteMutation.isError && (
          <p className="text-danger-500 text-sm">Failed to delete wallpaper</p>
        )}
      </div>
    </div>
  );
}

function VideoStatusPanel({ video }: { video: VideoStatus }) {
  return (
    <div className="p-3 rounded-lg bg-surface-700 border border-surface-600 space-y-2">
      <p className="text-xs text-gray-400 uppercase tracking-wide">Video</p>
      {video.streaming && (
        <p className="text-sm font-medium text-rose-300">
          Streaming to stations
        </p>
      )}
      {video.error && (
        <p
          className="text-xs text-danger-500 bg-danger-500/10 rounded px-2 py-1 font-mono line-clamp-3 break-words"
          title={video.error}
        >
          {video.error}
        </p>
      )}
      {!video.streaming && !video.error && (
        <p className="text-sm text-gray-500">
          Streams while this is the current wallpaper
        </p>
      )}
    </div>
  );
}
