.PHONY: run db db-stop test

GOCACHE ?= /tmp/go-build
PKGS := $(shell GOCACHE=$(GOCACHE) go list ./... | rg -v '/gen/' | rg -v '/internal/infra/ent' | rg -v '/cmd/')

run:
	go run ./cmd/backend/main.go

db:
	docker compose up -d

db-stop:
	docker compose down

test:
	GOCACHE=$(GOCACHE) go test $(PKGS)

rpc-ui:
	grpcui -plaintext -port 8090 0.0.0.0:8080
