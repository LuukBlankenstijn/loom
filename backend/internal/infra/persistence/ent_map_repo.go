package persistence

import (
	"context"
	"fmt"
	"log/slog"

	"github.com/google/uuid"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/contestareamap"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/contestmap"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/doorelement"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/tableelement"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/wallelement"
)

type EntMapRepository struct {
	client *ent.Client
}

func NewEntMapRepository(client *ent.Client) *EntMapRepository {
	return &EntMapRepository{client: client}
}

func (r *EntMapRepository) GetAll(ctx context.Context) ([]domain.Map, error) {
	maps, err := r.client.ContestAreaMap.Query().All(ctx)
	if err != nil {
		slog.Error("failed get all maps", slog.Any("error", err))
		return nil, fmt.Errorf("unexpected error")
	}
	domainMaps := []domain.Map{}
	for _, m := range maps {
		domainMaps = append(domainMaps, domain.Map{
			Id:   m.ID,
			Name: m.Name,
		})
	}
	return domainMaps, nil
}

func (r *EntMapRepository) GetMap(ctx context.Context, map_id int) (*domain.FullMap, error) {
	m, err := r.client.ContestAreaMap.Query().
		WithDoors().
		WithTables().
		WithWalls().
		Where(contestareamap.ID(map_id)).
		Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil
		}
		slog.Error("error when getting contestAreaMap", slog.Any("error", err))
		return nil, fmt.Errorf("unexpected error")
	}

	doors := []*domain.Door{}
	walls := []*domain.Wall{}
	tables := []*domain.Table{}
	for _, w := range m.Edges.Walls {
		element := domain.Wall{
			Id:    w.ID,
			Start: domain.NewPosition(w.XStart, w.YStart),
			End:   domain.NewPosition(w.XEnd, w.YEnd),
		}
		walls = append(walls, &element)
	}
	for _, d := range m.Edges.Doors {
		element := domain.Door{
			Id:       d.ID,
			Rotation: domain.Rotation(d.Rotation),
			Position: domain.NewPosition(d.X, d.Y),
		}
		doors = append(doors, &element)
	}
	for _, t := range m.Edges.Tables {
		element := domain.Table{
			Id:       t.ID,
			Rotation: domain.Rotation(t.Rotation),
			Position: domain.NewPosition(t.X, t.Y),
		}
		tables = append(tables, &element)
	}

	return &domain.FullMap{
		Name:   m.Name,
		Id:     m.ID,
		Doors:  doors,
		Tables: tables,
		Walls:  walls,
	}, nil
}

func (r *EntMapRepository) GetByContest(
	ctx context.Context,
	contest_id string,
) (*domain.Map, error) {
	contestMap, err := r.client.ContestMap.Query().Where(contestmap.ContestID(contest_id)).Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) || ent.IsNotSingular(err) {
			return nil, nil
		}
		return nil, fmt.Errorf("unexpected error")
	}
	m, err := r.client.ContestAreaMap.Query().Where(contestareamap.ID(contestMap.MapID)).Only(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil
		}
		return nil, fmt.Errorf("unexpected error")
	}
	return &domain.Map{
		Name: m.Name,
		Id:   m.ID,
	}, nil
}

func (r *EntMapRepository) SetMap(ctx context.Context, map_id int, contest_id string) error {
	err := r.client.ContestMap.Create().
		SetContestID(contest_id).
		SetMapID(map_id).
		OnConflictColumns(contestmap.FieldContestID).
		UpdateMapID().
		Exec(ctx)
	if err != nil {
		slog.Error(
			"failed to set map",
			slog.String("contestId", contest_id),
			slog.Int("mapId", map_id),
			slog.Any("error", err),
		)
		return fmt.Errorf("unexpected error")
	}
	return nil
}

func (r *EntMapRepository) CreateMap(ctx context.Context, name string) (int, error) {
	newMap, err := r.client.ContestAreaMap.Create().SetName(name).Save(ctx)
	if err != nil {
		slog.Error("failed to create new map", slog.Any("error", err))
		return 0, fmt.Errorf("unexpected error")
	}
	return newMap.ID, nil
}

