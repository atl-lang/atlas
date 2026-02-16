package main

import (
	"fmt"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/db"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func featureCreateCmd() *cobra.Command {
	var (
		name        string
		displayName string
		category    string
		version     string
		status      string
		description string
		specPath    string
		apiPath     string
		dryRun      bool
	)

	cmd := &cobra.Command{
		Use:   "create",
		Short: "Create a new feature",
		Long:  `Create a new feature with markdown file in docs/features/ and database record.`,
		Example: `  # Create feature
  atlas-dev feature create \
    --name pattern-matching \
    --display "Pattern Matching" \
    --category core \
    --status Planned`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Validate required fields
			if name == "" {
				return fmt.Errorf("--name is required")
			}

			// Default display name to name if not provided
			if displayName == "" {
				displayName = toTitleCase(name)
			}

			// Default version and status
			if version == "" {
				version = "v0.1"
			}
			if status == "" {
				status = "Planned"
			}

			// Dry-run: preview what would be created
			if dryRun {
				result := map[string]interface{}{
					"dry_run": true,
					"op":      "create_feature",
					"preview": map[string]interface{}{
						"name":    name,
						"display": displayName,
						"ver":     version,
						"stat":    status,
					},
					"msg": "Preview only - no changes made",
				}
				return output.Success(result)
			}

			// Create feature in database
			req := db.CreateFeatureRequest{
				Name:        name,
				DisplayName: displayName,
				Version:     version,
				Status:      status,
				Description: description,
				SpecPath:    specPath,
				APIPath:     apiPath,
			}

			feature, err := database.CreateFeature(req)
			if err != nil {
				return err
			}

			result := feature.ToCompactJSON()
			result["msg"] = "Feature created"
			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&name, "name", "", "Feature name (slug format) (required)")
	cmd.Flags().StringVar(&displayName, "display", "", "Display name (defaults to name)")
	cmd.Flags().StringVar(&category, "category", "", "Category")
	cmd.Flags().StringVar(&version, "version", "v0.1", "Version")
	cmd.Flags().StringVar(&status, "status", "Planned", "Status (Planned, InProgress, Implemented)")
	cmd.Flags().StringVar(&description, "description", "", "Description")
	cmd.Flags().StringVar(&specPath, "spec", "", "Spec file path")
	cmd.Flags().StringVar(&apiPath, "api", "", "API file path")
	cmd.Flags().BoolVar(&dryRun, "dry-run", false, "Preview feature without creating")

	return cmd
}

// toTitleCase converts "pattern-matching" to "Pattern Matching"
func toTitleCase(s string) string {
	words := strings.Split(s, "-")
	for i, word := range words {
		if len(word) > 0 {
			words[i] = strings.ToUpper(word[:1]) + word[1:]
		}
	}
	return strings.Join(words, " ")
}
