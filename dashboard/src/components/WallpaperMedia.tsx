import type { Wallpaper } from "@client/v1/admin/wallpaper_pb";
import { wallpaperPreviewUrl } from "../lib/client";

export function WallpaperMedia({
  wallpaper,
  className,
}: {
  wallpaper: Wallpaper;
  className: string;
}) {
  return (
    <img
      src={wallpaperPreviewUrl(wallpaper)}
      alt={wallpaper.name}
      className={className}
    />
  );
}
