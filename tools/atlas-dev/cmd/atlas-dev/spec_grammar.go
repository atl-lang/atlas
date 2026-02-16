package main

import (
	"path/filepath"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/spec"
	"github.com/spf13/cobra"
)

func specGrammarCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "grammar <spec-file>",
		Short: "Validate EBNF grammar in specification",
		Long:  `Parse and validate EBNF grammar rules in specification document.`,
		Example: `  # Validate grammar
  atlas-dev spec grammar ../../docs/specification/syntax.md`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			specPath := args[0]

			// Parse spec to extract grammar rules
			parsed, err := spec.Parse(specPath)
			if err != nil {
				return err
			}

			if len(parsed.GrammarRules) == 0 {
				result := map[string]interface{}{
					"spec":  filepath.Base(specPath),
					"rules": 0,
					"msg":   "no grammar rules found",
				}
				return output.Success(result)
			}

			// Validate grammar
			validation := spec.ValidateGrammar(parsed.GrammarRules)

			result := map[string]interface{}{
				"spec":  filepath.Base(specPath),
				"valid": validation.Valid,
				"rules": validation.TotalRules,
			}

			if len(validation.Errors) > 0 {
				result["errors"] = validation.Errors
			}
			if len(validation.Warnings) > 0 {
				result["warnings"] = validation.Warnings
			}
			if len(validation.Undefined) > 0 {
				result["undefined"] = validation.Undefined
			}
			if len(validation.Circular) > 0 {
				result["circular"] = validation.Circular
			}

			return output.Success(result)
		},
	}

	return cmd
}
