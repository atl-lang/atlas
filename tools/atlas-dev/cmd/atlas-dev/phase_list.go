package main

import (
	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func phaseListCmd() *cobra.Command {
	var (
		category string
		status   string
		limit    int
		offset   int
	)

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List phases",
		Long:  `List phases with optional filters.`,
		Example: `  # List all phases
  atlas-dev phase list

  # Filter by category
  atlas-dev phase list --category stdlib

  # Filter from stdin (auto-detected - show only phases from input)
  echo '[{"path":"phases/stdlib/phase-01.md"},{"path":"phases/stdlib/phase-02.md"}]' | atlas-dev phase list`,
		RunE: func(cmd *cobra.Command, args []string) error {
			var filterPaths []string

			// Auto-detect stdin for filtering
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}
				filterPaths = compose.ExtractPaths(input)
			}

			// Apply default limit for token efficiency (AI-only)
			const DEFAULT_LIST_LIMIT = 10
			const MAX_LIST_LIMIT = 100
			if limit == 0 {
				limit = DEFAULT_LIST_LIMIT
			}
			if limit > MAX_LIST_LIMIT {
				limit = MAX_LIST_LIMIT
			}

			phases, err := database.ListPhases(db.ListPhasesOptions{
				Category: category,
				Status:   status,
				Limit:    limit,
				Offset:   offset,
			})
			if err != nil {
				return err
			}

			// Convert to minimal JSON (surgical by default for token efficiency)
			result := make([]map[string]interface{}, 0, len(phases))
			for _, p := range phases {
				// Filter by stdin paths if provided
				if len(filterPaths) > 0 {
					found := false
					for _, fp := range filterPaths {
						if fp == p.Path {
							found = true
							break
						}
					}
					if !found {
						continue
					}
				}

				// Minimal fields: path (unique ID), cat (for filtering), sts (for status)
				item := map[string]interface{}{
					"path": p.Path,
					"cat":  p.Category,
					"sts":  p.Status,
				}
				// Only include date if completed (conditional to save tokens)
				if p.Status == "completed" && p.CompletedDate.Valid {
					item["date"] = p.CompletedDate.String
				}
				result = append(result, item)
			}

			return output.Success(map[string]interface{}{
				"phases": result,
				"cnt":    len(result),
				"lim":    limit,
			})
		},
	}

	cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")
	cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status")
	cmd.Flags().IntVar(&limit, "limit", 0, "Limit number of results")
	cmd.Flags().IntVar(&offset, "offset", 0, "Offset for pagination")

	return cmd
}
