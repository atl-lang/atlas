package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func phaseCurrentCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "current",
		Short: "Show last completed phase",
		Long:  `Display the most recently completed phase.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			phase, err := database.GetCurrentPhase()
			if err != nil {
				return err
			}

			if phase == nil {
				return output.Success(map[string]interface{}{
					"msg": "No phases completed yet",
				})
			}

			return output.Success(phase.ToCompactJSON())
		},
	}
}
