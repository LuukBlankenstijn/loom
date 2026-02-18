package persistence

import (
	"context"
	"errors"
	"log/slog"
	"time"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/wallpaper"
)

type EntWallpaperRepository struct {
	client *ent.Client
}

func NewEntWallpaperRepository(client *ent.Client) *EntWallpaperRepository {
	return &EntWallpaperRepository{client: client}
}

func (r *EntWallpaperRepository) SetWallpaperData(
	ctx context.Context,
	contestId string,
	imageData []byte,
	mimeType string,
) error {
	err := r.client.Wallpaper.
		Create().
		SetContestID(contestId).
		SetImageData(imageData).
		SetMimeType(mimeType).
		OnConflictColumns(
			wallpaper.FieldContestID,
		).
		UpdateNewValues().
		Exec(ctx)
	if err != nil {
		slog.Error(
			"failed to set wallpaper for contest",
			slog.String("contestId", contestId),
			slog.Any("error", err),
		)
		return errors.New("failed to set wallpaper")
	}
	return nil
}

func (r *EntWallpaperRepository) DeleteWallpaper(ctx context.Context, contestId string) error {
	_, err := r.client.Wallpaper.Delete().Where(wallpaper.ContestID(contestId)).Exec(ctx)
	if err != nil {
		slog.Error(
			"failed to remove wallpaper for contest",
			"contestId",
			contestId,
			"error",
			err,
		)
		return errors.New("failed to remove wallpaper")
	}
	return nil
}

func (r *EntWallpaperRepository) SetWallpaperTextColor(
	ctx context.Context,
	contestId string,
	color string,
) error {
	err := r.client.Wallpaper.Update().
		Where(wallpaper.ContestID(contestId)).
		SetColor(color).
		Exec(ctx)
	if err != nil {
		slog.Error("Unexpected error setting wallpaper text color", "error", err)
		return errors.New("Unexpected error")
	}
	return nil
}

func (r *EntWallpaperRepository) GetWallpaper(
	ctx context.Context,
	contestId string,
) (*domain.Wallpaper, error) {
	wallpaper, err := r.client.Wallpaper.Query().Where(wallpaper.ContestID(contestId)).Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil
		}
		slog.Error("Unexpected error getting wallpaper", "error", err)
		return nil, errors.New("Unexpected error")
	}
	return &domain.Wallpaper{
		ContestId: wallpaper.ContestID,
		MimeType:  wallpaper.MimeType,
		UpdatedAt: wallpaper.UpdatedAt,
		Data:      wallpaper.ImageData,
		TextColor: wallpaper.Color,
	}, nil
}

func (r *EntWallpaperRepository) GetLastUpdated(
	ctx context.Context,
	contestId string,
) (time.Time, error) {
	w, err := r.client.Wallpaper.Query().
		Where(wallpaper.ContestID(contestId)).
		Select(wallpaper.FieldUpdatedAt).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return time.Now(), nil
		}
		slog.Error("Unexpected error getting wallpaper", "error", err)
		return time.Now(), errors.New("unexpected error")
	}

	return w.UpdatedAt, nil
}
