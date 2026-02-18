package persistence

import (
	"context"
	"testing"

	_ "github.com/mattn/go-sqlite3"

	"github.com/LuukBlankenstijn/loom/backend/internal/infra/ent/enttest"
)

func TestEntWallpaperRepositorySetAndGet(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_setget?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	imageData := []byte("fake-image-data")
	if err := repo.SetWallpaperData(ctx, "contest-1", imageData, "image/png"); err != nil {
		t.Fatalf("SetWallpaper failed: %v", err)
	}

	result, err := repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper failed: %v", err)
	}

	if result == nil {
		t.Fatal("expected wallpaper data, got nil")
	}

	if string(result.Data) != "fake-image-data" {
		t.Errorf("expected 'fake-image-data', got '%s'", string(result.Data))
	}

	if result.MimeType != "image/png" {
		t.Errorf("expected mime type 'image/png', got '%s'", result.MimeType)
	}

	if result.ContestId != "contest-1" {
		t.Errorf("expected contest id 'contest-1', got '%s'", result.ContestId)
	}
}

func TestEntWallpaperRepositoryGetNotFound(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(
		t,
		"sqlite3",
		"file:ent_wallpaper_notfound?mode=memory&cache=shared&_fk=1",
	)
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

func TestEntWallpaperRepositoryDeleteWallpaper(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_delete?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// First set a wallpaper
	imageData := []byte("to-be-deleted")
	if err := repo.SetWallpaperData(ctx, "contest-1", imageData, "image/jpeg"); err != nil {
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

	// Delete the wallpaper
	if err := repo.DeleteWallpaper(ctx, "contest-1"); err != nil {
		t.Fatalf("DeleteWallpaper failed: %v", err)
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

func TestEntWallpaperRepositoryDeleteNonexistent(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(
		t,
		"sqlite3",
		"file:ent_wallpaper_delete_nonexistent?mode=memory&cache=shared&_fk=1",
	)
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// Deleting a nonexistent wallpaper should not error
	if err := repo.DeleteWallpaper(ctx, "nonexistent"); err != nil {
		t.Fatalf("DeleteWallpaper on nonexistent should not fail: %v", err)
	}
}

func TestEntWallpaperRepositorySetUpdatesExisting(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_update?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// Set initial wallpaper
	imageData1 := []byte("initial-data")
	if err := repo.SetWallpaperData(ctx, "contest-1", imageData1, "image/png"); err != nil {
		t.Fatalf("SetWallpaper failed: %v", err)
	}

	// Update with new data and mime type
	imageData2 := []byte("updated-data")
	if err := repo.SetWallpaperData(ctx, "contest-1", imageData2, "image/jpeg"); err != nil {
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

	if string(result.Data) != "updated-data" {
		t.Errorf("expected 'updated-data', got '%s'", string(result.Data))
	}

	if result.MimeType != "image/jpeg" {
		t.Errorf("expected mime type 'image/jpeg', got '%s'", result.MimeType)
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

	if err := repo.SetWallpaperData(ctx, "contest-1", data1, "image/png"); err != nil {
		t.Fatalf("SetWallpaper contest-1 failed: %v", err)
	}
	if err := repo.SetWallpaperData(ctx, "contest-2", data2, "image/jpeg"); err != nil {
		t.Fatalf("SetWallpaper contest-2 failed: %v", err)
	}

	// Verify each contest has its own wallpaper
	result1, err := repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper contest-1 failed: %v", err)
	}
	if string(result1.Data) != "contest-1-wallpaper" {
		t.Errorf("contest-1: expected 'contest-1-wallpaper', got '%s'", string(result1.Data))
	}
	if result1.MimeType != "image/png" {
		t.Errorf("contest-1: expected mime type 'image/png', got '%s'", result1.MimeType)
	}

	result2, err := repo.GetWallpaper(ctx, "contest-2")
	if err != nil {
		t.Fatalf("GetWallpaper contest-2 failed: %v", err)
	}
	if string(result2.Data) != "contest-2-wallpaper" {
		t.Errorf("contest-2: expected 'contest-2-wallpaper', got '%s'", string(result2.Data))
	}
	if result2.MimeType != "image/jpeg" {
		t.Errorf("contest-2: expected mime type 'image/jpeg', got '%s'", result2.MimeType)
	}
}

func TestEntWallpaperRepositorySetTextColor(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_textcolor?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// First create a wallpaper
	imageData := []byte("test-image")
	if err := repo.SetWallpaperData(ctx, "contest-1", imageData, "image/png"); err != nil {
		t.Fatalf("SetWallpaperData failed: %v", err)
	}

	// Verify default color
	result, err := repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper failed: %v", err)
	}
	if result.TextColor != "#ffffff" {
		t.Errorf("expected default text color '#ffffff', got '%s'", result.TextColor)
	}

	// Set a new text color
	if err := repo.SetWallpaperTextColor(ctx, "contest-1", "#000000"); err != nil {
		t.Fatalf("SetWallpaperTextColor failed: %v", err)
	}

	// Verify the color was updated
	result, err = repo.GetWallpaper(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetWallpaper failed: %v", err)
	}
	if result.TextColor != "#000000" {
		t.Errorf("expected text color '#000000', got '%s'", result.TextColor)
	}
}

func TestEntWallpaperRepositoryGetLastUpdated(t *testing.T) {
	ctx := context.Background()
	client := enttest.Open(t, "sqlite3", "file:ent_wallpaper_lastupdated?mode=memory&cache=shared&_fk=1")
	t.Cleanup(func() { _ = client.Close() })

	repo := NewEntWallpaperRepository(client)

	// GetLastUpdated on nonexistent should return current time (not error)
	_, err := repo.GetLastUpdated(ctx, "nonexistent")
	if err != nil {
		t.Fatalf("GetLastUpdated on nonexistent should not error: %v", err)
	}

	// Create a wallpaper
	imageData := []byte("test-image")
	if err := repo.SetWallpaperData(ctx, "contest-1", imageData, "image/png"); err != nil {
		t.Fatalf("SetWallpaperData failed: %v", err)
	}

	// Get the last updated time
	updatedAt, err := repo.GetLastUpdated(ctx, "contest-1")
	if err != nil {
		t.Fatalf("GetLastUpdated failed: %v", err)
	}

	// Verify it's a valid timestamp (not zero)
	if updatedAt.IsZero() {
		t.Error("expected non-zero updated_at timestamp")
	}
}
