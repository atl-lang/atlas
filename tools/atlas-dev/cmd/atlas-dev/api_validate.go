package main

import (
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/api"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiValidateCmd() *cobra.Command {
	var codePath string

	cmd := &cobra.Command{
		Use:   "validate <api-file>",
		Short: "Validate API documentation against code",
		Long:  `Compare API documentation to actual Rust code implementation.`,
		Example: `  # Validate API docs
  atlas-dev api validate ../../docs/api/stdlib.md --code ../../crates/atlas-runtime/src/stdlib`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			apiPath := args[0]

			// Parse API doc
			doc, err := api.Parse(apiPath)
			if err != nil {
				return err
			}

			// Default code path
			if codePath == "" {
				codePath = "../../crates/atlas-runtime/src"
			}

			// Validate
			report, err := api.Validate(doc, codePath)
			if err != nil {
				return err
			}

			result := map[string]interface{}{
				"api":       filepath.Base(apiPath),
				"valid":     report.Valid,
				"documented": report.Documented,
				"in_code":   report.InCode,
				"coverage":  report.Coverage,
				"matches":   report.MatchCount,
			}

			if len(report.Missing) > 0 {
				result["missing"] = report.Missing
			}
			if len(report.Undocumented) > 0 {
				result["undocumented"] = report.Undocumented
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&codePath, "code", "", "Path to Rust source code")

	return cmd
}
