package hub

import (
	"log/slog"
	"sync"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

type memoryHub struct {
	mu       sync.RWMutex
	stations map[string]chan domain.ConfigUpdatedEvent
}

func New() domain.Hub {
	return &memoryHub{
		stations: make(map[string]chan domain.ConfigUpdatedEvent),
	}
}

func (m *memoryHub) Register(ip string) (<-chan domain.ConfigUpdatedEvent, func(), error) {
	m.mu.Lock()
	defer m.mu.Unlock()
	if _, exists := m.stations[ip]; exists {
		return nil, nil, domain.ErrAlreadyRegistered
	}
	slog.Debug("[HUB]: registered client", "ip", ip)

	ch := make(chan domain.ConfigUpdatedEvent, 16)
	m.stations[ip] = ch

	cleanup := func() {
		m.mu.Lock()
		defer m.mu.Unlock()
		delete(m.stations, ip)
		close(ch)
		slog.Debug("[HUB]: deregistered client", "ip", ip)
	}

	return ch, cleanup, nil
}

func (m *memoryHub) Notify(event domain.ConfigUpdatedEvent, ips ...string) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	if len(ips) == 0 {
		for _, ch := range m.stations {
			m.trySend(ch, event)
		}
		return
	}

	for _, ip := range ips {
		if ch, ok := m.stations[ip]; ok {
			m.trySend(ch, event)
		}
	}
}

func (m *memoryHub) trySend(ch chan domain.ConfigUpdatedEvent, event domain.ConfigUpdatedEvent) {
	select {
	case ch <- event:
	default:
	}
}
