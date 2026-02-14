package persistence

import (
	"context"
	"errors"
	"log/slog"
	"time"

	"entgo.io/ent/dialect/sql"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/station"
)

type EntStationRepository struct {
	client *ent.Client
}

func NewEntStationRepository(client *ent.Client) *EntStationRepository {
	return &EntStationRepository{client: client}
}

// Creates station if not exists, updates ConnectedAt if it does
func (r *EntStationRepository) Upsert(ctx context.Context, ip string) error {
	err := r.client.Station.
		Create().
		SetIP(ip).
		SetConnectedAt(time.Now()).
		OnConflict(sql.ConflictColumns(station.FieldIP)).
		UpdateConnectedAt().
		Exec(ctx)
	if err != nil {
		return err
	}

	return nil
}

// Creates station if not exists, updates ConnectedAt if it does
func (r *EntStationRepository) UpdateDisconnectedAt(ctx context.Context, ip string) error {
	err := r.client.Station.
		Update().
		SetDisconnectedAt(time.Now()).
		Where(station.IPEQ(ip)).
		Exec(ctx)
	if err != nil {
		return err
	}

	return nil
}

func (r *EntStationRepository) GetAll(ctx context.Context) ([]domain.Station, error) {
	eStations, err := r.client.Station.Query().All(ctx)
	if err != nil {
		slog.Error("failed to get all stations", "error", err)
		return []domain.Station{}, errors.New("failed to get all stations")
	}
	stations := []domain.Station{}
	for _, s := range eStations {
		stations = append(stations, domain.Station{
			Id:             s.ID,
			Ip:             s.IP,
			ConnectedAt:    s.ConnectedAt,
			DisconnectedAt: s.DisconnectedAt,
		})
	}
	return stations, nil
}
