package stations

import (
	"log/slog"
	"net/http"
	"sync"

	"connectrpc.com/connect"
	"connectrpc.com/grpcreflect"
	"connectrpc.com/validate"
	"github.com/LuukBlankenstijn/loom/gen/go/v1/stations/stationsv1connect"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

type wallpaperState struct {
	mu        sync.RWMutex
	wallpaper *domain.Wallpaper
}

type stationsServer struct {
	contestRepo    domain.ContestRepository
	stationsHub    domain.Hub
	stationsRepo   domain.StationRepository
	teamRepo       domain.TeamRepository
	wallpaperCache wallpaperState
	wallpaperRepo  domain.WallpaperRepository
}

func New(
	hub domain.Hub,
	contestRepo domain.ContestRepository,
	stationsRepo domain.StationRepository,
	teamRepo domain.TeamRepository,
	wallpaperRepo domain.WallpaperRepository,
) *stationsServer {
	return &stationsServer{
		contestRepo:   contestRepo,
		stationsHub:   hub,
		stationsRepo:  stationsRepo,
		teamRepo:      teamRepo,
		wallpaperRepo: wallpaperRepo,
	}
}

func (s *stationsServer) Run() error {
	mux := http.NewServeMux()
	path, handler := stationsv1connect.NewStationServiceHandler(
		s,
		connect.WithInterceptors(validate.NewInterceptor()),
	)
	mux.Handle(path, handler)
	mux.HandleFunc("/wallpaper", s.WallpaperHandler)

	reflector := grpcreflect.NewStaticReflector(
		stationsv1connect.StationServiceName,
	)
	mux.Handle(grpcreflect.NewHandlerV1(reflector))
	mux.Handle(grpcreflect.NewHandlerV1Alpha(reflector))

	p := new(http.Protocols)
	p.SetHTTP1(true)
	p.SetUnencryptedHTTP2(true)
	server := http.Server{
		Addr:      "0.0.0.0:8080",
		Handler:   mux,
		Protocols: p,
	}
	slog.Info("running stations server on 0.0.0.0:8080")
	return server.ListenAndServe()
}
