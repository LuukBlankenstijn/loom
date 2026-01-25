package admin

import (
	"log/slog"
	"net/http"

	"connectrpc.com/connect"
	"connectrpc.com/grpcreflect"
	"connectrpc.com/validate"
	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/gen/go/admin/v1/adminv1connect"
)

type adminServer struct {
	teamService domain.TeamService
	stationRepo domain.StationRepository
	teamRepo    domain.TeamRepository
	contestRepo domain.ContestRepository
}

func New(
	teamService domain.TeamService,
	stationRepo domain.StationRepository,
	teamRepo domain.TeamRepository,
	contestRepo domain.ContestRepository,
) *adminServer {
	return &adminServer{
		teamService,
		stationRepo,
		teamRepo,
		contestRepo,
	}
}

func (a *adminServer) Run() error {
	mux := http.NewServeMux()
	path, handler := adminv1connect.NewAdminServiceHandler(
		a,
		connect.WithInterceptors(validate.NewInterceptor()),
	)
	mux.Handle(path, handler)

	reflector := grpcreflect.NewStaticReflector(
		adminv1connect.AdminServiceName,
	)
	mux.Handle(grpcreflect.NewHandlerV1(reflector))
	mux.Handle(grpcreflect.NewHandlerV1Alpha(reflector))

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
