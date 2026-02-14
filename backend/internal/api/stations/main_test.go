package stations

import (
	"context"
	"net"
	"net/http"
	"net/http/httptest"
	"testing"

	"connectrpc.com/connect"
	stationsv1 "github.com/LuukBlankenstijn/loom/gen/go/stations/v1"
	"github.com/LuukBlankenstijn/loom/gen/go/stations/v1/stationsv1connect"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

// MockHub allows us to control the registration and event flow
type MockHub struct {
	domain.Hub
	RegisterFunc func(ip string) (<-chan domain.ConfigUpdatedEvent, func(), error)
}

func (m *MockHub) Register(ip string) (<-chan domain.ConfigUpdatedEvent, func(), error) {
	return m.RegisterFunc(ip)
}

type MockRepo struct {
	domain.StationRepository
	UpsertFunc               func(ctx context.Context, ip string) error
	UpdateDisconnectedAtFunc func(ctx context.Context, ip string) error
}

func (m *MockRepo) Upsert(ctx context.Context, ip string) error {
	if m.UpsertFunc != nil {
		return m.UpsertFunc(ctx, ip)
	}
	return nil
}

func (m *MockRepo) UpdateDisconnectedAt(ctx context.Context, ip string) error {
	if m.UpdateDisconnectedAtFunc != nil {
		return m.UpdateDisconnectedAtFunc(ctx, ip)
	}
	return nil
}

func TestStationsServer_Connect(t *testing.T) {
	// 1. Setup Mock
	mock := &MockHub{}
	repo := &MockRepo{}

	// 2. Start Test Server
	server := &stationsServer{stationsHub: mock, repo: repo}
	mux := http.NewServeMux()
	mux.Handle(stationsv1connect.NewStationServiceHandler(server))
	listener, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		t.Skipf("skipping test: cannot listen on a local port: %v", err)
	}
	srv := httptest.NewUnstartedServer(mux)
	srv.Listener = listener
	srv.Start()
	defer srv.Close()

	// 3. Create Client
	client := stationsv1connect.NewStationServiceClient(srv.Client(), srv.URL)

	t.Run("Successful Stream", func(t *testing.T) {
		eventCh := make(chan domain.ConfigUpdatedEvent, 1)
		registered := make(chan struct{})
		mock.RegisterFunc = func(ip string) (<-chan domain.ConfigUpdatedEvent, func(), error) {
			close(registered)
			return eventCh, func() { close(eventCh) }, nil
		}

		ctx, cancel := context.WithCancel(t.Context())
		defer cancel()

		type connectResult struct {
			stream *connect.ServerStreamForClient[stationsv1.ConfigUpdatedResponse]
			err    error
		}
		resultCh := make(chan connectResult, 1)
		go func() {
			stream, err := client.Subscribe(ctx, &stationsv1.RegisterRequest{Ip: "127.0.0.1"})
			resultCh <- connectResult{stream: stream, err: err}
		}()

		<-registered
		eventCh <- domain.ConfigUpdatedEvent{}

		result := <-resultCh
		if result.err != nil {
			t.Fatalf("failed to connect: %v", result.err)
		}

		// Verify the client receives the response
		if !result.stream.Receive() {
			t.Fatalf("stream closed unexpectedly: %v", result.stream.Err())
		}

		_ = result.stream.Msg() // Assert message type is correct
	})

	t.Run("Duplicate Registration Error", func(t *testing.T) {
		mock.RegisterFunc = func(ip string) (<-chan domain.ConfigUpdatedEvent, func(), error) {
			return nil, nil, domain.ErrAlreadyRegistered
		}

		stream, err := client.Subscribe(
			context.Background(),
			&stationsv1.RegisterRequest{Ip: "127.0.0.1"},
		)
		if err == nil {
			if stream.Receive() {
				t.Fatal("expected error, got message")
			}
			err = stream.Err()
		}

		if err == nil {
			t.Fatal("expected error, got nil")
		}

		if connect.CodeOf(err) != connect.CodeFailedPrecondition {
			t.Errorf("expected FailedPrecondition, got %v", connect.CodeOf(err))
		}
	})
}
