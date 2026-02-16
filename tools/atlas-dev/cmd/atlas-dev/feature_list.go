package main

import (
	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureListCmd() *cobra.Command {
	var (
		category string
		status   string
		limit    int
		offset   int
	)

	cmd := &cobra.Command{
		Use:   "list",
		Short: "List features",
		Long:  `List all features with optional category and status filters.`,
		Example: `  # List all features
  atlas-dev feature list

  # List by status
  atlas-dev feature list --status Implemented

  # List by category
  atlas-dev feature list --category core

  # Filter from stdin (auto-detected - show only features from input)
  echo '[{"name":"pattern-matching"},{"name":"modules"}]' | atlas-dev feature list`,
		RunE: func(cmd *cobra.Command, args []string) error {
			var filterNames []string

			// Auto-detect stdin for filtering
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}
				filterNames = compose.ExtractField(input, "name")
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

			features, err := database.ListFeatures(db.ListFeaturesOptions{
				Category: category,
				Status:   status,
				Limit:    limit,
				Offset:   offset,
			})
			if err != nil {
				return err
			}

			// Convert to minimal JSON (surgical by default)
			items := make([]map[string]interface{}, 0, len(features))
			for _, f := range features {
				// Filter by stdin names if provided
				if len(filterNames) > 0 {
					found := false
					for _, fn := range filterNames {
						if fn == f.Name {
							found = true
							break
						}
					}
					if !found {
						continue
					}
				}

				items = append(items, map[string]interface{}{
					"name": f.Name,
					"ver":  f.Version,
					"stat": f.Status,
				})
			}

			result := map[string]interface{}{
				"features": items,
				"cnt":      len(items),
				"lim":      limit,
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")
	cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status")
	cmd.Flags().IntVar(&limit, "limit", 0, "Limit results (default: 10, max: 100)")
	cmd.Flags().IntVar(&offset, "offset", 0, "Offset for pagination")

	return cmd
}
