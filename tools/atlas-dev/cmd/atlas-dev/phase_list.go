package main

import (
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
		RunE: func(cmd *cobra.Command, args []string) error {
			phases, err := database.ListPhases(db.ListPhasesOptions{
				Category: category,
				Status:   status,
				Limit:    limit,
				Offset:   offset,
			})
			if err != nil {
				return err
			}

			// Convert to compact JSON
			result := make([]map[string]interface{}, 0, len(phases))
			for _, p := range phases {
				item := map[string]interface{}{
					"path": p.Path,
					"name": p.Name,
					"cat":  p.Category,
					"sts":  p.Status,
				}
				if p.CompletedDate.Valid {
					item["date"] = p.CompletedDate.String
				}
				result = append(result, item)
			}

			return output.Success(map[string]interface{}{
				"phases": result,
				"count":  len(result),
			})
		},
	}

	cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")
	cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status")
	cmd.Flags().IntVar(&limit, "limit", 0, "Limit number of results")
	cmd.Flags().IntVar(&offset, "offset", 0, "Offset for pagination")

	return cmd
}