func (r *EntMapRepository) DeleteElements(ctx context.Context, ids *[]uuid.UUID) error {
	return withTx(ctx, r.client, func(tx *ent.Tx) error {
		_, err := tx.WallElement.Delete().Where(wallelement.IDIn(*ids...)).Exec(ctx)
		if err != nil {
			slog.Error("failed to delete walls", slog.Any("error", err))
			return fmt.Errorf("failed to delete walls")
		}
		_, err = tx.DoorElement.Delete().Where(doorelement.IDIn(*ids...)).Exec(ctx)
		if err != nil {
			slog.Error("failed to delete doors", slog.Any("error", err))
			return fmt.Errorf("failed to delete doors")
		}
		_, err = tx.TableElement.Delete().Where(tableelement.IDIn(*ids...)).Exec(ctx)
		if err != nil {
			slog.Error("failed to delete tables", slog.Any("error", err))
			return fmt.Errorf("failed to delete tables")
		}
		return nil
	})
}

func (r *EntMapRepository) UpsertElements(
	ctx context.Context,
	map_id int,
	walls []domain.Wall,
	doors []domain.Door,
	tables []domain.Table,
) error {
	return withTx(ctx, r.client, func(tx *ent.Tx) error {
		wallBuilders := make([]*ent.WallElementCreate, len(walls))
		for i, wall := range walls {
			wallBuilders[i] = tx.WallElement.Create().
				SetID(wall.Id).
				SetMapID(map_id).
				SetXStart(wall.Start.X).
				SetYStart(wall.Start.Y).
				SetXEnd(wall.End.X).
				SetYEnd(wall.End.Y)
		}

		doorBuilders := make([]*ent.DoorElementCreate, len(doors))
		for i, door := range doors {
			doorBuilders[i] = tx.DoorElement.Create().
				SetID(door.Id).
				SetMapID(map_id).
				SetX(door.Position.X).
				SetY(door.Position.Y).
				SetRotation(doorelement.Rotation(door.Rotation))
		}

		tableBuilders := make([]*ent.TableElementCreate, len(tables))
		for i, door := range tables {
			tableBuilders[i] = tx.TableElement.Create().
				SetID(door.Id).
				SetMapID(map_id).
				SetX(door.Position.X).
				SetY(door.Position.Y).
				SetRotation(tableelement.Rotation(door.Rotation))
		}

		err := tx.WallElement.CreateBulk(wallBuilders...).
			OnConflictColumns(wallelement.FieldID).
			UpdateNewValues().
			Exec(ctx)
		if err != nil {
			slog.Error("failed to create walls", slog.Any("error", err))
			return fmt.Errorf("failed creating walls")
		}

		err = tx.DoorElement.CreateBulk(doorBuilders...).
			OnConflictColumns(doorelement.FieldID).
			UpdateNewValues().
			Exec(ctx)
		if err != nil {
			slog.Error("failed to create doors", slog.Any("error", err))
			return fmt.Errorf("failed creating doors")
		}

		err = tx.TableElement.CreateBulk(tableBuilders...).
			OnConflictColumns(tableelement.FieldID).
			UpdateNewValues().
			Exec(ctx)
		if err != nil {
			slog.Error("failed to create tables", slog.Any("error", err))
			return fmt.Errorf("failed creating tables")
		}
		return nil
	})
}

func withTx(ctx context.Context, client *ent.Client, fn func(tx *ent.Tx) error) error {
	tx, err := client.Tx(ctx)
	if err != nil {
		return err
	}
	defer func() {
		if v := recover(); v != nil {
			tx.Rollback()
			panic(v)
		}
	}()
	if err := fn(tx); err != nil {
		if rerr := tx.Rollback(); rerr != nil {
			err = fmt.Errorf("%w: rolling back transaction: %v", err, rerr)
		}
		return err
	}
	return tx.Commit()
}
