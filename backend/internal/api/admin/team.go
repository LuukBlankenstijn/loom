package admin

import (
	"context"
	"errors"

	"connectrpc.com/connect"
	adminv1 "github.com/LuukBlankenstijn/loom/gen/go/admin/v1"
	"google.golang.org/protobuf/types/known/emptypb"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

// Gets all teams for the next active contest
func (a *adminHandler) GetActiveTeams(
	ctx context.Context,
	empty *emptypb.Empty,
) (*adminv1.TeamsResponse, error) {
	contest, err := a.contestRepo.GetNextContest(ctx)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, errors.New("failed to get teams"))
	}
	var domainTeams []domain.Team
	if contest == nil {
		domainTeams = []domain.Team{}
	} else {
		domainTeams, err = a.teamRepo.GetAll(ctx, contest.Id)
		if err != nil {
			return nil, connect.NewError(connect.CodeInternal, errors.New("failed to get teams"))
		}
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
func (a *adminHandler) SetIp(
	ctx context.Context,
	request *adminv1.SetIpRequest,
) (*emptypb.Empty, error) {
	err := a.teamRepo.SetIp(ctx, request.TeamId, request.Ip)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, errors.New("failed to set ip"))
	}
	return &emptypb.Empty{}, nil
}
