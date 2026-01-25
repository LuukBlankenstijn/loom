package persistence

import (
	"context"
	"fmt"
	"sort"
	"time"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
)

type HttpContestRepository struct {
	httpBaseRepository
}

func NewHttpContestRepository(baseURL, username, password string) *HttpContestRepository {
	return &HttpContestRepository{
		httpBaseRepository: newHTTPBaseRepository(baseURL, username, password),
	}
}

type contestDTO struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	StartTime time.Time `json:"start_time"`
	EndTime   time.Time `json:"end_time"`
}

func (r *HttpContestRepository) GetNextContest(ctx context.Context) (*domain.Contest, error) {
	url := fmt.Sprintf("%s/api/v4/contests", r.baseURL)

	var allContests []contestDTO
	if err := r.get(ctx, url, &allContests); err != nil {
		return nil, err
	}

	// 1. Filter: EndTime > time.Now()
	var upcoming []contestDTO
	now := time.Now()
	for _, c := range allContests {
		if c.EndTime.After(now) {
			upcoming = append(upcoming, c)
		}
	}

	if len(upcoming) == 0 {
		return nil, nil
	}

	// 2. Sort: StartTime Ascending
	sort.Slice(upcoming, func(i, j int) bool {
		return upcoming[i].StartTime.Before(upcoming[j].StartTime)
	})

	// 3. Take the first one (Next Contest)
	next := upcoming[0]

	return &domain.Contest{
		Id:        next.ID,
		Name:      next.Name,
		StartTime: next.StartTime,
		EndTime:   next.EndTime,
	}, nil
}
