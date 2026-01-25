package persistence

import (
	"context"
	"testing"
	"time"

	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/enttest"
	_ "github.com/mattn/go-sqlite3"
)

func TestEntContestRepositoryGetNextContest(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_contest_next?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	now := time.Now()
	_, err := client.Contest.
		Create().
		SetID("past").
		SetName("Past").
		SetStartTime(now.Add(-2 * time.Hour)).
		SetEndTime(now.Add(-1 * time.Hour)).
		Save(ctx)
	if err != nil {
		t.Fatalf("create past contest: %v", err)
	}
	_, err = client.Contest.
		Create().
		SetID("later").
		SetName("Later").
		SetStartTime(now.Add(2 * time.Hour)).
		SetEndTime(now.Add(3 * time.Hour)).
		Save(ctx)
	if err != nil {
		t.Fatalf("create later contest: %v", err)
	}
	_, err = client.Contest.
		Create().
		SetID("next").
		SetName("Next").
		SetStartTime(now.Add(1 * time.Hour)).
		SetEndTime(now.Add(2 * time.Hour)).
		Save(ctx)
	if err != nil {
		t.Fatalf("create next contest: %v", err)
	}

	repo := NewEntContestRepository(client)
	got, err := repo.GetNextContest(ctx)
	if err != nil {
		t.Fatalf("get next contest: %v", err)
	}
	if got == nil {
		t.Fatalf("expected contest, got nil")
	}
	if got.Id != "next" {
		t.Fatalf("unexpected contest id: %s", got.Id)
	}
}

func TestEntContestRepositoryGetNextContestNone(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_contest_none?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	now := time.Now()
	_, err := client.Contest.
		Create().
		SetID("past").
		SetName("Past").
		SetStartTime(now.Add(-2 * time.Hour)).
		SetEndTime(now.Add(-1 * time.Hour)).
		Save(ctx)
	if err != nil {
		t.Fatalf("create past contest: %v", err)
	}

	repo := NewEntContestRepository(client)
	got, err := repo.GetNextContest(ctx)
	if err != nil {
		t.Fatalf("get next contest: %v", err)
	}
	if got != nil {
		t.Fatalf("expected nil contest, got %v", got)
	}
}
