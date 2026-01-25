package persistence

import (
	"context"
	"testing"
	"time"

	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/enttest"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/team"
	_ "github.com/mattn/go-sqlite3"
)

func TestEntTeamRepositorySetIp(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_team_setip?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	stationRec, err := client.Station.Create().SetIP("10.0.0.3").SetConnectedAt(time.Now()).Save(ctx)
	if err != nil {
		t.Fatalf("create station: %v", err)
	}
	teamRec, err := client.Team.Create().SetID("team-2").SetName("Team 2").Save(ctx)
	if err != nil {
		t.Fatalf("create team: %v", err)
	}

	repo := NewEntTeamRepository(client)
	ip := "10.0.0.3"
	if err := repo.SetIp(ctx, teamRec.ID, &ip); err != nil {
		t.Fatalf("set ip: %v", err)
	}

	updated, err := client.Team.Query().Where(team.ID(teamRec.ID)).WithStation().Only(ctx)
	if err != nil {
		t.Fatalf("query team: %v", err)
	}
	if updated.Edges.Station == nil || updated.Edges.Station.ID != stationRec.ID {
		t.Fatalf("station not set correctly")
	}
}

func TestEntTeamRepositorySetIpAlreadyUsed(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_team_setip_used?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	stationRec, err := client.Station.Create().SetIP("10.0.0.4").SetConnectedAt(time.Now()).Save(ctx)
	if err != nil {
		t.Fatalf("create station: %v", err)
	}
	_, err = client.Team.Create().SetID("team-a").SetName("Team A").SetStation(stationRec).Save(ctx)
	if err != nil {
		t.Fatalf("create team a: %v", err)
	}
	teamB, err := client.Team.Create().SetID("team-b").SetName("Team B").Save(ctx)
	if err != nil {
		t.Fatalf("create team b: %v", err)
	}

	repo := NewEntTeamRepository(client)
	ip := "10.0.0.4"
	if err := repo.SetIp(ctx, teamB.ID, &ip); err == nil {
		t.Fatalf("expected error when ip is already used")
	}

	updated, err := client.Team.Query().Where(team.ID(teamB.ID)).WithStation().Only(ctx)
	if err != nil {
		t.Fatalf("query team b: %v", err)
	}
	if updated.Edges.Station != nil {
		t.Fatalf("station should remain unset")
	}
}

func TestEntTeamRepositorySetIpNilRemovesStation(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_team_setip_nil?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	stationRec, err := client.Station.Create().SetIP("10.0.0.5").SetConnectedAt(time.Now()).Save(ctx)
	if err != nil {
		t.Fatalf("create station: %v", err)
	}
	teamRec, err := client.Team.Create().SetID("team-c").SetName("Team C").SetStation(stationRec).Save(ctx)
	if err != nil {
		t.Fatalf("create team: %v", err)
	}

	repo := NewEntTeamRepository(client)
	if err := repo.SetIp(ctx, teamRec.ID, nil); err != nil {
		t.Fatalf("clear ip: %v", err)
	}

	updated, err := client.Team.Query().Where(team.ID(teamRec.ID)).WithStation().Only(ctx)
	if err != nil {
		t.Fatalf("query team: %v", err)
	}
	if updated.Edges.Station != nil {
		t.Fatalf("station not cleared")
	}
}

func TestEntTeamRepositoryGetAllWithContestFilter(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_team_getall?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	station1, err := client.Station.Create().SetIP("10.0.1.1").SetConnectedAt(time.Now()).Save(ctx)
	if err != nil {
		t.Fatalf("create station1: %v", err)
	}
	station2, err := client.Station.Create().SetIP("10.0.1.2").SetConnectedAt(time.Now()).Save(ctx)
	if err != nil {
		t.Fatalf("create station2: %v", err)
	}
	team1, err := client.Team.Create().SetID("team-1").SetName("Team 1").SetStation(station1).Save(ctx)
	if err != nil {
		t.Fatalf("create team1: %v", err)
	}
	_, err = client.Team.Create().SetID("team-2").SetName("Team 2").SetStation(station2).Save(ctx)
	if err != nil {
		t.Fatalf("create team2: %v", err)
	}
	contestRec, err := client.Contest.
		Create().
		SetID("contest-1").
		SetName("Contest 1").
		SetStartTime(time.Now()).
		SetEndTime(time.Now().Add(1 * time.Hour)).
		AddTeamIDs(team1.ID).
		Save(ctx)
	if err != nil {
		t.Fatalf("create contest: %v", err)
	}

	repo := NewEntTeamRepository(client)
	filtered, err := repo.GetAll(ctx, contestRec.ID)
	if err != nil {
		t.Fatalf("get all with contest: %v", err)
	}
	if len(filtered) != 1 || filtered[0].Id != team1.ID {
		t.Fatalf("unexpected filtered teams: %+v", filtered)
	}
}
