package main

import (
	"fmt"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/spf13/cobra"
)

func migrateCmd() *cobra.Command {
	cmd := &cobra.Command{
		Use:   "migrate",
		Short: "Database migration commands",
		Long:  `Migrate existing markdown files to SQLite or initialize fresh schema.`,
	}

	cmd.AddCommand(migrateSchemaCmd())
	cmd.AddCommand(migrateBootstrapCmd())

	return cmd
}

func migrateSchemaCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "schema",
		Short: "Initialize database schema",
		Long:  `Create all tables, indexes, triggers, views and seed initial data.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// Initialize schema
			if err := database.InitSchema(); err != nil {
				return fmt.Errorf("failed to initialize schema: %w", err)
			}

			return output.Success(map[string]interface{}{
				"msg":    "Schema initialized successfully",
				"tables": 10,
				"views":  4,
			})
		},
	}
}

func migrateBootstrapCmd() *cobra.Command {
	return &cobra.Command{
		Use:   "bootstrap",
		Short: "Bootstrap database from existing markdown files",
		Long: `One-time migration: parse STATUS.md and trackers/*.md to populate database.
Backs up markdown files to .migration-backup/ directory.`,
		RunE: func(cmd *cobra.Command, args []string) error {
			// TODO: Implement bootstrap migration (Phase 2)
			// This will parse existing STATUS.md and trackers/*.md
			// and populate the database

			return fmt.Errorf("bootstrap not yet implemented - use 'migrate schema' for fresh database")
		},
	}
}
