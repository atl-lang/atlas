package main

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/atlas-lang/atlas-dev/internal/output"
	"github.com/atlas-lang/atlas-dev/internal/spec"
	"github.com/spf13/cobra"
)

func specSearchCmd() *cobra.Command {
	var specFilter string

	cmd := &cobra.Command{
		Use:   "search <query>",
		Short: "Search specification documents",
		Long:  `Search across all specification files for matching content.`,
		Example: `  # Search all specs
  atlas-dev spec search "pattern matching"

  # Search specific spec
  atlas-dev spec search "syntax" --spec syntax.md`,
		Args: cobra.ExactArgs(1),
		RunE: func(cmd *cobra.Command, args []string) error {
			query := strings.ToLower(args[0])
			specDir := "../../docs/specification"

			matches := []map[string]interface{}{}

			// Read all spec files
			entries, err := os.ReadDir(specDir)
			if err != nil {
				return err
			}

			for _, entry := range entries {
				if entry.IsDir() || !strings.HasSuffix(entry.Name(), ".md") {
					continue
				}

				// Filter by spec if specified
				if specFilter != "" && entry.Name() != specFilter {
					continue
				}

				specPath := filepath.Join(specDir, entry.Name())
				parsed, err := spec.Parse(specPath)
				if err != nil {
					continue // Skip invalid specs
				}

				// Search in title
				if strings.Contains(strings.ToLower(parsed.Title), query) {
					matches = append(matches, map[string]interface{}{
						"spec":      entry.Name(),
						"match":     "title",
						"title":     parsed.Title,
						"relevance": 10,
					})
				}

				// Search in sections
				for _, section := range parsed.Sections {
					if strings.Contains(strings.ToLower(section.Title), query) {
						matches = append(matches, map[string]interface{}{
							"spec":      entry.Name(),
							"match":     "section",
							"section":   section.Title,
							"relevance": 5,
						})
					}
					if strings.Contains(strings.ToLower(section.Content), query) {
						snippet := extractSnippet(section.Content, query)
						matches = append(matches, map[string]interface{}{
							"spec":      entry.Name(),
							"match":     "content",
							"section":   section.Title,
							"snippet":   snippet,
							"relevance": 3,
						})
					}
				}
			}

			result := map[string]interface{}{
				"query":   args[0],
				"matches": matches,
				"cnt":     len(matches),
			}

			return output.Success(result)
		},
	}

	cmd.Flags().StringVar(&specFilter, "spec", "", "Search only in specific spec file")

	return cmd
}

func extractSnippet(content, query string) string {
	lower := strings.ToLower(content)
	queryLower := strings.ToLower(query)

	idx := strings.Index(lower, queryLower)
	if idx == -1 {
		return content[:min(100, len(content))]
	}

	start := max(0, idx-50)
	end := min(len(content), idx+len(query)+50)

	snippet := content[start:end]
	if start > 0 {
		snippet = "..." + snippet
	}
	if end < len(content) {
		snippet = snippet + "..."
	}

	return snippet
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}

func max(a, b int) int {
	if a > b {
		return a
	}
	return b
}
