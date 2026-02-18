package admin

import (
	"bytes"
	"context"
	"errors"
	"fmt"
	"image"
	_ "image/gif"
	_ "image/jpeg"
	_ "image/png"

	"connectrpc.com/connect"
	adminv1 "github.com/LuukBlankenstijn/loom/gen/go/admin/v1"
	_ "golang.org/x/image/bmp"
	_ "golang.org/x/image/tiff"
	_ "golang.org/x/image/webp"
	"google.golang.org/protobuf/types/known/emptypb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// Gets the contest that is currently active, or the contest that will start next
func (a *adminHandler) GetNextContest(
	ctx context.Context,
	empty *emptypb.Empty,
) (*adminv1.Contest, error) {
	nextContest, err := a.contestRepo.GetNextContest(ctx)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, errors.New("failed to get next contest"))
	}
	if nextContest == nil {
		return nil, connect.NewError(connect.CodeNotFound, errors.New("next contest not found"))
	}

	var mapId *int32
	m, err := a.mapRepo.GetByContest(ctx, nextContest.Id)
	if err == nil && m != nil {
		val := int32(m.Id)
		mapId = &val
	}
	return &adminv1.Contest{
		Id:        nextContest.Id,
		Name:      nextContest.Name,
		StartTime: timestamppb.New(nextContest.StartTime),
		EndTime:   timestamppb.New(nextContest.EndTime),
		MapId:     mapId,
	}, nil
}

// Sets the wallpaper for some contest
func (a *adminHandler) SetWallpaper(
	ctx context.Context,
	request *adminv1.UploadWallpaperRequest,
) (*emptypb.Empty, error) {
	if len(request.ImageData) > 0 {
		mimeType, err := validateImageFormat(request.ImageData)
		if err != nil {
			return nil, connect.NewError(connect.CodeInvalidArgument, err)
		}
		if err := a.wallpaperRepo.SetWallpaperData(ctx, request.ContestId, request.ImageData, mimeType); err != nil {
			return nil, connect.NewError(connect.CodeInternal, err)
		}
		return &emptypb.Empty{}, nil
	}
	if err := a.wallpaperRepo.DeleteWallpaper(ctx, request.ContestId); err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	return &emptypb.Empty{}, nil
}

func (a *adminHandler) SetWallpaperTextColor(
	ctx context.Context,
	request *adminv1.SetWallpaperTextColorRequest,
) (*emptypb.Empty, error) {
	if err := a.wallpaperRepo.SetWallpaperTextColor(ctx, request.ContestId, request.Color); err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	return &emptypb.Empty{}, nil
}

func (a *adminHandler) GetWallpaper(
	ctx context.Context,
	request *adminv1.GetWallpaperRequest,
) (*adminv1.WallpaperResponse, error) {
	// if id is nil, try to get the first active contest
	if request.ContestId == nil {
		contest, err := a.contestRepo.GetNextContest(ctx)
		if err != nil {
			return nil, connect.NewError(
				connect.CodeInternal,
				fmt.Errorf("failed to get next context to get wallpaper for"),
			)
		}
		if contest == nil {
			return &adminv1.WallpaperResponse{ImageData: []byte{}}, nil
		}
		request.ContestId = &contest.Id
	}
	wallpaper, err := a.wallpaperRepo.GetWallpaper(ctx, *request.ContestId)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	if wallpaper == nil {
		return &adminv1.WallpaperResponse{ImageData: []byte{}}, nil
	}
	return &adminv1.WallpaperResponse{ImageData: wallpaper.Data, Color: &wallpaper.TextColor}, nil
}

// Sets the map for some contest, does not error when either does not exist
func (a *adminHandler) SetMap(
	ctx context.Context,
	request *adminv1.SetMapRequest,
) (*emptypb.Empty, error) {
	err := a.mapRepo.SetMap(ctx, int(request.MapId), request.ContestId)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	return &emptypb.Empty{}, nil
}

var formatToMIME = map[string]string{
	"jpeg": "image/jpeg",
	"png":  "image/png",
	"gif":  "image/gif",
	"webp": "image/webp",
	"tiff": "image/tiff",
	"bmp":  "image/bmp",
}

func validateImageFormat(data []byte) (string, error) {
	if len(data) == 0 {
		return "", fmt.Errorf("empty image data")
	}
	const maxSize = 10 * 1024 * 1024
	if len(data) > maxSize {
		return "", fmt.Errorf("image too large: %d bytes (max %d bytes)", len(data), maxSize)
	}
	config, format, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return "", fmt.Errorf("invalid or corrupted image: %w", err)
	}
	mimeType, ok := formatToMIME[format]
	if !ok {
		return "", fmt.Errorf("unsupported image format: %s", format)
	}
	if config.Width < 1 || config.Height < 1 {
		return "", fmt.Errorf("invalid image dimensions: %dx%d", config.Width, config.Height)
	}
	return mimeType, nil
}
