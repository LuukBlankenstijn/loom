package persistence

import (
	"context"
	"testing"
	"time"

	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/enttest"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/station"
	_ "github.com/mattn/go-sqlite3"
)

func TestEntStationRepositoryUpsertCreatesAndUpdates(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_upsert?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntStationRepository(client)
	ip := "127.0.0.1"

	if err := repo.Upsert(ctx, ip); err != nil {
		t.Fatalf("upsert create: %v", err)
	}

	created, err := client.Station.Query().Where(station.IPEQ(ip)).Only(ctx)
	if err != nil {
		t.Fatalf("query after create: %v", err)
	}
	firstConnectedAt := created.ConnectedAt

	time.Sleep(10 * time.Millisecond)

	if err := repo.Upsert(ctx, ip); err != nil {
		t.Fatalf("upsert update: %v", err)
	}

	updated, err := client.Station.Query().Where(station.IPEQ(ip)).Only(ctx)
	if err != nil {
		t.Fatalf("query after update: %v", err)
	}
	if !updated.ConnectedAt.After(firstConnectedAt) {
		t.Fatalf("connected_at was not updated: before=%v after=%v", firstConnectedAt, updated.ConnectedAt)
	}
}

func TestEntStationRepositoryUpdateDisconnectedAt(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_disconnect?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntStationRepository(client)
	ip := "127.0.0.1"

	created, err := client.Station.Create().SetIP(ip).SetConnectedAt(time.Now()).Save(ctx)
	if err != nil {
		t.Fatalf("create station: %v", err)
	}

	if err := repo.UpdateDisconnectedAt(ctx, ip); err != nil {
		t.Fatalf("update disconnected_at: %v", err)
	}

	updated, err := client.Station.Query().Where(station.IPEQ(ip)).Only(ctx)
	if err != nil {
		t.Fatalf("query after update: %v", err)
	}
	if updated.DisconnectedAt == nil {
		t.Fatalf("disconnected_at not set")
	}
	if updated.DisconnectedAt.Before(created.ConnectedAt) {
		t.Fatalf("disconnected_at is before connected_at: connected=%v disconnected=%v", created.ConnectedAt, *updated.DisconnectedAt)
	}
}
