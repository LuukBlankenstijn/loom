package domain

import "context"

type Team struct {
	Id   string
	Name string
	Ip   *string
}

type TeamRepository interface {
	// Sets the ip of a team if it exists
	// Errors if some other team already uses the ip
	SetIp(ctx context.Context, teamId string, ip *string) error

	// Gets all teams for some contest
	GetAll(ctx context.Context, contestId string) ([]Team, error)
}
