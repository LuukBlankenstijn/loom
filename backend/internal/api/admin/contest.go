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

var supportedFormats = map[string]bool{
	"jpeg": true,
	"png":  true,
	"gif":  true,
	"webp": true,
	"tiff": true,
	"bmp":  true,
}

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
		if err := validateImageFormat(request.ImageData); err != nil {
			return nil, connect.NewError(connect.CodeInvalidArgument, err)
		}
		if err := a.wallpaperRepo.SetWallpaper(ctx, request.ContestId, &request.ImageData); err != nil {
			return nil, connect.NewError(connect.CodeInternal, err)
		}
		return &emptypb.Empty{}, nil
	}
	if err := a.wallpaperRepo.SetWallpaper(ctx, request.ContestId, nil); err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	return &emptypb.Empty{}, nil
}

func (a *adminHandler) GetWallpaper(
	ctx context.Context,
	request *adminv1.GetWallpaperRequest,
) (*adminv1.WallpaperResponse, error) {
	image, err := a.wallpaperService.GetWallpaper(ctx, request.ContestId)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	if image == nil {
		return &adminv1.WallpaperResponse{ImageData: []byte{}}, nil
	}
	return &adminv1.WallpaperResponse{ImageData: *image}, nil
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

func validateImageFormat(data []byte) error {
	if len(data) == 0 {
		return fmt.Errorf("empty image data")
	}

	// Check max file size (e.g., 10MB)
	const maxSize = 10 * 1024 * 1024
	if len(data) > maxSize {
		return fmt.Errorf("image too large: %d bytes (max %d bytes)", len(data), maxSize)
	}

	config, format, err := image.DecodeConfig(bytes.NewReader(data))
	if err != nil {
		return fmt.Errorf("invalid or corrupted image: %w", err)
	}

	if !supportedFormats[format] {
		return fmt.Errorf("unsupported image format: %s", format)
	}

	if config.Width < 1 || config.Height < 1 {
		return fmt.Errorf("invalid image dimensions: %dx%d", config.Width, config.Height)
	}

	return nil
}
