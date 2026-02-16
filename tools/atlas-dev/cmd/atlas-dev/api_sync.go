package main

import (
	"encoding/json"
	"fmt"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/api"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiSyncCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "sync",
		Short: "Sync API documentation files to database",
		Long:  `Parse docs/api/*.md files and store FULL content in database. Replaces MD files.`,
		Example: `  # Sync all API doc files
  atlas-dev api sync`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Find API doc files
			apiDir := "docs/api"
			pattern := filepath.Join(apiDir, "*.md")
			files, err := filepath.Glob(pattern)
			if err != nil {
				return fmt.Errorf("failed to glob API files: %w", err)
			}

			synced := 0
			for _, path := range files {
				// Parse FULL API content
				parsed, err := api.Parse(path)
				if err != nil {
					return fmt.Errorf("failed to parse %s: %w", path, err)
				}

				// Extract module from filename (e.g., "stdlib.md" → "stdlib")
				name := strings.TrimSuffix(filepath.Base(path), ".md")
				module := name

				// Serialize functions to JSON
				functionsJSON, err := json.Marshal(parsed.Functions)
				if err != nil {
					return fmt.Errorf("failed to marshal functions: %w", err)
				}

				// Serialize types to JSON
				typesJSON := ""
				if len(parsed.Types) > 0 {
					typesBytes, err := json.Marshal(parsed.Types)
					if err != nil {
						return fmt.Errorf("failed to marshal types: %w", err)
					}
					typesJSON = string(typesBytes)
				}

				// Serialize examples to JSON
				examplesJSON := ""
				if len(parsed.Examples) > 0 {
					examplesBytes, err := json.Marshal(parsed.Examples)
					if err != nil {
						return fmt.Errorf("failed to marshal examples: %w", err)
					}
					examplesJSON = string(examplesBytes)
				}

				// Insert or update with FULL content
				_, err = database.Exec(`
					INSERT INTO api_docs (path, module, name, title, content, functions, types, examples, functions_count)
					VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
					ON CONFLICT(path) DO UPDATE SET
						module = excluded.module,
						name = excluded.name,
						title = excluded.title,
						content = excluded.content,
						functions = excluded.functions,
						types = excluded.types,
						examples = excluded.examples,
						functions_count = excluded.functions_count,
						updated_at = datetime('now')
				`, path, module, name, parsed.Title, parsed.RawContent, string(functionsJSON), typesJSON, examplesJSON, len(parsed.Functions))

				if err != nil {
					return fmt.Errorf("failed to insert API doc %s: %w", path, err)
				}
				synced++
			}

			return output.Success(map[string]interface{}{
				"synced": synced,
				"dir":    apiDir,
			})
		},
	}

	return cmd
}
