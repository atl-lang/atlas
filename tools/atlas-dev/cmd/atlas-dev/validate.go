package main

import (
	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func validateCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "validate",
		Short: "Validate database consistency",
		Long: `Check database for consistency issues:
- Category counts match actual phase counts
- Percentages calculated correctly
- Metadata is accurate
- No orphaned phases
- No invalid statuses
- All triggers exist`,
		RunE: func(cmd *cobra.Command, args []string) error {
			report, err := database.Validate()
			if err != nil {
				return err
			}

			// Build response
			response := map[string]interface{}{
				"checks": report.ChecksRun,
				"errors": report.ErrorsFound,
			}

			if len(report.Issues) > 0 {
				issues := make([]map[string]interface{}, 0, len(report.Issues))
				for _, issue := range report.Issues {
					issues = append(issues, map[string]interface{}{
						"check":  issue.Check,
						"sev":    issue.Severity,
						"msg":    issue.Message,
						"fix":    issue.Suggestion,
					})
				}
				response["issues"] = issues
			}

			if report.OK {
				response["msg"] = "Database is consistent"
			}

			return output.Success(response)
		},
	}
}
