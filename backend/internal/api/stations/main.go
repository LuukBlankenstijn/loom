package stations

import (
	"log/slog"
	"net/http"

	"connectrpc.com/connect"
	"connectrpc.com/grpcreflect"
	"connectrpc.com/validate"
	"github.com/LuukBlankenstijn/loom/gen/go/stations/v1/stationsv1connect"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

type stationsServer struct {
	stationsHub domain.Hub
	repo        domain.StationRepository
}

func New(
	hub domain.Hub,
	repo domain.StationRepository,
) *stationsServer {
	return &stationsServer{
		stationsHub: hub,
		repo:        repo,
	}
}

func (s *stationsServer) Run() error {
	mux := http.NewServeMux()
	path, handler := stationsv1connect.NewStationServiceHandler(
		s,
		connect.WithInterceptors(validate.NewInterceptor()),
	)
	mux.Handle(path, handler)

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
