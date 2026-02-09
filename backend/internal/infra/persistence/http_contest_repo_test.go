package persistence

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
	"time"
)

func TestHttpContestRepositoryGetNextContest(t *testing.T) {
	now := time.Now()
	contests := []contestDTO{
		{ID: "past", Name: "Past Contest", StartTime: now.Add(-2 * time.Hour), EndTime: now.Add(-1 * time.Hour)},
		{ID: "current", Name: "Current Contest", StartTime: now.Add(-30 * time.Minute), EndTime: now.Add(1 * time.Hour)},
		{ID: "future", Name: "Future Contest", StartTime: now.Add(2 * time.Hour), EndTime: now.Add(3 * time.Hour)},
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v4/contests" {
			json.NewEncoder(w).Encode(contests)
			return
		}
		http.NotFound(w, r)
	}))
	defer srv.Close()

	repo := NewHttpContestRepository(srv.URL, "admin", "password")
	result, err := repo.GetNextContest(context.Background())
	if err != nil {
		t.Fatalf("GetNextContest failed: %v", err)
	}

	if result == nil {
		t.Fatal("expected a contest, got nil")
	}

	// Should return "current" as it's the first upcoming (EndTime > now) sorted by StartTime
	if result.Id != "current" {
		t.Errorf("expected 'current' contest, got %s", result.Id)
	}
}

func TestHttpContestRepositoryGetNextContestNoUpcoming(t *testing.T) {
	now := time.Now()
	contests := []contestDTO{
		{ID: "past1", Name: "Past 1", StartTime: now.Add(-3 * time.Hour), EndTime: now.Add(-2 * time.Hour)},
		{ID: "past2", Name: "Past 2", StartTime: now.Add(-2 * time.Hour), EndTime: now.Add(-1 * time.Hour)},
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v4/contests" {
			json.NewEncoder(w).Encode(contests)
			return
		}
		http.NotFound(w, r)
	}))
	defer srv.Close()

	repo := NewHttpContestRepository(srv.URL, "admin", "password")
	result, err := repo.GetNextContest(context.Background())
	if err != nil {
		t.Fatalf("GetNextContest failed: %v", err)
	}

	if result != nil {
		t.Errorf("expected nil, got %+v", result)
	}
}

func TestHttpContestRepositoryGetNextContestEmpty(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v4/contests" {
			json.NewEncoder(w).Encode([]contestDTO{})
			return
		}
		http.NotFound(w, r)
	}))
	defer srv.Close()

	repo := NewHttpContestRepository(srv.URL, "admin", "password")
	result, err := repo.GetNextContest(context.Background())
	if err != nil {
		t.Fatalf("GetNextContest failed: %v", err)
	}

	if result != nil {
		t.Errorf("expected nil for empty list, got %+v", result)
	}
}

func TestHttpContestRepositoryGetNextContestApiError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer srv.Close()

	repo := NewHttpContestRepository(srv.URL, "admin", "password")
	_, err := repo.GetNextContest(context.Background())
	if err == nil {
		t.Fatal("expected error on API failure")
	}
}

func TestHttpContestRepositoryGetNextContestSortsByStartTime(t *testing.T) {
	now := time.Now()
	contests := []contestDTO{
		{ID: "later", Name: "Later", StartTime: now.Add(3 * time.Hour), EndTime: now.Add(4 * time.Hour)},
		{ID: "sooner", Name: "Sooner", StartTime: now.Add(1 * time.Hour), EndTime: now.Add(2 * time.Hour)},
		{ID: "middle", Name: "Middle", StartTime: now.Add(2 * time.Hour), EndTime: now.Add(3 * time.Hour)},
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v4/contests" {
			json.NewEncoder(w).Encode(contests)
			return
		}
		http.NotFound(w, r)
	}))
	defer srv.Close()

	repo := NewHttpContestRepository(srv.URL, "admin", "password")
	result, err := repo.GetNextContest(context.Background())
	if err != nil {
		t.Fatalf("GetNextContest failed: %v", err)
	}

	if result.Id != "sooner" {
		t.Errorf("expected 'sooner' contest (earliest start), got %s", result.Id)
	}
}
