package main

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/spec"
	"github.com/spf13/cobra"
)

func specSyncCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sync",
		Short: "Sync specification files to database",
		Long:  `Parse docs/specification/*.md files and store FULL content in database. Replaces MD files.`,
		Example: `  # Sync all spec files
  atlas-dev spec sync`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Find spec files
			specDir := "docs/specification"
			pattern := filepath.Join(specDir, "*.md")
			files, err := filepath.Glob(pattern)
			if err != nil {
				return fmt.Errorf("failed to glob spec files: %w", err)
			}

			synced := 0
			for _, path := range files {
				// Parse FULL spec content
				parsed, err := spec.Parse(path)
				if err != nil {
					return fmt.Errorf("failed to parse %s: %w", path, err)
				}

				// Extract name from filename
				name := strings.TrimSuffix(filepath.Base(path), ".md")

				// Determine section from filename
				section := "specification"
				if strings.Contains(name, "grammar") {
					section = "grammar"
				} else if strings.Contains(name, "bytecode") {
					section = "bytecode"
				} else if strings.Contains(name, "diagnostic") {
					section = "diagnostics"
				}

				// Serialize outline to JSON
				outline, err := json.Marshal(parsed.Outline)
				if err != nil {
					return fmt.Errorf("failed to marshal outline: %w", err)
				}

				// Serialize sections to JSON
				sections, err := json.Marshal(parsed.Sections)
				if err != nil {
					return fmt.Errorf("failed to marshal sections: %w", err)
				}

				// Extract grammar if present
				grammar := ""
				if len(parsed.Grammar) > 0 {
					grammarJSON, err := json.Marshal(parsed.Grammar)
					if err != nil {
						return fmt.Errorf("failed to marshal grammar: %w", err)
					}
					grammar = string(grammarJSON)
				}

				// Insert or update with FULL content
				_, err = database.Exec(`
					INSERT INTO specs (path, name, section, title, version, status, content, outline, sections, grammar)
					VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
					ON CONFLICT(path) DO UPDATE SET
						name = excluded.name,
						section = excluded.section,
						title = excluded.title,
						version = excluded.version,
						status = excluded.status,
						content = excluded.content,
						outline = excluded.outline,
						sections = excluded.sections,
						grammar = excluded.grammar,
						updated_at = datetime('now')
				`, path, name, section, parsed.Title, parsed.Version, parsed.Status, parsed.RawContent, string(outline), string(sections), grammar)

				if err != nil {
					return fmt.Errorf("failed to insert spec %s: %w", path, err)
				}
				synced++
			}

			return output.Success(map[string]interface{}{
				"synced": synced,
				"dir":    specDir,
			})
		},
	}

	return cmd
}
