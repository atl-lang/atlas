package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func decisionUpdateCmd() *cobra.Command {
	var (
		status       string
		supersededBy string
		useStdin     bool
	)

	cmd := &cobra.Command{
		Use:   "update <id>",
		Short: "Update decision status",
		Long:  `Update decision status or mark as superseded.`,
		Example: `  # Update status
  atlas-dev decision update DR-001 --status accepted

  # Mark as superseded
  atlas-dev decision update DR-001 --superseded-by DR-002

  # Update from stdin
  echo '{"id":"DR-001"}' | atlas-dev decision update --stdin --status accepted`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var id string

			// Get ID from stdin or args
			if useStdin {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				id, err = compose.ExtractFirstID(input)
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("decision ID required")
				}
				id = args[0]
			}

			if status == "" && supersededBy == "" {
				return fmt.Errorf("must provide --status or --superseded-by")
			}

			req := db.UpdateDecisionRequest{
				ID:           id,
				Status:       status,
				SupersededBy: supersededBy,
			}

			decision, err := database.UpdateDecision(req)
			if err != nil {
				return err
			}

			result := decision.ToCompactJSON()
			result["msg"] = "Decision updated"
			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&status, "status", "", "New status")
	cmd.Flags().StringVar(&supersededBy, "superseded-by", "", "Superseded by decision ID")
	cmd.Flags().BoolVar(&useStdin, "stdin", false, "Read ID from stdin JSON")

	return cmd
}
