package phase

import (
	"fmt"
	"path/filepath"
	"strings"
)

// PhaseInfo represents parsed phase information
type PhaseInfo struct {
	Category string
	Name     string
	Path     string
}

// Parse extracts category and phase name from a phase path
// Handles formats:
//   - "phases/{category}/{phase}.md"
//   - "{category}/{phase}.md"
//   - "{phase}.md"
func Parse(phasePath string) (*PhaseInfo, error) {
	if phasePath == "" {
		return nil, fmt.Errorf("phase path cannot be empty")
	}

	// Normalize path
	phasePath = filepath.Clean(phasePath)

	// Remove .md extension if present
	name := strings.TrimSuffix(filepath.Base(phasePath), ".md")
	if name == "" {
		return nil, fmt.Errorf("invalid phase path: %s", phasePath)
	}

	// Extract directory parts
	dir := filepath.Dir(phasePath)
	parts := strings.Split(dir, string(filepath.Separator))

	var category string

	// Determine category based on path structure
	if len(parts) >= 2 && parts[len(parts)-2] == "phases" {
		// Format: "phases/{category}/{phase}.md"
		category = parts[len(parts)-1]
	} else if len(parts) >= 1 && parts[len(parts)-1] != "." {
		// Format: "{category}/{phase}.md"
		category = parts[len(parts)-1]
	} else {
		// Format: "{phase}.md" - extract category from phase name
		// e.g., "phase-01-foundation" -> "foundation"
		nameParts := strings.Split(name, "-")
		if len(nameParts) >= 3 {
			// Join parts after phase number
			category = strings.Join(nameParts[2:], "-")
		} else {
			return nil, fmt.Errorf("cannot determine category from phase path: %s", phasePath)
		}
	}

	// Ensure path has .md extension
	fullPath := phasePath
	if !strings.HasSuffix(fullPath, ".md") {
		fullPath += ".md"
	}

	return &PhaseInfo{
		Category: category,
		Name:     name,
		Path:     fullPath,
	}, nil
}

// ValidCategories returns list of known categories
var ValidCategories = []string{
	"foundation",
	"stdlib",
	"bytecode-vm",
	"frontend",
	"typing",
	"interpreter",
	"cli",
	"lsp",
	"polish",
}

// IsValidCategory checks if category is valid
func IsValidCategory(category string) bool {
	for _, valid := range ValidCategories {
		if category == valid {
			return true
		}
	}
	return false
}
