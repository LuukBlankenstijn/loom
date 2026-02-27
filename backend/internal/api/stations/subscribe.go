package stations

import (
	"context"
	"errors"
	"log/slog"
	"net"

	"connectrpc.com/connect"
	stationsv1 "github.com/LuukBlankenstijn/loom/gen/go/v1/stations"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

func (s *stationsServer) Subscribe(
	ctx context.Context,
	serverStream *connect.BidiStream[stationsv1.ClientMessage, stationsv1.ServerMessage],
) error {
	ip, _, err := net.SplitHostPort(serverStream.Peer().Addr)
	if err != nil {
		slog.Error("failed to parse ip address", err)
		return connect.NewError(
			connect.CodeInvalidArgument,
			errors.New("could not get client ip from stream"),
		)
	}
	err = s.stationsRepo.Upsert(ctx, ip)
	if err != nil {
		slog.Error("failed to upsert station", "ip", ip, "err", err)
		return connect.NewError(connect.CodeInternal, errors.New("failed to connect"))
	}
	channel, cleanup, err := s.stationsHub.Register(ip)
	if err != nil {
		if errors.Is(err, domain.ErrAlreadyRegistered) {
			slog.Warn("Station registered while it was already connected", "ip", ip)
			return connect.NewError(
				connect.CodeFailedPrecondition,
				errors.New("station already connected"),
			)
		} else {
			return connect.NewError(connect.CodeInternal, err)
		}
	}
	defer func() {
		cleanup()

		// Use context.Background because ctx is cancelled here
		_ = s.stationsRepo.UpdateDisconnectedAt(context.Background(), ip)
	}()
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case _, ok := <-channel:
			if !ok {
				return nil
			}
			err := serverStream.Send(&stationsv1.ServerMessage{
				Message: &stationsv1.ServerMessage_SetWallpaperSource{
					SetWallpaperSource: "Some address",
				},
			})
			if err != nil {
				// If we can't send, the connection is dead.
				return err
			}
		}
	}
}
