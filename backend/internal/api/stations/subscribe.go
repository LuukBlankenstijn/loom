package stations

import (
	"context"
	"errors"
	"log/slog"

	"connectrpc.com/connect"
	stationsv1 "github.com/LuukBlankenstijn/loom/gen/go/stations/v1"
	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

func (s *stationsServer) Subscribe(
	ctx context.Context,
	connectRequest *stationsv1.RegisterRequest,
	serverStream *connect.ServerStream[stationsv1.ConfigUpdatedResponse],
) error {
	err := s.repo.Upsert(ctx, connectRequest.Ip)
	if err != nil {
		slog.Error("failed to upsert station", "ip", connectRequest.Ip, "err", err)
		return connect.NewError(connect.CodeInternal, errors.New("failed to connect"))
	}
	channel, cleanup, err := s.stationsHub.Register(connectRequest.Ip)
	if err != nil {
		if errors.Is(err, domain.ErrAlreadyRegistered) {
			slog.Warn("Station registered while it was already connected", "ip", connectRequest.Ip)
			return connect.NewError(connect.CodeFailedPrecondition, errors.New("station already connected"))
		} else {
			return connect.NewError(connect.CodeInternal, err)
		}
	}
	defer func() {
		cleanup()

		// Use context.Background because ctx is cancelled here
		_ = s.repo.UpdateDisconnectedAt(context.Background(), connectRequest.Ip)
	}()
	for {
		select {
		case <-ctx.Done():
			return ctx.Err()
		case _, ok := <-channel:
			if !ok {
				return nil
			}
			err := serverStream.Send(&stationsv1.ConfigUpdatedResponse{})
			if err != nil {
				// If we can't send, the connection is dead.
				return err
			}
		}
	}
}
