package hub

import (
	"testing"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

func TestMemoryHub_Register(t *testing.T) {
	hub := New()
	ip := "192.168.1.10"

	// Test successful registration
	ch, cleanup, err := hub.Register(ip)
	if err != nil {
		t.Fatalf("expected no error, got %v", err)
	}
	if ch == nil || cleanup == nil {
		t.Fatal("returned channel or cleanup is nil")
	}

	// Test duplicate registration
	_, _, err = hub.Register(ip)
	if err != domain.ErrAlreadyRegistered {
		t.Errorf("expected ErrAlreadyRegistered, got %v", err)
	}

	// Test cleanup
	cleanup()
	_, cleanup2, err := hub.Register(ip)
	if err != nil {
		t.Errorf("expected to be able to register again after cleanup, got %v", err)
	}
	cleanup2()
}

func TestMemoryHub_Notify(t *testing.T) {
	hub := New()
	ip1, ip2 := "1.1.1.1", "2.2.2.2"

	ch1, c1, _ := hub.Register(ip1)
	defer c1()
	ch2, c2, _ := hub.Register(ip2)
	defer c2()

	event := domain.ConfigUpdatedEvent{}

	t.Run("Broadcast to all", func(t *testing.T) {
		hub.Notify(event)

		select {
		case <-ch1:
		default:
			t.Error("ch1 did not receive broadcast")
		}
		select {
		case <-ch2:
		default:
			t.Error("ch2 did not receive broadcast")
		}
	})

	t.Run("Targeted notification", func(t *testing.T) {
		hub.Notify(event, ip1)

		select {
		case <-ch1:
		default:
			t.Error("ch1 did not receive targeted notify")
		}
		select {
		case <-ch2:
			t.Error("ch2 received message meant for ch1")
		default:
		}
	})
}

func TestMemoryHub_DroppedMessages(t *testing.T) {
	hub := New()
	ip := "1.2.3.4"
	// Registry uses buffer of 16. We will fill it + 1.
	ch, cleanup, _ := hub.Register(ip)
	defer cleanup()

	event := domain.ConfigUpdatedEvent{}

	// Fill the buffer (16)
	for range 16 {
		hub.Notify(event, ip)
	}

	// This 17th message should be dropped immediately via 'default' in trySend
	hub.Notify(event, ip)

	// Verify we can still read the 16 messages and the system didn't hang
	count := 0
	for range 16 {
		select {
		case <-ch:
			count++
		default:
		}
	}

	if count != 16 {
		t.Errorf("expected 16 messages, got %d", count)
	}
}
