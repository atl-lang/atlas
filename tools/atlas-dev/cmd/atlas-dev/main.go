package main

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
)

var (
	version = "1.0.0"
)

func main() {
	rootCmd := &cobra.Command{
		Use:   "atlas-dev",
		Short: "Atlas development automation and docs management",
		Long: `Atlas Dev automates development workflows for Atlas.
Manages phases, docs, specs, APIs, and parity validation.

AI agents run ONE command to update everything automatically.`,
		Version: version,
	}

	// Complete command - PRIMARY
	completeCmd := &cobra.Command{
		Use:   "complete <phase-path>",
		Short: "Mark phase complete and update all tracking files",
		Long: `Mark a phase as complete and automatically update:
- Tracker file (mark ✅ with description)
- STATUS.md current phase
- STATUS.md percentages (calculated automatically)
- STATUS.md table row
- Last Updated date
- Validate sync
- Optionally commit changes`,
		Args: cobra.ExactArgs(1),
		Run:  runComplete,
	}
	completeCmd.Flags().StringP("description", "d", "", "Phase completion description (required)")
	completeCmd.Flags().String("date", "", "Completion date (default: today, format: YYYY-MM-DD)")
	completeCmd.Flags().BoolP("commit", "c", false, "Auto-commit changes")
	completeCmd.Flags().Bool("dry-run", false, "Show what would change without modifying files")
	completeCmd.MarkFlagRequired("description")

	// Validate command
	validateCmd := &cobra.Command{
		Use:   "validate",
		Short: "Validate STATUS.md sync with trackers",
		Long: `Verify that STATUS.md percentages and counts match tracker files.
Reports any mismatches and suggests fixes.`,
		Run: runValidate,
	}
	validateCmd.Flags().BoolP("verbose", "v", false, "Show detailed validation info")
	validateCmd.Flags().Bool("fix", false, "Auto-fix percentages if counts match (use with caution)")

	// Next command
	nextCmd := &cobra.Command{
		Use:   "next",
		Short: "Show next phase to work on",
		Long:  `Display the next phase in the current category based on STATUS.md.`,
		Run:   runNext,
	}

	// Summary command
	summaryCmd := &cobra.Command{
		Use:   "summary",
		Short: "Show progress summary",
		Long:  `Display Atlas v0.2 progress dashboard with category breakdown.`,
		Run:   runSummary,
	}
	summaryCmd.Flags().StringP("category", "c", "", "Show detailed category progress")
	summaryCmd.Flags().Bool("json", false, "Output as JSON")

	rootCmd.AddCommand(completeCmd, validateCmd, nextCmd, summaryCmd)

	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintf(os.Stderr, "Error: %v\n", err)
		os.Exit(1)
	}
}

func runComplete(cmd *cobra.Command, args []string) {
	phasePath := args[0]
	description, _ := cmd.Flags().GetString("description")
	date, _ := cmd.Flags().GetString("date")
	commit, _ := cmd.Flags().GetBool("commit")
	dryRun, _ := cmd.Flags().GetBool("dry-run")

	fmt.Printf("🚀 Completing phase: %s\n", phasePath)
	fmt.Printf("   Description: %s\n", description)
	fmt.Printf("   Date: %s\n", date)
	fmt.Printf("   Commit: %v\n", commit)
	fmt.Printf("   Dry Run: %v\n", dryRun)

	// TODO: Implement phase completion logic
	fmt.Println("\n⚠️  Implementation pending - see DESIGN.md for full spec")
}

func runValidate(cmd *cobra.Command, args []string) {
	verbose, _ := cmd.Flags().GetBool("verbose")
	fix, _ := cmd.Flags().GetBool("fix")

	fmt.Println("🔍 Validating STATUS.md sync...")

	// TODO: Implement validation logic
	fmt.Println("\n⚠️  Implementation pending - see DESIGN.md for full spec")

	if verbose {
		fmt.Println("\n(Verbose mode would show detailed tracker counts)")
	}
	if fix {
		fmt.Println("\n(Fix mode would auto-correct percentages)")
	}
}

func runNext(cmd *cobra.Command, args []string) {
	fmt.Println("📋 Finding next phase...")

	// TODO: Implement next phase logic
	fmt.Println("\n⚠️  Implementation pending - see DESIGN.md for full spec")
}

func runSummary(cmd *cobra.Command, args []string) {
	category, _ := cmd.Flags().GetString("category")
	jsonOutput, _ := cmd.Flags().GetBool("json")

	fmt.Println("📊 Atlas v0.2 Progress Summary")

	// TODO: Implement summary logic
	fmt.Println("\n⚠️  Implementation pending - see DESIGN.md for full spec")

	if category != "" {
		fmt.Printf("\n(Category filter: %s)\n", category)
	}
	if jsonOutput {
		fmt.Println("\n(Would output JSON)")
	}
}
