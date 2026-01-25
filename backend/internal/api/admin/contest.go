package admin

import (
	"context"
	"errors"

	"connectrpc.com/connect"
	adminv1 "github.com/LuukBlankenstijn/loom/gen/go/admin/v1"
	"google.golang.org/protobuf/types/known/emptypb"
	"google.golang.org/protobuf/types/known/timestamppb"
)

// Gets the contest that is currently active, or the contest that will start next
func (a *adminServer) GetNextContest(ctx context.Context, empty *emptypb.Empty) (*adminv1.Contest, error) {
	nextContest, err := a.contestRepo.GetNextContest(ctx)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, errors.New("failed to get next contest"))
	}
	if nextContest == nil {
		return nil, connect.NewError(connect.CodeNotFound, errors.New("next contest not found"))
	}
	return &adminv1.Contest{
		Id:        nextContest.Id,
		Name:      nextContest.Name,
		StartTime: timestamppb.New(nextContest.StartTime),
		EndTime:   timestamppb.New(nextContest.EndTime),
	}, nil
}

// Sets the wallpaper for some contest
func (a *adminServer) SetWallpaper(ctx context.Context, request *adminv1.UploadImageRequest) (*emptypb.Empty, error) {
	// TODO implement me
	panic("implement me")
}
