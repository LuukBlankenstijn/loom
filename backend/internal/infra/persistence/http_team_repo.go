package persistence

import (
	"context"
	"errors"
	"fmt"
	"log/slog"
	"net/http"
	"net/url"
	"strings"

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
	ID       string   `json:"id"`
	TeamID   string   `json:"team_id"`
	IP       string   `json:"ip"`
	Name     string   `json:"name"`
	Username string   `json:"username"`
	Email    string   `json:"email"`
	Enabled  bool     `json:"enabled"`
	Roles    []string `json:"roles"`
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
		return r.getUsers(ctx, &users, nil)
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

func (r *HttpTeamRepository) SetIp(ctx context.Context, teamId string, ip *string) error {
	var allUsers []apiUser
	if err := r.getUsers(ctx, &allUsers, nil); err != nil {
		return err
	}

	newIp := ""
	if ip != nil {
		newIp = *ip
		for _, u := range allUsers {
			if u.IP == newIp && u.TeamID != teamId {
				return errors.New("ip already in use by another team")
			}
		}
	}

	var users []apiUser
	for _, u := range allUsers {
		if u.TeamID == teamId {
			users = append(users, u)
		}
	}

	g, ctx := errgroup.WithContext(ctx)
	for _, u := range users {
		g.Go(func() error {
			endpoint := fmt.Sprintf("%s/api/v4/users/%s", r.baseURL, u.ID)
			form := url.Values{}
			form.Set("id", u.ID)
			form.Set("name", u.Name)
			form.Set("username", u.Username)
			form.Set("email", u.Email)
			form.Set("ip", newIp)
			if u.Enabled {
				form.Set("enabled", "1")
			} else {
				form.Set("enabled", "0")
			}
			for _, role := range u.Roles {
				form.Add("roles[]", role)
			}

			req, err := http.NewRequestWithContext(ctx, http.MethodPut, endpoint, strings.NewReader(form.Encode()))
			if err != nil {
				slog.Error("failed to create http request", "error", err)
				return err
			}
			req.SetBasicAuth(r.username, r.password)
			req.Header.Set("Content-Type", "application/x-www-form-urlencoded")

			resp, err := r.client.Do(req)
			if err != nil {
				slog.Error("http request failed", "error", err)
				return err
			}
			defer resp.Body.Close()

			if resp.StatusCode < 200 || resp.StatusCode >= 300 {
				slog.Error("unexpected status code", "status", resp.StatusCode)
				return fmt.Errorf("api error: status %d", resp.StatusCode)
			}
			return nil
		})
	}

	return g.Wait()
}

// get all users, optionally filtering by team
func (r *HttpTeamRepository) getUsers(ctx context.Context, target *[]apiUser, teamId *string) error {
	url := fmt.Sprintf("%s/api/v4/users", r.baseURL)
	if teamId != nil {
		url = fmt.Sprintf("%s?team_id=%s", url, *teamId)
	}
	return r.get(ctx, url, target)
}
