package persistence

import (
	"context"
	"testing"

	"github.com/google/uuid"
	_ "github.com/mattn/go-sqlite3"

	"github.com/LuukBlankenstijn/loom/backend/internal/domain"
	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/enttest"
)

func TestEntMapRepositorySetMap(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_set?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	if err := repo.SetMap(ctx, 42, "contest-1"); err != nil {
		t.Fatalf("SetMap failed: %v", err)
	}

	// Verify by querying the database directly
	contestMap, err := client.ContestMap.Query().Only(ctx)
	if err != nil {
		t.Fatalf("failed to query contest map: %v", err)
	}

	if contestMap.ContestID != "contest-1" {
		t.Errorf("expected contest_id 'contest-1', got '%s'", contestMap.ContestID)
	}
	if contestMap.MapID != 42 {
		t.Errorf("expected map_id 42, got %d", contestMap.MapID)
	}
}

func TestEntMapRepositorySetMapUpdatesExisting(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_update?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	// Set initial map
	if err := repo.SetMap(ctx, 42, "contest-1"); err != nil {
		t.Fatalf("SetMap failed: %v", err)
	}

	// Update to a different map
	if err := repo.SetMap(ctx, 99, "contest-1"); err != nil {
		t.Fatalf("SetMap update failed: %v", err)
	}

	// Verify there's still only one entry and it has the updated map_id
	contestMaps, err := client.ContestMap.Query().All(ctx)
	if err != nil {
		t.Fatalf("failed to query contest maps: %v", err)
	}

	if len(contestMaps) != 1 {
		t.Fatalf("expected 1 contest map entry, got %d", len(contestMaps))
	}

	if contestMaps[0].MapID != 99 {
		t.Errorf("expected map_id 99 after update, got %d", contestMaps[0].MapID)
	}
}

func TestEntMapRepositoryMultipleContests(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_multi?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	// Set maps for different contests
	if err := repo.SetMap(ctx, 1, "contest-1"); err != nil {
		t.Fatalf("SetMap contest-1 failed: %v", err)
	}
	if err := repo.SetMap(ctx, 2, "contest-2"); err != nil {
		t.Fatalf("SetMap contest-2 failed: %v", err)
	}

	// Verify both entries exist
	contestMaps, err := client.ContestMap.Query().All(ctx)
	if err != nil {
		t.Fatalf("failed to query contest maps: %v", err)
	}

	if len(contestMaps) != 2 {
		t.Fatalf("expected 2 contest map entries, got %d", len(contestMaps))
	}

	// Create a map for easy lookup
	mapByContest := make(map[string]int)
	for _, cm := range contestMaps {
		mapByContest[cm.ContestID] = cm.MapID
	}

	if mapByContest["contest-1"] != 1 {
		t.Errorf("contest-1: expected map_id 1, got %d", mapByContest["contest-1"])
	}
	if mapByContest["contest-2"] != 2 {
		t.Errorf("contest-2: expected map_id 2, got %d", mapByContest["contest-2"])
	}
}

func TestEntMapRepositoryGetAll(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_getall?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	// Initially should be empty
	maps, err := repo.GetAll(ctx)
	if err != nil {
		t.Fatalf("GetAll failed: %v", err)
	}
	if len(maps) != 0 {
		t.Errorf("expected 0 maps, got %d", len(maps))
	}

	// Create some maps
	id1, err := repo.CreateMap(ctx, "Map 1")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}
	id2, err := repo.CreateMap(ctx, "Map 2")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	// Should now return both maps
	maps, err = repo.GetAll(ctx)
	if err != nil {
		t.Fatalf("GetAll failed: %v", err)
	}
	if len(maps) != 2 {
		t.Fatalf("expected 2 maps, got %d", len(maps))
	}

	// Verify map contents
	mapById := make(map[int]domain.Map)
	for _, m := range maps {
		mapById[m.Id] = m
	}

	if mapById[id1].Name != "Map 1" {
		t.Errorf("expected name 'Map 1', got '%s'", mapById[id1].Name)
	}
	if mapById[id2].Name != "Map 2" {
		t.Errorf("expected name 'Map 2', got '%s'", mapById[id2].Name)
	}
}

