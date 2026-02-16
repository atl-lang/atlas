package main

import (
	"github.com/atlas-lang/atlas-dev/internal/api"
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func apiGenerateCmd() *cobra.Command {
	var outputFile string

	cmd := &cobra.Command{
		Use:   "generate <source-path>",
		Short: "Generate API documentation from Rust code",
		Long:  `Auto-generate API documentation by parsing Rust source code.`,
		Example: `  # Generate API docs
  atlas-dev api generate ../../crates/atlas-runtime/src/stdlib -o api-docs.md`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			sourcePath := args[0]

			// Generate
			result, err := api.Generate(sourcePath, outputFile)
			if err != nil {
				return err
			}

			response := map[string]interface{}{
				"functions": result.FunctionCount,
			}

			if result.OutputPath != "" {
				response["output"] = result.OutputPath
				response["msg"] = "API documentation generated"
			} else {
				response["generated"] = result.Generated
			}

			return output.Success(response)
		},
	}

	cmd.Flags().StringVarP(&outputFile, "output", "o", "", "Output file path")

	return cmd
}
