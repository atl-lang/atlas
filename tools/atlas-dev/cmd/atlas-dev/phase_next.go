package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func phaseNextCmd() *cobra.Command {
	var category string

	cmd := &cobra.Command{
		Use:   "next",
		Short: "Show next pending phase",
		Long:  `Display the next phase to work on.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			phase, err := database.GetNextPhase(category)
			if err != nil {
				return err
			}

			if phase == nil {
				return output.Success(map[string]interface{}{
					"msg": "All phases complete",
				})
			}

			return output.Success(phase.ToCompactJSON())
		},
	}

	cmd.Flags().StringVarP(&category, "category", "c", "", "Filter by category")

	return cmd
}
