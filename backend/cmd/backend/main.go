package main

import (
	"context"
	"fmt"
	"log"
	"log/slog"
	"os"
	"time"

	"github.com/LuukBlankenstijn/loom/backend/internal/api/admin"
	"github.com/LuukBlankenstijn/loom/backend/internal/api/stations"
	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/envutil"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/hub"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/persistence"
	"github.com/joho/godotenv"
	_ "github.com/lib/pq"
)

type repoContainer struct {
	team      domain.TeamRepository
	contest   domain.ContestRepository
	station   domain.StationRepository
	wallpaper domain.WallpaperRepository
}

func init() {
	handler := slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelDebug,
	})
	logger := slog.New(handler)

	slog.SetDefault(logger)
}

func main() {
	loadEnv()
	client := createEntClient()
	defer client.Close()
	// Run the auto migration tool.
	if err := client.Schema.Create(context.Background()); err != nil {
		log.Fatalf("failed creating schema resources: %v", err)
	}

	hub := hub.New()
	repoContainer := createRepos(client)
	stationsServer := stations.New(hub, repoContainer.station)
	wallpaperService := domain.NewWallpaperService(repoContainer.contest, repoContainer.wallpaper)
	teamService := domain.NewTeamService(repoContainer.team, repoContainer.contest)
	adminServer := admin.New(*teamService, repoContainer.station, repoContainer.team, repoContainer.contest, repoContainer.wallpaper, *wallpaperService)
	go func() {
		ticker := time.NewTicker(2 * time.Second)
		defer ticker.Stop()

		for range ticker.C {
			hub.Notify(domain.ConfigUpdatedEvent{})
		}
	}()
	go func() {
		if err := adminServer.Run(); err != nil {
			slog.Error("failed to run admin server", slog.Any("error", err))
		}
	}()
	if err := stationsServer.Run(); err != nil {
		slog.Error("failed to run stations server", slog.Any("error", err))
	}
}

func loadEnv() {
	err := godotenv.Load()
	if err != nil {
		slog.Warn("No env file loaded", "error", err)
	}
}

func createEntClient() *ent.Client {
	client, err := ent.Open("postgres", fmt.Sprintf(
		"host=%s port=%s user=%s dbname=%s password=%s sslmode=%s",
		envutil.GetEnvX("DB_HOST"),
		envutil.GetEnvWithFallback("DB_PORT", "5432"),
		envutil.GetEnvWithFallback("DB_USER", "loom"),
		envutil.GetEnvWithFallback("DB_DATABASE", "loom"),
		envutil.GetEnvWithFallback("DB_PASSWORD", "loom"),
		envutil.GetEnvWithFallback("DB_SSLMODE", "disable"),
	))
	if err != nil {
		log.Fatalf("failed opening connection to postgres: %v", err)
	}
	return client
}

func createRepos(client *ent.Client) *repoContainer {
	container := repoContainer{}
	if baseUrl, ok := envutil.GetEnv("DJ_BASE_URL"); ok {
		username, password := envutil.GetEnvX("DJ_USERNAME"), envutil.GetEnvX("DJ_PASSWORD")
		container.contest = persistence.NewHttpContestRepository(baseUrl, username, password)
		container.team = persistence.NewHttpTeamRepository(baseUrl, username, password)
	} else {
		container.contest = persistence.NewEntContestRepository(client)
		container.team = persistence.NewEntTeamRepository(client)
	}
	container.station = persistence.NewEntStationRepository(client)
	container.wallpaper = persistence.NewEntWallpaperRepository(client)
	return &container
}
