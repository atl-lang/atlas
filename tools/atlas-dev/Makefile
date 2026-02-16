.PHONY: build install test bench lint fmt clean all

# Build binary (CGO required for SQLite)
build:
	go build -ldflags="-w -s" -o bin/atlas-dev ./cmd/atlas-dev

# Install to PATH
install:
	go install -ldflags="-w -s" ./cmd/atlas-dev

# Run tests with coverage and race detector
test:
	go test -v -race -cover -coverprofile=coverage.out ./...
	@echo "\nCoverage report:"
	@go tool cover -func=coverage.out | grep total

# Run benchmarks
bench:
	go test -bench=. -benchmem -run=^$$ ./...

# Lint code
lint:
	golangci-lint run

# Format code
fmt:
	go fmt ./...
	goimports -w .

# Clean build artifacts
clean:
	rm -rf bin/ coverage.out

# All quality checks
all: fmt lint test

# Development: build and run
dev: build
	./bin/atlas-dev

# Show version
version:
	@echo "atlas-dev v1.0.0"
