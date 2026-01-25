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

func init() {
	handler := slog.NewJSONHandler(os.Stdout, &slog.HandlerOptions{
		Level: slog.LevelDebug,
	})
	logger := slog.New(handler)

	slog.SetDefault(logger)
}

func main() {
	err := godotenv.Load()
	if err != nil {
		slog.Warn("No env file loaded", "error", err)
	}
	client, err := ent.Open("postgres", fmt.Sprintf(
		"host=%s port=%s user=%s dbname=%s password=%s sslmode=%s",
		envutil.GetEnv("DB_HOST"),
		envutil.GetEnvWithFallback("DB_PORT", "5432"),
		envutil.GetEnvWithFallback("DB_USER", "loom"),
		envutil.GetEnvWithFallback("DB_DATABASE", "loom"),
		envutil.GetEnvWithFallback("DB_PASSWORD", "loom"),
		envutil.GetEnvWithFallback("DB_SSLMODE", "disable"),
	))
	if err != nil {
		log.Fatalf("failed opening connection to postgres: %v", err)
	}
	defer client.Close()
	// Run the auto migration tool.
	if err := client.Schema.Create(context.Background()); err != nil {
		log.Fatalf("failed creating schema resources: %v", err)
	}

	hub := hub.New()
	stationsRepo := persistence.NewEntStationRepository(client)
	teamsRepo := persistence.NewEntTeamRepository(client)
	contestRepo := persistence.NewEntContestRepository(client)
	stationsServer := stations.New(hub, stationsRepo)
	teamService := domain.NewTeamService(teamsRepo, contestRepo)
	adminServer := admin.New(*teamService, stationsRepo, teamsRepo, contestRepo)
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
