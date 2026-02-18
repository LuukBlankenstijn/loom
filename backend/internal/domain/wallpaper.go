package domain

import (
	"context"
	"time"
)

type Wallpaper struct {
	ContestId string
	MimeType  string
	UpdatedAt time.Time
	TextColor string
	Data      []byte
}

type WallpaperRepository interface {
	// Sets the wallpaper data bytes for some contest
	SetWallpaperData(ctx context.Context, contestId string, wallpaper []byte, mimeType string) error

	// Sets the color of the text on the wallpaper, errors if color is not a valid hex string
	SetWallpaperTextColor(ctx context.Context, contestId string, color string) error

	// Deletes the wallpaper for some contest
	DeleteWallpaper(ctx context.Context, contestId string) error

	// Gets the wallpaper for some contest
	GetWallpaper(ctx context.Context, contestId string) (*Wallpaper, error)

	// Gets the last time the wallpaper for some contest was updated to avoid loading the entire wallpaper in memory
	GetLastUpdated(ctx context.Context, contestId string) (time.Time, error)
}
