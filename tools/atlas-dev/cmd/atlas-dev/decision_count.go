package main

import (
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionCountCmd() *cobra.Command {
	var (
		component string
		status    string
	)

	cmd := &cobra.Command{
		Use:   "count",
		Short: "Count decisions (token-efficient)",
		Long:  `Count decisions matching filters without fetching data. Returns just the count in minimal JSON.`,
		Example: `  # Count all decisions
  atlas-dev decision count

  # Count accepted decisions
  atlas-dev decision count -s accepted

  # Count stdlib decisions
  atlas-dev decision count -c stdlib

  # Count stdlib decisions that are accepted
  atlas-dev decision count -c stdlib -s accepted`,
		RunE: func(cmd *cobra.Command, args []string) error {
			count, err := database.CountDecisions(db.ListDecisionsOptions{
				Component: component,
				Status:    status,
			})
			if err != nil {
				return err
			}

			return output.Success(map[string]interface{}{
				"cnt": count,
			})
		},
	}

	cmd.Flags().StringVarP(&component, "component", "c", "", "Filter by component")
	cmd.Flags().StringVarP(&status, "status", "s", "", "Filter by status")

	return cmd
}
