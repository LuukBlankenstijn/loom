package stations

import (
	"context"
	"log/slog"
	"net/http"
	"strconv"
	"sync"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

func (s *stationsServer) WallpaperHandler(w http.ResponseWriter, r *http.Request) {
	ctx := r.Context()
	ip := getRealIP(r)

	var (
		contest *domain.Contest
		team    *domain.Team
		err     error
		wg      sync.WaitGroup
	)

	wg.Add(2)

	// Fetch Contest and Team in parallel to minimize external API latency
	go func() {
		defer wg.Done()
		contest, err = s.contestRepo.GetNextContest(ctx)
	}()

	go func() {
		defer wg.Done()
		team, _ = s.teamRepo.GetByIp(ctx, ip)
	}()

	wg.Wait()

	if err != nil {
		slog.Error("failed to get wallpaper, contest fetch failed", slog.Any("err", err))
		http.Error(w, "failed to get contest context", http.StatusNotFound)
		return
	}

	// Ensure the memory cache is in sync with the current contest
	if err := s.updateCache(ctx, contest.Id); err != nil {
		slog.Error("cache refresh failed", "err", err)
		http.Error(w, "wallpaper not set", http.StatusServiceUnavailable)
		return
	}

	s.wallpaperCache.mu.RLock()
	cached := s.wallpaperCache.wallpaper
	s.wallpaperCache.mu.RUnlock()

	if cached == nil {
		http.Error(w, "wallpaper not initialized", http.StatusServiceUnavailable)
		return
	}

	// Set dynamic headers based on the fetched team
	if team != nil {
		w.Header().Set("X-Wallpaper-Text", team.Name)
		w.Header().Set("X-Wallpaper-Text-Color", cached.TextColor)
	}

	w.Header().Set("Content-Type", cached.MimeType)
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Content-Length", strconv.Itoa(len(cached.Data)))

	// Serve the cached blob directly from memory
	w.Write(cached.Data)
}

func (s *stationsServer) updateCache(ctx context.Context, contestID string) error {
	// Fetch only the timestamp to check for freshness
	dbUpdatedAt, err := s.wallpaperRepo.GetLastUpdated(ctx, contestID)
	if err != nil {
		return err
	}

	// Fast path: check if cache is already current
	s.wallpaperCache.mu.RLock()
	isCurrent := s.wallpaperCache.wallpaper != nil &&
		s.wallpaperCache.wallpaper.UpdatedAt.Equal(dbUpdatedAt)
	s.wallpaperCache.mu.RUnlock()

	if isCurrent {
		return nil
	}

	s.wallpaperCache.mu.Lock()
	defer s.wallpaperCache.mu.Unlock()

	// Double-check inside lock to handle concurrent requests
	if s.wallpaperCache.wallpaper != nil &&
		!s.wallpaperCache.wallpaper.UpdatedAt.Before(dbUpdatedAt) {
		return nil
	}

	// Fetch the full blob only when necessary
	wallpaper, err := s.wallpaperRepo.GetWallpaper(ctx, contestID)
	if err != nil {
		return err
	}

	s.wallpaperCache.wallpaper = wallpaper
	return nil
}

func getRealIP(r *http.Request) string {
	IPAddress := r.Header.Get("X-Real-IP")
	if IPAddress == "" {
		IPAddress = r.Header.Get("X-Forwarded-For")
	}
	if IPAddress == "" {
		IPAddress = r.RemoteAddr
	}
	return IPAddress
}
