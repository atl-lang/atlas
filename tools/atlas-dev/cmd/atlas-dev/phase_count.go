package main

import (
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func phaseCountCmd() *cobra.Command {
	var (
		category string
		status   string
	)

	cmd := &cobra.Command{
		Use:   "count",
		Short: "Count phases (token-efficient)",
		Long:  `Count phases matching filters without fetching data. Returns just the count in minimal JSON.`,
		Example: `  # Count all phases
  atlas-dev phase count

  # Count pending phases
  atlas-dev phase count -s pending

  # Count stdlib phases
  atlas-dev phase count -c stdlib

  # Count stdlib phases that are pending
  atlas-dev phase count -c stdlib -s pending`,
		RunE: func(cmd *cobra.Command, args []string) error {
			count, err := database.CountPhases(db.ListPhasesOptions{
				Category: category,
				Status:   status,
			})
			if err != nil {
				return err
			}

			return output.Success(map[string]interface{}{
				"cnt": count,
			})
		},
	}

	cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")
	cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status")

	return cmd
}
