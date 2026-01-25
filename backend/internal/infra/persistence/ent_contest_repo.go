package persistence

import (
	"context"
	"errors"
	"log/slog"
	"time"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/contest"
)

type EntContestRepository struct {
	client *ent.Client
}

func NewEntContestRepository(client *ent.Client) *EntContestRepository {
	return &EntContestRepository{client: client}
}

func (r *EntContestRepository) GetNextContest(ctx context.Context) (*domain.Contest, error) {
	eContest, err := r.
		client.
		Contest.
		Query().
		Where(
			contest.EndTimeGT(time.Now()),
		).
		Order(
			ent.Asc(contest.FieldStartTime),
		).
		First(ctx)
	if err != nil {
		if ent.IsNotFound(err) {
			return nil, nil
		}
		slog.Error("failed to get next contest", "error", err)
		return nil, errors.New("failed to get next contest")
	}
	return &domain.Contest{
		Id:        eContest.ID,
		Name:      eContest.Name,
		StartTime: eContest.StartTime,
		EndTime:   eContest.EndTime,
	}, nil
}
