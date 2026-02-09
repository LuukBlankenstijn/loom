package persistence

import (
	"context"
	"testing"

	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/enttest"
	_ "github.com/mattn/go-sqlite3"
)

func TestEntWallpaperRepositorySetAndGet(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_setget?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	imageData := []byte("fake-image-data")
	if err := repo.SetWallpaper(ctx, "contest-1", &imageData); err != nil {
		t.Fatalf("SetWallpaper failed: %v", err)
	}

	result, err := repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper failed: %v", err)
	}

	if result == nil {
		t.Fatal("expected wallpaper data, got nil")
	}

	if string(*result) != "fake-image-data" {
		t.Errorf("expected 'fake-image-data', got '%s'", string(*result))
	}
}

func TestEntWallpaperRepositoryGetNotFound(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_notfound?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	result, err := repo.GetWallpaper(ctx, "nonexistent")
	if err != nil {
		t.Fatalf("GetWallpaper failed: %v", err)
	}

	if result != nil {
		t.Errorf("expected nil for nonexistent wallpaper, got %v", result)
	}
}

func TestEntWallpaperRepositorySetNilDeletes(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_delete?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// First set a wallpaper
	imageData := []byte("to-be-deleted")
	if err := repo.SetWallpaper(ctx, "contest-1", &imageData); err != nil {
		t.Fatalf("SetWallpaper failed: %v", err)
	}

	// Verify it exists
	result, err := repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper failed: %v", err)
	}
	if result == nil {
		t.Fatal("wallpaper should exist before deletion")
	}

	// Delete by setting nil
	if err := repo.SetWallpaper(ctx, "contest-1", nil); err != nil {
		t.Fatalf("SetWallpaper(nil) failed: %v", err)
	}

	// Verify it's gone
	result, err = repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper failed after delete: %v", err)
	}
	if result != nil {
		t.Error("wallpaper should be deleted")
	}
}

func TestEntWallpaperRepositorySetUpdatesExisting(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_update?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// Set initial wallpaper
	imageData1 := []byte("initial-data")
	if err := repo.SetWallpaper(ctx, "contest-1", &imageData1); err != nil {
		t.Fatalf("SetWallpaper failed: %v", err)
	}

	// Update with new data
	imageData2 := []byte("updated-data")
	if err := repo.SetWallpaper(ctx, "contest-1", &imageData2); err != nil {
		t.Fatalf("SetWallpaper update failed: %v", err)
	}

	// Verify update
	result, err := repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper failed: %v", err)
	}

	if result == nil {
		t.Fatal("expected wallpaper data, got nil")
	}

	if string(*result) != "updated-data" {
		t.Errorf("expected 'updated-data', got '%s'", string(*result))
	}
}

func TestEntWallpaperRepositoryMultipleContests(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_multi?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// Set wallpapers for different contests
	data1 := []byte("contest-1-wallpaper")
	data2 := []byte("contest-2-wallpaper")

	if err := repo.SetWallpaper(ctx, "contest-1", &data1); err != nil {
		t.Fatalf("SetWallpaper contest-1 failed: %v", err)
	}
	if err := repo.SetWallpaper(ctx, "contest-2", &data2); err != nil {
		t.Fatalf("SetWallpaper contest-2 failed: %v", err)
	}

	// Verify each contest has its own wallpaper
	result1, err := repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper contest-1 failed: %v", err)
	}
	if string(*result1) != "contest-1-wallpaper" {
		t.Errorf("contest-1: expected 'contest-1-wallpaper', got '%s'", string(*result1))
	}

	result2, err := repo.GetWallpaper(ctx, "contest-2")
	if err != nil {
		t.Fatalf("GetWallpaper contest-2 failed: %v", err)
	}
	if string(*result2) != "contest-2-wallpaper" {
		t.Errorf("contest-2: expected 'contest-2-wallpaper', got '%s'", string(*result2))
	}
}
