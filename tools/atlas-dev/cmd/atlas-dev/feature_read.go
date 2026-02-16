package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureReadCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "read <name>",
		Short: "Read a feature from database",
		Long:  `Read complete feature details from database. No MD files required.`,
		Example: `  # Read feature
  atlas-dev feature read pattern-matching

  # Read from stdin (auto-detected)
  echo '{"name":"pattern-matching"}' | atlas-dev feature read`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var name string

			// Auto-detect stdin or use args
			if compose.HasStdin() {
				input, err := compose.ReadAndParseStdin()
				if err != nil {
					return err
				}

				name, err = compose.ExtractFirstString(input, "name")
				if err != nil {
					return err
				}
			} else {
				if len(args) < 1 {
					return fmt.Errorf("feature name required")
				}
				name = args[0]
			}

			// Get from database
			feature, err := database.GetFeature(name)
			if err != nil {
				return err
			}

			result := feature.ToCompactJSON()
			result["ok"] = true
			return output.Success(result)
		},
	}

	return cmd
}
