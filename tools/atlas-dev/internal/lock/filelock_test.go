package lock

import (
	"os"
	"path/filepath"
	"testing"
	"time"
)

func TestAcquireAndRelease(t *testing.T) {
	tmpDir := t.TempDir()
	lockPath := filepath.Join(tmpDir, "test")

	lock, err := Acquire(lockPath)
	if err != nil {
		t.Fatalf("Acquire() error: %v", err)
	}

	// Verify lock file exists
	if _, err := os.Stat(lockPath + ".lock"); os.IsNotExist(err) {
		t.Error("lock file was not created")
	}

	// Release lock
	if err := lock.Release(); err != nil {
		t.Errorf("Release() error: %v", err)
	}

	// Verify lock file removed
	if _, err := os.Stat(lockPath + ".lock"); !os.IsNotExist(err) {
		t.Error("lock file was not removed")
	}
}

func TestAcquireExclusive(t *testing.T) {
	tmpDir := t.TempDir()
	lockPath := filepath.Join(tmpDir, "test")

	// Acquire first lock
	lock1, err := Acquire(lockPath)
	if err != nil {
		t.Fatalf("Acquire() error: %v", err)
	}
	defer lock1.Release()

	// Try to acquire second lock (should timeout)
	done := make(chan error, 1)
	go func() {
		_, err := Acquire(lockPath)
		done <- err
	}()

	select {
	case err := <-done:
		if err == nil {
			t.Error("expected error when acquiring already-locked file")
		}
	case <-time.After(6 * time.Second):
		t.Error("Acquire() should have timed out")
	}
}

func TestAcquireRetry(t *testing.T) {
	tmpDir := t.TempDir()
	lockPath := filepath.Join(tmpDir, "test")

	// Acquire first lock
	lock1, err := Acquire(lockPath)
	if err != nil {
		t.Fatalf("Acquire() error: %v", err)
	}

	// Try to acquire second lock in background
	done := make(chan *FileLock, 1)
	go func() {
		time.Sleep(200 * time.Millisecond)
		lock1.Release()
	}()

	go func() {
		lock2, _ := Acquire(lockPath)
		done <- lock2
	}()

	// Should succeed after lock1 is released
	select {
	case lock2 := <-done:
		if lock2 == nil {
			t.Error("expected to acquire lock after retry")
		} else {
			lock2.Release()
		}
	case <-time.After(2 * time.Second):
		t.Error("Acquire() should have succeeded after retry")
	}
}

func TestWithLock(t *testing.T) {
	tmpDir := t.TempDir()
	lockPath := filepath.Join(tmpDir, "test")

	executed := false

	err := WithLock(lockPath, func() error {
		executed = true

		// Verify lock file exists during execution
		if _, err := os.Stat(lockPath + ".lock"); os.IsNotExist(err) {
			t.Error("lock file should exist during WithLock execution")
		}

		return nil
	})

	if err != nil {
		t.Errorf("WithLock() error: %v", err)
	}

	if !executed {
		t.Error("WithLock() did not execute function")
	}

	// Verify lock released after execution
	if _, err := os.Stat(lockPath + ".lock"); !os.IsNotExist(err) {
		t.Error("lock file should be removed after WithLock")
	}
}

func TestWithLockError(t *testing.T) {
	tmpDir := t.TempDir()
	lockPath := filepath.Join(tmpDir, "test")

	testErr := &testError{msg: "test error"}

	err := WithLock(lockPath, func() error {
		return testErr
	})

	if err != testErr {
		t.Errorf("WithLock() error = %v, want %v", err, testErr)
	}

	// Verify lock released even on error
	if _, err := os.Stat(lockPath + ".lock"); !os.IsNotExist(err) {
		t.Error("lock file should be removed even on error")
	}
}

func TestLockFileContainsPID(t *testing.T) {
	tmpDir := t.TempDir()
	lockPath := filepath.Join(tmpDir, "test")

	lock, err := Acquire(lockPath)
	if err != nil {
		t.Fatalf("Acquire() error: %v", err)
	}
	defer lock.Release()

	// Read lock file
	data, err := os.ReadFile(lockPath + ".lock")
	if err != nil {
		t.Fatalf("failed to read lock file: %v", err)
	}

	// Verify contains PID
	if len(data) == 0 {
		t.Error("lock file should contain PID")
	}
}

type testError struct {
	msg string
}

func (e *testError) Error() string {
	return e.msg
}
