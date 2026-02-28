package hub

import (
	"log/slog"
	"sync"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

type memoryHub struct {
	mu       sync.RWMutex
	stations map[string]stationState
}

type stationState struct {
	channel  chan domain.StationHubEvent
	loggedIn bool
}

func New() domain.Hub {
	return &memoryHub{
		stations: make(map[string]stationState),
	}
}

func (m *memoryHub) Register(ip string) (<-chan domain.StationHubEvent, func(), error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if _, exists := m.stations[ip]; exists {
		return nil, nil, domain.ErrAlreadyRegistered
	}
	slog.Debug("[HUB]: registered client", "ip", ip)

	ch := make(chan domain.StationHubEvent, 16)
	m.stations[ip] = stationState{
		channel:  ch,
		loggedIn: false,
	}

	cleanup := func() {
		m.mu.Lock()
		defer m.mu.Unlock()
		delete(m.stations, ip)
		close(ch)
		slog.Debug("[HUB]: deregistered client", "ip", ip)
	}

	return ch, cleanup, nil
}

func (m *memoryHub) Send(event domain.StationHubEvent, ips ...string) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	if len(ips) == 0 {
		for _, state := range m.stations {
			m.trySend(state.channel, event)
		}
		return
	}

	for _, ip := range ips {
		if state, ok := m.stations[ip]; ok {
			m.trySend(state.channel, event)
		}
	}
}

func (m *memoryHub) trySend(ch chan domain.StationHubEvent, event domain.StationHubEvent) {
	select {
	case ch <- event:
	default:
	}
}

func (m *memoryHub) SetLoginStatus(stationIp string, loggedIn bool) {
	m.mu.Lock()
	defer m.mu.Unlock()

	state, exists := m.stations[stationIp]
	if !exists {
		return
	}
	state.loggedIn = loggedIn
	m.stations[stationIp] = state
}
