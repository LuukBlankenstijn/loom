package domain

import (
	"context"
	"time"
)

type Contest struct {
	Id        string
	Name      string
	StartTime time.Time
	EndTime   time.Time
}

type ContestRepository interface {
	GetNextContest(ctx context.Context) (*Contest, error)
}
