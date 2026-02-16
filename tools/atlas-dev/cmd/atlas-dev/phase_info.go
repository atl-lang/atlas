package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func phaseInfoCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "info <phase-path>",
		Short: "Show phase details",
		Long:  `Display detailed information about a specific phase.`,
		Args:  cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			phasePath := args[0]

			phase, err := database.GetPhaseInfo(phasePath)
			if err != nil {
				return err
			}

			return output.Success(phase.ToCompactJSON())
		},
	}
}
