package envutil

import (
	"fmt"
	"testing"
)

func TestGetEnvX(t *testing.T) {
	t.Run("present", func(t *testing.T) {
		t.Setenv("ENVUTIL_PRESENT", "value")

		got := GetEnvX("ENVUTIL_PRESENT")
		if got != "value" {
			t.Fatalf("expected value, got %q", got)
		}
	})

	t.Run("missing panics", func(t *testing.T) {
		key := "ENVUTIL_MISSING"
		expected := fmt.Sprintf("env variable %s not found", key)

		defer func() {
			t.Helper()
			if r := recover(); r == nil {
				t.Fatal("expected panic, got nil")
			} else if msg, ok := r.(string); ok {
				if msg != expected {
					t.Fatalf("expected panic %q, got %q", expected, msg)
				}
			} else {
				t.Fatalf("expected string panic, got %T", r)
			}
		}()

		_ = GetEnvX(key)
	})
}

func TestGetEnvWithFallback(t *testing.T) {
	t.Run("present", func(t *testing.T) {
		t.Setenv("ENVUTIL_FALLBACK_PRESENT", "value")

		got := GetEnvWithFallback("ENVUTIL_FALLBACK_PRESENT", "fallback")
		if got != "value" {
			t.Fatalf("expected value, got %q", got)
		}
	})

	t.Run("missing uses fallback", func(t *testing.T) {
		got := GetEnvWithFallback("ENVUTIL_FALLBACK_MISSING", "fallback")
		if got != "fallback" {
			t.Fatalf("expected fallback, got %q", got)
		}
	})
}

func TestGetEnv(t *testing.T) {
	t.Run("present", func(t *testing.T) {
		t.Setenv("ENVUTIL_PRESENT", "value")

		got, ok := GetEnv("ENVUTIL_PRESENT")
		if got != "value" {
			t.Fatalf("expected value, got %q", got)
		}
		if ok != true {
			t.Fatalf("expected ok, got not ok")
		}
	})

	t.Run("missing", func(t *testing.T) {
		got, ok := GetEnv("ENVUTIL_MISSING")
		if got != "" {
			t.Fatalf("expected empty value, got %q", got)
		}
		if ok != false {
			t.Fatalf("expected not ok, got ok")
		}
	})
}
