package admin

import (
	"context"
	"errors"

	"connectrpc.com/connect"
	adminv1 "github.com/LuukBlankenstijn/loom/gen/go/admin/v1"
	"google.golang.org/protobuf/types/known/emptypb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// Gets all stations
func (a *adminHandler) GetStations(
	ctx context.Context,
	empty *emptypb.Empty,
) (*adminv1.StationsResponse, error) {
	domainStations, err := a.stationRepo.GetAll(ctx)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, errors.New("failed to get stations"))
	}
	stations := []*adminv1.Station{}
	for _, s := range domainStations {
		var disconnectedAt timestamppb.Timestamp
		if s.DisconnectedAt != nil {
			disconnectedAt = *timestamppb.New(*s.DisconnectedAt)
		}
		stations = append(stations, &adminv1.Station{
			Id:            int32(s.Id),
			Ip:            s.Ip,
			ConnectedAt:   timestamppb.New(s.ConnectedAt),
			DiconnectedAt: &disconnectedAt,
		})
	}
	return &adminv1.StationsResponse{
		Stations: stations,
	}, nil
}
