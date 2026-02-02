package admin

import (
	"context"
	"errors"

	"connectrpc.com/connect"
	adminv1 "github.com/LuukBlankenstijn/loom/gen/go/admin/v1"
	"google.golang.org/protobuf/types/known/emptypb"
)

// Gets all teams for the next active contest
func (a *adminServer) GetActiveTeams(ctx context.Context, empty *emptypb.Empty) (*adminv1.TeamsResponse, error) {
	domainTeams, err := a.teamService.GetTeamsForActiveContest(ctx)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, errors.New("failed to get teams"))
	}
	teams := []*adminv1.Team{}
	for _, s := range domainTeams {
		teams = append(teams, &adminv1.Team{
			Id:   s.Id,
			Ip:   s.Ip,
			Name: s.Name,
		})
	}
	return &adminv1.TeamsResponse{
		Teams: teams,
	}, nil
}

// Sets the ip of some team. Only allows ips that are now used yet by other teams
func (a *adminServer) SetIp(ctx context.Context, request *adminv1.SetIpRequest) (*emptypb.Empty, error) {
	err := a.teamRepo.SetIp(ctx, request.TeamId, request.Ip)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, errors.New("failed to set ip"))
	}
	return &emptypb.Empty{}, nil
}
