package domain

import "context"

type Wallpaper struct {
	ContestId string
	Data      []byte
}

type WallpaperRepository interface {
	// Sets the wallpaper for some contest
	SetWallpaper(ctx context.Context, contestId string, wallpaper *[]byte) error

	// Gets the wallpaper for some contest
	GetWallpaper(ctx context.Context, contestId string) (*[]byte, error)
}
