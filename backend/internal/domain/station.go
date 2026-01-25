package domain

import (
	"context"
	"errors"
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

type ConfigUpdatedEvent struct{}

var ErrAlreadyRegistered = errors.New("station already registered")

type Hub interface {
	Register(stationIp string) (<-chan ConfigUpdatedEvent, func(), error)

	// Notify notify station, optionally filtering by ip
	// This method does not guarantee delivery, if some channel is full the message is dropped
	Notify(event ConfigUpdatedEvent, stationIp ...string)
}
