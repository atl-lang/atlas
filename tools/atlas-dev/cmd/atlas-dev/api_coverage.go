package main

import (
	"github.com/atlas-lang/atlas-dev/internal/api"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiCoverageCmd() *cobra.Command {
	var codePath string

	cmd := &cobra.Command{
		Use:   "coverage <api-file>",
		Short: "Track API documentation coverage",
		Long:  `Calculate what percentage of public functions are documented.`,
		Example: `  # Check coverage
  atlas-dev api coverage ../../docs/api/stdlib.md --code ../../crates/atlas-runtime/src/stdlib`,
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

			// Validate to get coverage
			report, err := api.Validate(doc, codePath)
			if err != nil {
				return err
			}

			result := map[string]interface{}{
				"total":       report.InCode,
				"documented":  report.Documented,
				"coverage":    report.Coverage,
				"missing_cnt": len(report.Undocumented),
			}

			if len(report.Undocumented) > 0 {
				result["missing"] = report.Undocumented
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&codePath, "code", "", "Path to Rust source code")

	return cmd
}
