package main

import (
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureCountCmd() *cobra.Command {
	var (
		category string
		status   string
	)

	cmd := &cobra.Command{
		Use:   "count",
		Short: "Count features (token-efficient)",
		Long:  `Count features matching filters without fetching data. Returns just the count in minimal JSON.`,
		Example: `  # Count all features
  atlas-dev feature count

  # Count implemented features
  atlas-dev feature count -s Implemented

  # Count core features
  atlas-dev feature count -c core

  # Count core features that are implemented
  atlas-dev feature count -c core -s Implemented`,
		RunE: func(cmd *cobra.Command, args []string) error {
			count, err := database.CountFeatures(db.ListFeaturesOptions{
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
