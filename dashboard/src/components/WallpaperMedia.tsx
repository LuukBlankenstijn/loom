import {
  WallpaperKind,
  type Wallpaper,
} from "@client/v1/admin/wallpaper_pb";
import { wallpaperPreviewUrl } from "../lib/client";

export function WallpaperMedia({
  wallpaper,
  className,
}: {
  wallpaper: Wallpaper;
  className: string;
}) {
  const src = wallpaperPreviewUrl(wallpaper);

  if (wallpaper.kind === WallpaperKind.VIDEO) {
    return (
      <video
        src={src}
        muted
        loop
        autoPlay
        playsInline
        className={className}
      />
    );
  }

  return <img src={src} alt={wallpaper.name} className={className} />;
}
