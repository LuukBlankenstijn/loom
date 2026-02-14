package domain

import (
	"context"

	"github.com/google/uuid"
)

type Position struct {
	X int
	Y int
}

type Rotation string

func NewPosition(x, y int) Position {
	return Position{
		X: x,
		Y: y,
	}
}

const (
	Rotation0   Rotation = "0"
	Rotation90  Rotation = "90"
	Rotation180 Rotation = "180"
	Rotation270 Rotation = "270"
)

type Door struct {
	Id       uuid.UUID
	Position Position
	Rotation Rotation
}

type Wall struct {
	Id    uuid.UUID
	Start Position
	End   Position
}

type Table struct {
	Id       uuid.UUID
	Position Position
	Rotation Rotation
}

type Map struct {
	Id   int
	Name string
}

type FullMap struct {
	Id     int
	Name   string
	Doors  []*Door
	Walls  []*Wall
	Tables []*Table
}

type MapRepository interface {
	// Get all Maps
	GetAll(ctx context.Context) ([]Map, error)
	// Gets a map by id
	GetMap(ctx context.Context, map_id int) (*FullMap, error)
	// Gets a map by contest_id
	GetByContest(ctx context.Context, contest_id string) (*Map, error)
	// Sets the map for some context
	SetMap(ctx context.Context, map_id int, contest_id string) error
	// Creates a map, returning the id of the created map
	CreateMap(ctx context.Context, name string) (int, error)
	// Delete Map elements
	DeleteElements(ctx context.Context, ids *[]uuid.UUID) error
	// Updates or creates elements
	UpsertElements(
		ctx context.Context,
		map_id int,
		walls []Wall,
		doors []Door,
		tables []Table,
	) error
}
