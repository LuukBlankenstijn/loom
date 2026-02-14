package admin

import (
	"log/slog"
	"net/http"

	"connectrpc.com/connect"
	"connectrpc.com/grpcreflect"
	"connectrpc.com/validate"
	"github.com/LuukBlankenstijn/loom/gen/go/admin/v1/adminv1connect"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/envutil"
)

type adminHandler struct {
	teamService      domain.TeamService
	wallpaperService domain.WallpaperService
	stationRepo      domain.StationRepository
	teamRepo         domain.TeamRepository
	contestRepo      domain.ContestRepository
	wallpaperRepo    domain.WallpaperRepository
	mapRepo          domain.MapRepository
}

func NewAdminHandler(
	teamService domain.TeamService,
	stationRepo domain.StationRepository,
	teamRepo domain.TeamRepository,
	contestRepo domain.ContestRepository,
	wallpaperRepo domain.WallpaperRepository,
	wallpaperService domain.WallpaperService,
	mapRepo domain.MapRepository,
) *adminHandler {
	return &adminHandler{
		teamService,
		wallpaperService,
		stationRepo,
		teamRepo,
		contestRepo,
		wallpaperRepo,
		mapRepo,
	}
}

func (a *adminHandler) Run() error {
	mux := http.NewServeMux()
	path, handler := adminv1connect.NewAdminServiceHandler(
		a,
		connect.WithInterceptors(validate.NewInterceptor()),
	)
	mux.Handle(path, handler)

	reflector := grpcreflect.NewStaticReflector(
		adminv1connect.AdminServiceName,
	)

	if envutil.GetEnvWithFallback("ENABLE_REFLECTOR", "false") == "true" {
		mux.Handle(grpcreflect.NewHandlerV1(reflector))
		mux.Handle(grpcreflect.NewHandlerV1Alpha(reflector))
	}

	p := new(http.Protocols)
	p.SetHTTP1(true)
	p.SetUnencryptedHTTP2(true)
	server := http.Server{
		Addr:      "0.0.0.0:8081",
		Handler:   mux,
		Protocols: p,
	}

	slog.Info("running admin server on 0.0.0.0:8081")
	return server.ListenAndServe()
}
