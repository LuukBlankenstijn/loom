package persistence

import (
	"context"
	"errors"
	"fmt"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"golang.org/x/sync/errgroup"
)

type HttpTeamRepository struct {
	httpBaseRepository
}

func NewHttpTeamRepository(baseURL, username, password string) *HttpTeamRepository {
	return &HttpTeamRepository{
		httpBaseRepository: newHTTPBaseRepository(baseURL, username, password),
	}
}

// DTOs matching the provided API definitions
type apiTeam struct {
	ID   string `json:"id"`
	Name string `json:"name"`
}

type apiUser struct {
	TeamID string `json:"team_id"`
	IP     string `json:"ip"`
}

func (r *HttpTeamRepository) SetIp(ctx context.Context, teamId string, ip *string) error {
	return errors.New("SetIp not implemented for HTTP repository")
}

func (r *HttpTeamRepository) GetAll(ctx context.Context, contestId string) ([]domain.Team, error) {
	var teams []apiTeam
	var users []apiUser

	// Fetch teams and users in parallel
	g, ctx := errgroup.WithContext(ctx)

	g.Go(func() error {
		// Endpoint for all teams in a contest
		url := fmt.Sprintf("%s/api/v4/contests/%s/teams", r.baseURL, contestId)
		return r.get(ctx, url, &teams)
	})

	g.Go(func() error {
		// Endpoint for all users
		url := fmt.Sprintf("%s/api/v4/users", r.baseURL)
		return r.get(ctx, url, &users)
	})

	if err := g.Wait(); err != nil {
		return nil, err
	}

	// Build a map of TeamID -> IP
	// If multiple users exist per team, the last one processed wins.
	ipMap := make(map[string]string)
	for _, u := range users {
		if u.IP != "" && u.TeamID != "" {
			ipMap[u.TeamID] = u.IP
		}
	}

	// Join the data into domain entities
	result := make([]domain.Team, 0, len(teams))
	for _, t := range teams {
		var teamIP *string
		if ip, ok := ipMap[t.ID]; ok {
			val := ip
			teamIP = &val
		}

		result = append(result, domain.Team{
			Id:   t.ID,
			Name: t.Name,
			Ip:   teamIP,
		})
	}

	return result, nil
}
