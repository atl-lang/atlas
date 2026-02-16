package lock

import (
	"fmt"
	"log/slog"
	"os"
	"time"
)

const (
	maxRetries     = 50              // 50 retries
	retryInterval  = 100 * time.Millisecond // 100ms between retries
	lockTimeout    = 5 * time.Second        // Total 5 seconds timeout
)

// FileLock represents an exclusive file lock
type FileLock struct {
	path string
	file *os.File
}

// Acquire creates an exclusive lock file
func Acquire(path string) (*FileLock, error) {
	lockPath := path + ".lock"
	start := time.Now()

	for i := 0; i < maxRetries; i++ {
		// Try to create exclusive lock file
		file, err := os.OpenFile(lockPath, os.O_CREATE|os.O_EXCL|os.O_WRONLY, 0600)
		if err == nil {
			// Lock acquired
			// Write PID to lock file
			_, _ = fmt.Fprintf(file, "%d\n", os.Getpid())
			_ = file.Sync()

			duration := time.Since(start)
			slog.Debug("lock acquired", "path", lockPath, "duration_ms", duration.Milliseconds())

			return &FileLock{
				path: lockPath,
				file: file,
			}, nil
		}

		// Lock exists, wait and retry
		if os.IsExist(err) {
			if time.Since(start) >= lockTimeout {
				return nil, fmt.Errorf("lock timeout after %v: %s", lockTimeout, lockPath)
			}
			time.Sleep(retryInterval)
			continue
		}

		// Other error
		return nil, fmt.Errorf("failed to create lock file: %w", err)
	}

	return nil, fmt.Errorf("failed to acquire lock after %d retries: %s", maxRetries, lockPath)
}

// Release removes the lock file
func (fl *FileLock) Release() error {
	if fl.file != nil {
		_ = fl.file.Close() // Best effort
	}

	err := os.Remove(fl.path)
	if err != nil && !os.IsNotExist(err) {
		return fmt.Errorf("failed to remove lock file: %w", err)
	}

	slog.Debug("lock released", "path", fl.path)
	return nil
}

// WithLock executes fn with an exclusive lock
func WithLock(path string, fn func() error) error {
	lock, err := Acquire(path)
	if err != nil {
		return err
	}
	defer func() { _ = lock.Release() }()

	return fn()
}
