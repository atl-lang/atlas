package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/compose"
	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureUpdateCmd() *cobra.Command {
	var (
		version     string
		status      string
		description string
		specPath    string
		apiPath     string
		useStdin    bool
	)

	cmd := &cobra.Command{
		Use:   "update <name>",
		Short: "Update a feature",
		Long:  `Update feature metadata in the database.`,
		Example: `  # Update status
  atlas-dev feature update pattern-matching --status Implemented

  # Update version
  atlas-dev feature update pattern-matching --version v0.2

  # Update from stdin
  echo '{"name":"pattern-matching"}' | atlas-dev feature update --stdin --status Implemented`,
		Args: cobra.MaximumNArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			var name string

			// Get name from stdin or args
			if useStdin {
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

			// Build update request
			req := db.UpdateFeatureRequest{
				Name:        name,
				Version:     version,
				Status:      status,
				Description: description,
				SpecPath:    specPath,
				APIPath:     apiPath,
			}

			// Check if any fields to update
			if version == "" && status == "" && description == "" && specPath == "" && apiPath == "" {
				return fmt.Errorf("no fields to update (use --version, --status, etc.)")
			}

			feature, err := database.UpdateFeature(req)
			if err != nil {
				return err
			}

			result := feature.ToCompactJSON()
			result["msg"] = "Feature updated"
			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&version, "version", "", "Update version")
	cmd.Flags().StringVar(&status, "status", "", "Update status")
	cmd.Flags().StringVar(&description, "description", "", "Update description")
	cmd.Flags().StringVar(&specPath, "spec", "", "Update spec path")
	cmd.Flags().StringVar(&apiPath, "api", "", "Update API path")
	cmd.Flags().BoolVar(&useStdin, "stdin", false, "Read feature name from stdin JSON")

	return cmd
}
