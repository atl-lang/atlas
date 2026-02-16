package git

import (
	"fmt"
	"log/slog"
	"os"
	"os/exec"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/lock"
)

const lockPath = "atlas-dev.db"

// IsRepository checks if current directory is inside a git repository
func IsRepository() bool {
	_, err := os.Stat(".git")
	return err == nil
}

// IsConfigured checks if git user.name and user.email are set
func IsConfigured() (bool, error) {
	// Check user.name
	cmd := exec.Command("git", "config", "user.name")
	output, err := cmd.Output()
	if err != nil || len(strings.TrimSpace(string(output))) == 0 {
		return false, fmt.Errorf("git user.name not configured")
	}

	// Check user.email
	cmd = exec.Command("git", "config", "user.email")
	output, err = cmd.Output()
	if err != nil || len(strings.TrimSpace(string(output))) == 0 {
		return false, fmt.Errorf("git user.email not configured")
	}

	return true, nil
}

// Commit creates a git commit with the specified message
// Returns commit SHA on success
func Commit(message string, files ...string) (string, error) {
	// Check repository
	if !IsRepository() {
		return "", fmt.Errorf("not in a git repository")
	}

	// Check configuration
	if ok, err := IsConfigured(); !ok {
		return "", err
	}

	// Use file locking to prevent concurrent commits
	var sha string
	err := lock.WithLock(lockPath, func() error {
		// Stage files
		if len(files) > 0 {
			args := append([]string{"add"}, files...)
			cmd := exec.Command("git", args...)
			output, err := cmd.CombinedOutput()
			if err != nil {
				return fmt.Errorf("failed to stage files: %w\nOutput: %s", err, output)
			}
			slog.Debug("staged files", "files", files)
		}

		// Check if there's anything to commit
		cmd := exec.Command("git", "diff", "--cached", "--quiet")
		err := cmd.Run()
		if err == nil {
			// Exit code 0 means no changes
			return fmt.Errorf("nothing to commit")
		}

		// Create commit
		cmd = exec.Command("git", "commit", "-m", message)
		output, err := cmd.CombinedOutput()
		if err != nil {
			return fmt.Errorf("failed to create commit: %w\nOutput: %s", err, output)
		}

		slog.Debug("created commit", "message", message)

		// Get commit SHA
		cmd = exec.Command("git", "rev-parse", "HEAD")
		output, err = cmd.Output()
		if err != nil {
			return fmt.Errorf("failed to get commit SHA: %w", err)
		}

		sha = strings.TrimSpace(string(output))
		slog.Info("git commit created", "sha", sha)

		return nil
	})

	if err != nil {
		return "", err
	}

	return sha, nil
}

// CommitPhase creates a commit for a completed phase
// Message format: "Complete {phase-name} - {description}\n\nCo-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
func CommitPhase(phaseName, description string) (string, error) {
	message := fmt.Sprintf(`Complete %s - %s

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>`, phaseName, description)

	// Use file locking to prevent concurrent commits
	var sha string
	err := lock.WithLock(lockPath, func() error {
		// Check repository
		if !IsRepository() {
			return fmt.Errorf("not in a git repository")
		}

		// Check configuration
		if ok, err := IsConfigured(); !ok {
			return err
		}

		// Stage database file
		cmd := exec.Command("git", "add", "atlas-dev.db")
		output, err := cmd.CombinedOutput()
		if err != nil {
			return fmt.Errorf("failed to stage database: %w\nOutput: %s", err, output)
		}

		// Check if there's anything to commit
		cmd = exec.Command("git", "diff", "--cached", "--quiet")
		err = cmd.Run()
		if err == nil {
			// Exit code 0 means no changes
			return fmt.Errorf("nothing to commit")
		}

		// Create commit
		cmd = exec.Command("git", "commit", "-m", message)
		output, err = cmd.CombinedOutput()
		if err != nil {
			return fmt.Errorf("failed to create commit: %w\nOutput: %s", err, output)
		}

		slog.Debug("created phase commit", "phase", phaseName)

		// Get commit SHA
		cmd = exec.Command("git", "rev-parse", "HEAD")
		output, err = cmd.Output()
		if err != nil {
			return fmt.Errorf("failed to get commit SHA: %w", err)
		}

		sha = strings.TrimSpace(string(output))
		slog.Info("git commit created", "phase", phaseName, "sha", sha)

		return nil
	})

	if err != nil {
		return "", err
	}

	return sha, nil
}
