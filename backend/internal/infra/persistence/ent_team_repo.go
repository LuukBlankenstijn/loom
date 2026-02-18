package persistence

import (
	"context"
	"errors"
	"fmt"
	"log/slog"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/contest"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/station"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/team"
)

type EntTeamRepository struct {
	client *ent.Client
}

func NewEntTeamRepository(client *ent.Client) *EntTeamRepository {
	return &EntTeamRepository{client: client}
}

func (r *EntTeamRepository) SetIp(ctx context.Context, teamId string, ip *string) error {
	if ip != nil {
		teams, err := r.client.Team.Query().Where(team.HasStationWith(station.IPEQ(*ip))).All(ctx)
		if err != nil {
			slog.Error("failed to check if ip is unused when setting ip", "error", err)
			return errors.New("failed to check if ip is unused")
		}
		if len(teams) > 0 {
			return errors.New("ip is already use by a different team")
		}
		err = r.
			client.
			Team.
			UpdateOneID(teamId).
			SetStation(
				r.
					client.
					Station.
					Query().
					Where(station.IPEQ(*ip)).
					OnlyX(ctx),
			).
			Exec(ctx)
		if err != nil {
			slog.Error("failed to update ip", "error", err)
			return errors.New("failed to update ip")
		}
		return nil
	}
	err := r.
		client.
		Team.
		UpdateOneID(teamId).
		ClearStation().
		Exec(ctx)
	if err != nil {
		slog.Error("failed to update ip", "error", err)
		return errors.New("failed to update ip")
	}
	return nil
}

func (r *EntTeamRepository) GetAll(ctx context.Context, contestId string) ([]domain.Team, error) {
	eTeams, err := r.
		client.
		Team.
		Query().
		WithStation().
		Where(
			team.HasContestsWith(contest.ID(contestId)),
		).
		All(ctx)
	if err != nil {
		slog.Error("failed to get all teams", "error", err)
		return []domain.Team{}, errors.New("failed to get all teams")
	}
	teams := []domain.Team{}
	for _, eTeam := range eTeams {
		var ip *string
		if eTeam.Edges.Station != nil {
			ip = &eTeam.Edges.Station.IP
		} else {
			ip = nil
		}
		teams = append(teams, domain.Team{
			Id:   eTeam.ID,
			Name: eTeam.Name,
			Ip:   ip,
		})
	}
	return teams, nil
}

func (r *EntTeamRepository) GetByIp(ctx context.Context, ip string) (*domain.Team, error) {
	eTeam, err := r.client.Team.Query().
		WithStation().
		Where(team.HasStationWith(station.IP(ip))).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil
		}
		slog.Error("error while getting team by ip", slog.Any("err", err))
		return nil, fmt.Errorf("unexpected error")
	}
	return &domain.Team{
		Id:   eTeam.ID,
		Name: eTeam.Name,
		Ip:   &ip,
	}, nil
}
