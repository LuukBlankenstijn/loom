package persistence

import (
	"context"
	"errors"
	"log/slog"

	"entgo.io/ent/dialect/sql"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/wallpaper"
)

type EntWallpaperRepository struct {
	client *ent.Client
}

func NewEntWallpaperRepository(client *ent.Client) *EntWallpaperRepository {
	return &EntWallpaperRepository{client: client}
}

func (r *EntWallpaperRepository) SetWallpaper(ctx context.Context, contestId string, imageData *[]byte) error {
	if imageData == nil {
		_, err := r.client.Wallpaper.Delete().Where(wallpaper.ContestID(contestId)).Exec(ctx)
		if err != nil {
			slog.Error("failed to remove wallpaper for contest", "contestId", contestId, "error", err)
			return errors.New("failed to remove wallpaper")
		}
		return nil
	}

	err := r.client.Wallpaper.
		Create().
		SetContestID(contestId).
		SetImageData(*imageData).
		OnConflict(
			sql.ConflictColumns(wallpaper.FieldContestID),
		).
		UpdateNewValues().
		Exec(ctx)
	if err != nil {
		slog.Error("failed to create new wallpaper for contest", "contestId", contestId, "error", err)
		return errors.New("failed to set wallpaper")
	}
	return nil
}

func (r *EntWallpaperRepository) GetWallpaper(ctx context.Context, contestId string) (*[]byte, error) {
	wallpaper, err := r.client.Wallpaper.Query().Where(wallpaper.ContestID(contestId)).Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil
		}
		slog.Error("Unexpected error getting wallpaper", "error", err)
		return nil, errors.New("Unexpected error")
	}
	return &wallpaper.ImageData, nil
}
