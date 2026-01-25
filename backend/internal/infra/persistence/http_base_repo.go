package persistence

import (
	"context"
	"encoding/json"
	"fmt"
	"log/slog"
	"net/http"
	"time"
)

type httpBaseRepository struct {
	client   *http.Client
	baseURL  string
	username string
	password string
}

func newHTTPBaseRepository(baseURL, username, password string) httpBaseRepository {
	return httpBaseRepository{
		baseURL:  baseURL,
		username: username,
		password: password,
		client: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

// Generic helper for GET requests
func (r *httpBaseRepository) get(ctx context.Context, url string, target any) error {
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, url, nil)
	if err != nil {
		slog.Error("failed to create http request", "error", err)
		return err
	}
	req.SetBasicAuth(r.username, r.password)

	resp, err := r.client.Do(req)
	if err != nil {
		slog.Error("http request failed", "error", err)
		return err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		slog.Error("unexpected status code", "status", resp.StatusCode)
		return fmt.Errorf("api error: status %d", resp.StatusCode)
	}

	if err := json.NewDecoder(resp.Body).Decode(target); err != nil {
		slog.Error("failed to decode response", "error", err)
		return err
	}
	return nil
}
