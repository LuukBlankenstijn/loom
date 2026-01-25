package domain

import (
	"errors"
	"testing"
)

func TestErrAlreadyRegistered(t *testing.T) {
	if ErrAlreadyRegistered == nil {
		t.Fatalf("expected error to be non-nil")
	}
	if ErrAlreadyRegistered.Error() != "station already registered" {
		t.Fatalf("unexpected error message: %q", ErrAlreadyRegistered.Error())
	}
	if !errors.Is(ErrAlreadyRegistered, ErrAlreadyRegistered) {
		t.Fatalf("expected errors.Is to match ErrAlreadyRegistered")
	}
}