func TestEntMapRepositoryCreateMap(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_create?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	id, err := repo.CreateMap(ctx, "Test Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	if id == 0 {
		t.Error("expected non-zero id")
	}

	// Verify by querying directly
	areaMap, err := client.ContestAreaMap.Get(ctx, id)
	if err != nil {
		t.Fatalf("failed to get map: %v", err)
	}

	if areaMap.Name != "Test Map" {
		t.Errorf("expected name 'Test Map', got '%s'", areaMap.Name)
	}
}

func TestEntMapRepositoryGetMap(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_getmap?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	// Create a map
	mapId, err := repo.CreateMap(ctx, "Test Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	// Add some elements
	wallId := uuid.New()
	doorId := uuid.New()
	tableId := uuid.New()

	walls := []domain.Wall{
		{Id: wallId, Start: domain.Position{X: 0, Y: 0}, End: domain.Position{X: 100, Y: 0}},
	}
	doors := []domain.Door{
		{Id: doorId, Position: domain.Position{X: 50, Y: 50}, Rotation: domain.Rotation0},
	}
	tables := []domain.Table{
		{Id: tableId, Position: domain.Position{X: 75, Y: 75}, Rotation: domain.Rotation90},
	}

	err = repo.UpsertElements(ctx, mapId, walls, doors, tables)
	if err != nil {
		t.Fatalf("UpsertElements failed: %v", err)
	}

	// Get the full map
	fullMap, err := repo.GetMap(ctx, mapId)
	if err != nil {
		t.Fatalf("GetMap failed: %v", err)
	}

	if fullMap == nil {
		t.Fatal("expected fullMap, got nil")
	}

	if fullMap.Name != "Test Map" {
		t.Errorf("expected name 'Test Map', got '%s'", fullMap.Name)
	}
	if fullMap.Id != mapId {
		t.Errorf("expected id %d, got %d", mapId, fullMap.Id)
	}
	if len(fullMap.Walls) != 1 {
		t.Errorf("expected 1 wall, got %d", len(fullMap.Walls))
	}
	if len(fullMap.Doors) != 1 {
		t.Errorf("expected 1 door, got %d", len(fullMap.Doors))
	}
	if len(fullMap.Tables) != 1 {
		t.Errorf("expected 1 table, got %d", len(fullMap.Tables))
	}

	// Verify wall details
	if fullMap.Walls[0].Id != wallId {
		t.Errorf("wall id mismatch")
	}
	if fullMap.Walls[0].Start.X != 0 || fullMap.Walls[0].Start.Y != 0 {
		t.Errorf("wall start position mismatch")
	}
	if fullMap.Walls[0].End.X != 100 || fullMap.Walls[0].End.Y != 0 {
		t.Errorf("wall end position mismatch")
	}

	// Verify door details
	if fullMap.Doors[0].Id != doorId {
		t.Errorf("door id mismatch")
	}
	if fullMap.Doors[0].Position.X != 50 || fullMap.Doors[0].Position.Y != 50 {
		t.Errorf("door position mismatch")
	}
	if fullMap.Doors[0].Rotation != domain.Rotation0 {
		t.Errorf("door rotation mismatch")
	}

	// Verify table details
	if fullMap.Tables[0].Id != tableId {
		t.Errorf("table id mismatch")
	}
	if fullMap.Tables[0].Position.X != 75 || fullMap.Tables[0].Position.Y != 75 {
		t.Errorf("table position mismatch")
	}
	if fullMap.Tables[0].Rotation != domain.Rotation90 {
		t.Errorf("table rotation mismatch")
	}
}

func TestEntMapRepositoryGetMapNotFound(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_getmap_notfound?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	fullMap, err := repo.GetMap(ctx, 999)
	if err != nil {
		t.Fatalf("GetMap failed: %v", err)
	}

	if fullMap != nil {
		t.Errorf("expected nil for nonexistent map, got %v", fullMap)
	}
}

func TestEntMapRepositoryGetByContest(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_getbycontest?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	// Create a map and associate it with a contest
	mapId, err := repo.CreateMap(ctx, "Contest Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	err = repo.SetMap(ctx, mapId, "contest-1")
	if err != nil {
		t.Fatalf("SetMap failed: %v", err)
	}

	// Get by contest
	m, err := repo.GetByContest(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetByContest failed: %v", err)
	}

	if m == nil {
		t.Fatal("expected map, got nil")
	}

	if m.Id != mapId {
		t.Errorf("expected id %d, got %d", mapId, m.Id)
	}
	if m.Name != "Contest Map" {
		t.Errorf("expected name 'Contest Map', got '%s'", m.Name)
	}
}

func TestEntMapRepositoryGetByContestNotFound(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_getbycontest_notfound?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	m, err := repo.GetByContest(ctx, "nonexistent")
	if err != nil {
		t.Fatalf("GetByContest failed: %v", err)
	}

	if m != nil {
		t.Errorf("expected nil for nonexistent contest, got %v", m)
	}
}

func TestEntMapRepositoryUpsertElements(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_upsert?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	mapId, err := repo.CreateMap(ctx, "Test Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	wallId := uuid.New()
	doorId := uuid.New()
	tableId := uuid.New()

	walls := []domain.Wall{
		{Id: wallId, Start: domain.Position{X: 0, Y: 0}, End: domain.Position{X: 100, Y: 0}},
	}
	doors := []domain.Door{
		{Id: doorId, Position: domain.Position{X: 50, Y: 50}, Rotation: domain.Rotation0},
	}
	tables := []domain.Table{
		{Id: tableId, Position: domain.Position{X: 75, Y: 75}, Rotation: domain.Rotation90},
	}

	err = repo.UpsertElements(ctx, mapId, walls, doors, tables)
	if err != nil {
		t.Fatalf("UpsertElements failed: %v", err)
	}

	// Verify walls
	wallElements, err := client.WallElement.Query().All(ctx)
	if err != nil {
		t.Fatalf("failed to query walls: %v", err)
	}
	if len(wallElements) != 1 {
		t.Fatalf("expected 1 wall, got %d", len(wallElements))
	}
	if wallElements[0].ID != wallId {
		t.Errorf("wall id mismatch")
	}

	// Verify doors
	doorElements, err := client.DoorElement.Query().All(ctx)
	if err != nil {
		t.Fatalf("failed to query doors: %v", err)
	}
	if len(doorElements) != 1 {
		t.Fatalf("expected 1 door, got %d", len(doorElements))
	}

	// Verify tables
	tableElements, err := client.TableElement.Query().All(ctx)
	if err != nil {
		t.Fatalf("failed to query tables: %v", err)
	}
	if len(tableElements) != 1 {
		t.Fatalf("expected 1 table, got %d", len(tableElements))
	}
}

func TestEntMapRepositoryUpsertElementsUpdatesExisting(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_upsert_update?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	mapId, err := repo.CreateMap(ctx, "Test Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	wallId := uuid.New()

	// Initial insert
	walls := []domain.Wall{
		{Id: wallId, Start: domain.Position{X: 0, Y: 0}, End: domain.Position{X: 100, Y: 0}},
	}
	err = repo.UpsertElements(ctx, mapId, walls, nil, nil)
	if err != nil {
		t.Fatalf("UpsertElements failed: %v", err)
	}

	// Update with same ID but different coordinates
	walls = []domain.Wall{
		{Id: wallId, Start: domain.Position{X: 10, Y: 20}, End: domain.Position{X: 200, Y: 30}},
	}
	err = repo.UpsertElements(ctx, mapId, walls, nil, nil)
	if err != nil {
		t.Fatalf("UpsertElements update failed: %v", err)
	}

	// Verify still only one wall with updated coordinates
	wallElements, err := client.WallElement.Query().All(ctx)
	if err != nil {
		t.Fatalf("failed to query walls: %v", err)
	}
	if len(wallElements) != 1 {
		t.Fatalf("expected 1 wall, got %d", len(wallElements))
	}
	if wallElements[0].XStart != 10 || wallElements[0].YStart != 20 || wallElements[0].XEnd != 200 || wallElements[0].YEnd != 30 {
		t.Errorf("wall coordinates not updated")
	}
}

func TestEntMapRepositoryDeleteElements(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_delete?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	mapId, err := repo.CreateMap(ctx, "Test Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	wallId := uuid.New()
	doorId := uuid.New()
	tableId := uuid.New()

	walls := []domain.Wall{{Id: wallId, Start: domain.Position{X: 0, Y: 0}, End: domain.Position{X: 100, Y: 0}}}
	doors := []domain.Door{{Id: doorId, Position: domain.Position{X: 50, Y: 50}, Rotation: domain.Rotation0}}
	tables := []domain.Table{{Id: tableId, Position: domain.Position{X: 75, Y: 75}, Rotation: domain.Rotation0}}

	err = repo.UpsertElements(ctx, mapId, walls, doors, tables)
	if err != nil {
		t.Fatalf("UpsertElements failed: %v", err)
	}

	// Delete all elements
	ids := []uuid.UUID{wallId, doorId, tableId}
	err = repo.DeleteElements(ctx, &ids)
	if err != nil {
		t.Fatalf("DeleteElements failed: %v", err)
	}

	// Verify all elements are gone
	wallCount, _ := client.WallElement.Query().Count(ctx)
	doorCount, _ := client.DoorElement.Query().Count(ctx)
	tableCount, _ := client.TableElement.Query().Count(ctx)

	if wallCount != 0 {
		t.Errorf("expected 0 walls, got %d", wallCount)
	}
	if doorCount != 0 {
		t.Errorf("expected 0 doors, got %d", doorCount)
	}
	if tableCount != 0 {
		t.Errorf("expected 0 tables, got %d", tableCount)
	}
}

func TestEntMapRepositoryDeleteElementsPartial(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_delete_partial?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	mapId, err := repo.CreateMap(ctx, "Test Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	wallId1 := uuid.New()
	wallId2 := uuid.New()

	walls := []domain.Wall{
		{Id: wallId1, Start: domain.Position{X: 0, Y: 0}, End: domain.Position{X: 100, Y: 0}},
		{Id: wallId2, Start: domain.Position{X: 0, Y: 50}, End: domain.Position{X: 100, Y: 50}},
	}
	err = repo.UpsertElements(ctx, mapId, walls, nil, nil)
	if err != nil {
		t.Fatalf("UpsertElements failed: %v", err)
	}

	// Delete only one wall
	ids := []uuid.UUID{wallId1}
	err = repo.DeleteElements(ctx, &ids)
	if err != nil {
		t.Fatalf("DeleteElements failed: %v", err)
	}

	// Verify only one wall remains
	wallElements, err := client.WallElement.Query().All(ctx)
	if err != nil {
		t.Fatalf("failed to query walls: %v", err)
	}
	if len(wallElements) != 1 {
		t.Fatalf("expected 1 wall remaining, got %d", len(wallElements))
	}
	if wallElements[0].ID != wallId2 {
		t.Errorf("wrong wall remained")
	}
}

func TestEntMapRepositoryUpsertEmptySlices(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_map_upsert_empty?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntMapRepository(client)

	mapId, err := repo.CreateMap(ctx, "Test Map")
	if err != nil {
		t.Fatalf("CreateMap failed: %v", err)
	}

	// Should not error with empty slices
	err = repo.UpsertElements(ctx, mapId, []domain.Wall{}, []domain.Door{}, []domain.Table{})
	if err != nil {
		t.Fatalf("UpsertElements with empty slices failed: %v", err)
	}

	// Should also work with nil slices
	err = repo.UpsertElements(ctx, mapId, nil, nil, nil)
	if err != nil {
		t.Fatalf("UpsertElements with nil slices failed: %v", err)
	}
}
