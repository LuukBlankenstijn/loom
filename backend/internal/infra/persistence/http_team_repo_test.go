package persistence

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"
)

func TestHttpTeamRepositoryGetAll(t *testing.T) {
	teams := []apiTeam{
		{ID: "team-1", Name: "Team 1"},
		{ID: "team-2", Name: "Team 2"},
	}
	users := []apiUser{
		{ID: "user-1", TeamID: "team-1", IP: "10.0.0.1", Name: "User 1", Username: "user1"},
		{ID: "user-2", TeamID: "team-2", IP: "", Name: "User 2", Username: "user2"},
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch r.URL.Path {
		case "/api/v4/contests/contest-1/teams":
			json.NewEncoder(w).Encode(teams)
		case "/api/v4/users":
			json.NewEncoder(w).Encode(users)
		default:
			http.NotFound(w, r)
		}
	}))
	defer srv.Close()

	repo := NewHttpTeamRepository(srv.URL, "admin", "password")
	result, err := repo.GetAll(context.Background(), "contest-1")
	if err != nil {
		t.Fatalf("GetAll failed: %v", err)
	}

	if len(result) != 2 {
		t.Fatalf("expected 2 teams, got %d", len(result))
	}

	// Team 1 should have IP
	if result[0].Id != "team-1" || result[0].Ip == nil || *result[0].Ip != "10.0.0.1" {
		t.Errorf("team-1 IP mismatch: got %+v", result[0])
	}

	// Team 2 should have no IP
	if result[1].Id != "team-2" || result[1].Ip != nil {
		t.Errorf("team-2 should have no IP: got %+v", result[1])
	}
}

func TestHttpTeamRepositoryGetAllApiError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusInternalServerError)
	}))
	defer srv.Close()

	repo := NewHttpTeamRepository(srv.URL, "admin", "password")
	_, err := repo.GetAll(context.Background(), "contest-1")
	if err == nil {
		t.Fatal("expected error on API failure")
	}
}

func TestHttpTeamRepositorySetIp(t *testing.T) {
	users := []apiUser{
		{
			ID:       "user-1",
			TeamID:   "team-1",
			IP:       "",
			Name:     "User 1",
			Username: "user1",
			Email:    "u1@test.com",
			Enabled:  true,
		},
	}

	var putCalled bool
	var putBody string

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch {
		case r.URL.Path == "/api/v4/users" && r.Method == http.MethodGet:
			json.NewEncoder(w).Encode(users)
		case r.URL.Path == "/api/v4/users/user-1" && r.Method == http.MethodPut:
			putCalled = true
			buf := make([]byte, 1024)
			n, _ := r.Body.Read(buf)
			putBody = string(buf[:n])
			w.WriteHeader(http.StatusOK)
		default:
			http.NotFound(w, r)
		}
	}))
	defer srv.Close()

	repo := NewHttpTeamRepository(srv.URL, "admin", "password")
	ip := "192.168.1.1"
	err := repo.SetIp(context.Background(), "team-1", &ip)
	if err != nil {
		t.Fatalf("SetIp failed: %v", err)
	}

	if !putCalled {
		t.Fatal("PUT request was not made")
	}

	if putBody == "" {
		t.Fatal("PUT body was empty")
	}
}

func TestHttpTeamRepositorySetIpAlreadyUsed(t *testing.T) {
	users := []apiUser{
		{ID: "user-1", TeamID: "team-1", IP: "10.0.0.1", Name: "User 1", Username: "user1"},
		{ID: "user-2", TeamID: "team-2", IP: "", Name: "User 2", Username: "user2"},
	}

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if r.URL.Path == "/api/v4/users" && r.Method == http.MethodGet {
			json.NewEncoder(w).Encode(users)
			return
		}
		http.NotFound(w, r)
	}))
	defer srv.Close()

	repo := NewHttpTeamRepository(srv.URL, "admin", "password")
	ip := "10.0.0.1" // Already used by team-1
	err := repo.SetIp(context.Background(), "team-2", &ip)
	if err == nil {
		t.Fatal("expected error when IP is already used")
	}
}

func TestHttpTeamRepositorySetIpNil(t *testing.T) {
	users := []apiUser{
		{
			ID:       "user-1",
			TeamID:   "team-1",
			IP:       "10.0.0.1",
			Name:     "User 1",
			Username: "user1",
			Email:    "u1@test.com",
			Enabled:  true,
		},
	}

	var putCalled bool

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch {
		case r.URL.Path == "/api/v4/users" && r.Method == http.MethodGet:
			json.NewEncoder(w).Encode(users)
		case r.URL.Path == "/api/v4/users/user-1" && r.Method == http.MethodPut:
			putCalled = true
			w.WriteHeader(http.StatusOK)
		default:
			http.NotFound(w, r)
		}
	}))
	defer srv.Close()

	repo := NewHttpTeamRepository(srv.URL, "admin", "password")
	err := repo.SetIp(context.Background(), "team-1", nil)
	if err != nil {
		t.Fatalf("SetIp(nil) failed: %v", err)
	}

	if !putCalled {
		t.Fatal("PUT request was not made")
	}
}

func TestHttpTeamRepositorySetIpMultipleUsers(t *testing.T) {
	users := []apiUser{
		{
			ID:       "user-1",
			TeamID:   "team-1",
			IP:       "",
			Name:     "User 1",
			Username: "user1",
			Email:    "u1@test.com",
			Enabled:  true,
		},
		{
			ID:       "user-2",
			TeamID:   "team-1",
			IP:       "",
			Name:     "User 2",
			Username: "user2",
			Email:    "u2@test.com",
			Enabled:  true,
		},
	}

	putCount := 0

	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		switch {
		case r.URL.Path == "/api/v4/users" && r.Method == http.MethodGet:
			json.NewEncoder(w).Encode(users)
		case r.Method == http.MethodPut:
			putCount++
			w.WriteHeader(http.StatusOK)
		default:
			http.NotFound(w, r)
		}
	}))
	defer srv.Close()

	repo := NewHttpTeamRepository(srv.URL, "admin", "password")
	ip := "192.168.1.1"
	err := repo.SetIp(context.Background(), "team-1", &ip)
	if err != nil {
		t.Fatalf("SetIp failed: %v", err)
	}

	if putCount != 2 {
		t.Fatalf("expected 2 PUT requests, got %d", putCount)
	}
}
