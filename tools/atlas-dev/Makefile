.PHONY: build install test clean

# Build binary
build:
	go build -o bin/status-manager ./cmd/status-manager

# Install to PATH
install:
	go install ./cmd/status-manager

# Run tests
test:
	go test -v ./...

# Clean build artifacts
clean:
	rm -rf bin/

# Development: build and run
dev: build
	./bin/status-manager

# Format code
fmt:
	go fmt ./...

# Lint
lint:
	golangci-lint run

# Show version
version:
	@echo "status-manager v1.0.0"
