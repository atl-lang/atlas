package api

import (
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// GenerateResult represents generation results
type GenerateResult struct {
	FunctionCount int    `json:"function_count"`
	OutputPath    string `json:"output_path,omitempty"`
	Generated     string `json:"generated,omitempty"`
}

// Generate generates API documentation from Rust code
func Generate(sourcePath, outputPath string) (*GenerateResult, error) {
	functions := []*rustFunction{}

	// Extract functions from Rust code
	err := filepath.Walk(sourcePath, func(filePath string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if info.IsDir() || !strings.HasSuffix(filePath, ".rs") {
			return nil
		}

		fns, err := extractFunctionsWithDocs(filePath)
		if err != nil {
			return nil // Skip problematic files
		}

		functions = append(functions, fns...)
		return nil
	})

	if err != nil {
		return nil, err
	}

	// Generate markdown
	markdown := generateMarkdown(functions)

	result := &GenerateResult{
		FunctionCount: len(functions),
	}

	if outputPath != "" {
		err := os.WriteFile(outputPath, []byte(markdown), 0644)
		if err != nil {
			return nil, fmt.Errorf("failed to write output: %w", err)
		}
		result.OutputPath = outputPath
	} else {
		result.Generated = markdown
	}

	return result, nil
}

func extractFunctionsWithDocs(path string) ([]*rustFunction, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	text := string(content)
	functions := []*rustFunction{}

	// Find pub fn with doc comments
	// This is simplified - a real implementation would use a proper parser
	lines := strings.Split(text, "\n")
	var docComment strings.Builder

	for _, line := range lines {
		trimmed := strings.TrimSpace(line)

		// Collect doc comments
		if strings.HasPrefix(trimmed, "///") {
			comment := strings.TrimPrefix(trimmed, "///")
			docComment.WriteString(strings.TrimSpace(comment) + " ")
			continue
		}

		// Check for pub fn
		if strings.HasPrefix(trimmed, "pub fn ") {
			pattern := regexp.MustCompile(`pub\s+fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)(?:\s*->\s*[^{]+)?`)
			matches := pattern.FindStringSubmatch(trimmed)
			if len(matches) > 1 {
				fn := &rustFunction{
					name:      matches[1],
					signature: matches[0],
				}
				functions = append(functions, fn)
			}
			docComment.Reset()
		} else if trimmed != "" && !strings.HasPrefix(trimmed, "///") {
			docComment.Reset()
		}
	}

	return functions, nil
}

func generateMarkdown(functions []*rustFunction) string {
	var md strings.Builder

	md.WriteString("# API Documentation\n\n")
	md.WriteString("Auto-generated from Rust source code.\n\n")
	md.WriteString("---\n\n")

	for _, fn := range functions {
		md.WriteString(fmt.Sprintf("## %s\n\n", fn.name))
		md.WriteString(fmt.Sprintf("**Signature:**\n```rust\n%s\n```\n\n", fn.signature))
		md.WriteString("---\n\n")
	}

	return md.String()
}
