package admin

import (
	"context"
	"fmt"

	"connectrpc.com/connect"
	adminv1 "github.com/LuukBlankenstijn/loom/gen/go/admin/v1"
	"github.com/google/uuid"
	"google.golang.org/protobuf/types/known/emptypb"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

func (m *adminHandler) GetAllMaps(
	ctx context.Context,
	request *emptypb.Empty,
) (*adminv1.GetAllMapsResponse, error) {
	maps, err := m.mapRepo.GetAll(ctx)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	apiMaps := []*adminv1.Map{}
	for _, m := range maps {
		apiMaps = append(apiMaps, domainToApi(m))
	}
	return &adminv1.GetAllMapsResponse{
		Maps: apiMaps,
	}, nil
}

func (a *adminHandler) GetMap(
	ctx context.Context,
	request *adminv1.GetMapRequest,
) (*adminv1.MapResponse, error) {
	m, err := a.mapRepo.GetMap(ctx, int(request.Id))
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	if m == nil {
		return nil, connect.NewError(connect.CodeNotFound, fmt.Errorf("Map not found"))
	}
	elements := []*adminv1.Element{}
	for _, w := range m.Walls {
		element := adminv1.Element{
			Element: &adminv1.Element_Wall{
				Wall: &adminv1.Wall{
					Id: w.Id.String(),
					Start: &adminv1.Location{
						X: int32(w.Start.X),
						Y: int32(w.Start.Y),
					},
					End: &adminv1.Location{
						X: int32(w.End.X),
						Y: int32(w.End.Y),
					},
				},
			},
		}
		elements = append(elements, &element)
	}
	for _, d := range m.Doors {
		element := adminv1.Element{
			Element: &adminv1.Element_Door{
				Door: &adminv1.Door{
					Id:       d.Id.String(),
					Rotation: rotatationToApi(d.Rotation),
					Location: &adminv1.Location{
						X: int32(d.Position.X),
						Y: int32(d.Position.Y),
					},
				},
			},
		}
		elements = append(elements, &element)
	}
	for _, t := range m.Tables {
		element := adminv1.Element{
			Element: &adminv1.Element_Table{
				Table: &adminv1.Table{
					Id:       t.Id.String(),
					Rotation: rotatationToApi(t.Rotation),
					Location: &adminv1.Location{
						X: int32(t.Position.X),
						Y: int32(t.Position.Y),
					},
				},
			},
		}
		elements = append(elements, &element)
	}
	return &adminv1.MapResponse{
		Map: &adminv1.Map{
			Name: m.Name,
			Id:   int32(m.Id),
		},
		Elements: elements,
	}, nil
}

func (m *adminHandler) CreateMap(
	ctx context.Context,
	request *adminv1.CreateMapRequest,
) (*adminv1.MapResponse, error) {
	id, err := m.mapRepo.CreateMap(ctx, request.Name)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	return &adminv1.MapResponse{
		Map: &adminv1.Map{
			Id:   int32(id),
			Name: request.Name,
		},
	}, nil
}

func (m *adminHandler) UpdateMap(
	ctx context.Context,
	request *adminv1.UpdateMapRequest,
) (*emptypb.Empty, error) {
	println(fmt.Sprintf("%+v", request))
	deletedIds := []uuid.UUID{}
	for _, d := range request.Deleted {
		if id, err := uuid.Parse(d); err != nil {
			deletedIds = append(deletedIds, id)
		}
	}
	err := m.mapRepo.DeleteElements(ctx, &deletedIds)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}
	doors := []domain.Door{}
	walls := []domain.Wall{}
	tables := []domain.Table{}
	for _, u := range request.Updated {
		switch updated := u.Element.(type) {
		case *adminv1.Element_Door:
			if id, err := uuid.Parse(updated.Door.Id); err == nil {
				doors = append(doors, domain.Door{
					Id:       id,
					Position: domain.NewPosition(int(updated.Door.Location.X), int(updated.Door.Location.Y)),
					Rotation: rotationFromApi(updated.Door.Rotation),
				})
			}
		case *adminv1.Element_Wall:
			if id, err := uuid.Parse(updated.Wall.Id); err == nil {
				walls = append(walls, domain.Wall{
					Id:    id,
					Start: domain.NewPosition(int(updated.Wall.Start.X), int(updated.Wall.Start.Y)),
					End:   domain.NewPosition(int(updated.Wall.End.X), int(updated.Wall.End.Y)),
				})
			}
		case *adminv1.Element_Table:
			if id, err := uuid.Parse(updated.Table.Id); err == nil {
				tables = append(tables, domain.Table{
					Id:       id,
					Position: domain.NewPosition(int(updated.Table.Location.X), int(updated.Table.Location.Y)),
					Rotation: rotationFromApi(updated.Table.Rotation),
				})
			}
		}
	}
	println(fmt.Sprintf("%+v, %+v, %+v", walls, doors, tables))
	err = m.mapRepo.UpsertElements(ctx, int(request.Id), walls, doors, tables)
	if err != nil {
		return nil, connect.NewError(connect.CodeInternal, err)
	}

	return &emptypb.Empty{}, nil
}

func domainToApi(original domain.Map) *adminv1.Map {
	return &adminv1.Map{
		Name: original.Name,
		Id:   int32(original.Id),
	}
}

func rotationFromApi(original adminv1.Rotation) domain.Rotation {
	switch original {
	case adminv1.Rotation_ROTATION_0:
		return domain.Rotation0
	case adminv1.Rotation_ROTATION_90:
		return domain.Rotation90
	case adminv1.Rotation_ROTATION_180:
		return domain.Rotation180
	case adminv1.Rotation_ROTATION_270:
		return domain.Rotation270
	}
	return domain.Rotation0
}

func rotatationToApi(original domain.Rotation) adminv1.Rotation {
	switch original {
	case domain.Rotation0:
		return adminv1.Rotation_ROTATION_0
	case domain.Rotation90:
		return adminv1.Rotation_ROTATION_90
	case domain.Rotation180:
		return adminv1.Rotation_ROTATION_180
	case domain.Rotation270:
		return adminv1.Rotation_ROTATION_270
	}
	return adminv1.Rotation_ROTATION_0
}
