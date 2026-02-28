package stations

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net"

	"connectrpc.com/connect"
	commandv1 "github.com/LuukBlankenstijn/loom/gen/go/v1/command"
	stationsv1 "github.com/LuukBlankenstijn/loom/gen/go/v1/stations"
	"golang.org/x/sync/errgroup"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/envutil"
)

func (s *stationsServer) Subscribe(
	ctx context.Context,
	stream *connect.BidiStream[stationsv1.ClientMessage, stationsv1.ServerMessage],
) error {
	ip, _, err := net.SplitHostPort(stream.Peer().Addr)
	if err != nil {
		slog.Error("failed to parse ip address", slog.Any("err", err))
		return connect.NewError(
			connect.CodeInvalidArgument,
			errors.New("could not get client ip from stream"),
		)
	}

	if err := s.stationsRepo.Upsert(ctx, ip); err != nil {
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
		}
		return connect.NewError(connect.CodeInternal, err)
	}

	defer func() {
		cleanup()
		_ = s.stationsRepo.UpdateDisconnectedAt(context.Background(), ip)
	}()

	g, gCtx := errgroup.WithContext(ctx)

	// Receiver loop (Client -> Hub)
	g.Go(func() error {
		for {
			msg, err := stream.Receive()
			if err != nil {
				return err
			}
			s.handleClientMessage(ip, msg)
		}
	})

	// Sender loop (Hub -> Client)
	g.Go(func() error {
		for {
			select {
			case <-gCtx.Done():
				return gCtx.Err()
			case evt, ok := <-channel:
				if !ok {
					return nil
				}
				if err := stream.Send(s.mapEventToMessage(evt)); err != nil {
					return err
				}
			}
		}
	})

	g.Go(func() error {
		if baseUrl, valid := envutil.GetEnv("DJ_BASE_URL"); valid {
			contest, err := s.contestRepo.GetNextContest(ctx)
			if err == nil && contest != nil {
				contestUrl := fmt.Sprintf("%s/api/v4/contests/%s", baseUrl, contest.Id)
				s.stationsHub.Send(domain.SetContestUrl{Url: contestUrl})
			}
		}
		host := stream.RequestHeader().Get("Host")
		s.stationsHub.Send(
			domain.SetWallpaperSource{Source: fmt.Sprintf("http://%s/wallpaper", host)},
		)
		return nil
	})

	return g.Wait()
}

func (s *stationsServer) handleClientMessage(ip string, msg *stationsv1.ClientMessage) {
	switch msg.Message.(type) {
	case *stationsv1.ClientMessage_LoggedIn:
		s.stationsHub.SetLoginStatus(ip, true)
	case *stationsv1.ClientMessage_LoggedOut:
		s.stationsHub.SetLoginStatus(ip, false)
	}
}

func (s *stationsServer) mapEventToMessage(evt domain.StationHubEvent) *stationsv1.ServerMessage {
	switch e := evt.(type) {
	case domain.SetContestUrl:
		return &stationsv1.ServerMessage{
			Message: &stationsv1.ServerMessage_SetContestUrl{
				SetContestUrl: e.Url,
			},
		}
	case domain.SetWallpaperSource:
		return &stationsv1.ServerMessage{
			Message: &stationsv1.ServerMessage_SetWallpaperSource{
				SetWallpaperSource: e.Source,
			},
		}
	case domain.Login:
		return &stationsv1.ServerMessage{
			Message: &stationsv1.ServerMessage_Login{
				Login: &commandv1.LoginCommand{},
			},
		}
	case domain.Logout:
		return &stationsv1.ServerMessage{
			Message: &stationsv1.ServerMessage_Logout{
				Logout: &commandv1.LogoutCommand{},
			},
		}
	case domain.LoginWithCredentials:
		return &stationsv1.ServerMessage{
			Message: &stationsv1.ServerMessage_LoginWithCredentials{
				LoginWithCredentials: &commandv1.LoginWithCredentialsCommand{
					Username: e.Username,
					Password: e.Password,
				},
			},
		}
	case domain.CustomCommand:
		return &stationsv1.ServerMessage{
			Message: &stationsv1.ServerMessage_CustomCommand{
				CustomCommand: &commandv1.CustomCommand{
					Id:      e.Id.String(),
					Command: e.Command,
				},
			},
		}
	default:
		return nil
	}
}
