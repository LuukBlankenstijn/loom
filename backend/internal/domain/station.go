package domain

import (
	"context"
	"time"
)

type Station struct {
	Id             int
	Ip             string
	ConnectedAt    time.Time
	DisconnectedAt *time.Time
}

type StationRepository interface {
	GetAll(ctx context.Context) ([]Station, error)
	Upsert(ctx context.Context, ip string) error
	UpdateDisconnectedAt(ctx context.Context, ip string) error
}
