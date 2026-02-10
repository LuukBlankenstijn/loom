package domain

import (
	"context"
)

type WallpaperService struct {
	contestRepo   ContestRepository
	wallpaperRepo WallpaperRepository
}

func NewWallpaperService(contestRepo ContestRepository, wallpaperRepo WallpaperRepository) *WallpaperService {
	return &WallpaperService{contestRepo, wallpaperRepo}
}

func (s *WallpaperService) GetWallpaper(ctx context.Context, contest_id *string) (*[]byte, error) {
	if contest_id == nil {
		contest, err := s.contestRepo.GetNextContest(ctx)
		if err != nil {
			return nil, err
		}
		if contest == nil {
			return nil, nil
		}
		contest_id = &contest.Id
	}
	return s.wallpaperRepo.GetWallpaper(ctx, *contest_id)
}
