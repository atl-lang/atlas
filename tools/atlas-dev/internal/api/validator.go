package api

import (
	"os"
	"path/filepath"
	"regexp"
	"strings"
)

// ValidationReport represents API validation results
type ValidationReport struct {
	Valid           bool     `json:"valid"`
	MatchCount      int      `json:"match_count"`
	Coverage        float64  `json:"coverage"`
	Documented      int      `json:"documented"`
	InCode          int      `json:"in_code"`
	Missing         []string `json:"missing,omitempty"`         // In docs but not in code
	Undocumented    []string `json:"undocumented,omitempty"`    // In code but not in docs
	SignatureMismatch []string `json:"signature_mismatch,omitempty"`
}

// Validate compares API docs against Rust code
func Validate(apiDoc *APIDoc, codePath string) (*ValidationReport, error) {
	report := &ValidationReport{
		Valid:             true,
		Missing:           []string{},
		Undocumented:      []string{},
		SignatureMismatch: []string{},
	}

	// Extract functions from code
	codeFuncs, err := extractRustFunctions(codePath)
	if err != nil {
		return nil, err
	}

	report.InCode = len(codeFuncs)
	report.Documented = len(apiDoc.Functions)

	// Build maps for comparison
	docMap := make(map[string]*Function)
	for _, fn := range apiDoc.Functions {
		docMap[fn.Name] = fn
	}

	codeMap := make(map[string]string)
	for _, fn := range codeFuncs {
		codeMap[fn.name] = fn.signature
	}

	// Check documented functions exist in code
	for _, fn := range apiDoc.Functions {
		if _, exists := codeMap[fn.Name]; !exists {
			report.Missing = append(report.Missing, fn.Name)
			report.Valid = false
		} else {
			report.MatchCount++
		}
	}

	// Check code functions are documented
	for _, fn := range codeFuncs {
		if _, exists := docMap[fn.name]; !exists {
			report.Undocumented = append(report.Undocumented, fn.name)
		}
	}

	// Calculate coverage
	if report.InCode > 0 {
		report.Coverage = float64(report.Documented) / float64(report.InCode) * 100.0
	}

	return report, nil
}

type rustFunction struct {
	name      string
	signature string
}

// extractRustFunctions extracts public functions from Rust code
func extractRustFunctions(path string) ([]*rustFunction, error) {
	functions := []*rustFunction{}

	err := filepath.Walk(path, func(filePath string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		if info.IsDir() || !strings.HasSuffix(filePath, ".rs") {
			return nil
		}

		content, err := os.ReadFile(filePath)
		if err != nil {
			return nil // Skip files we can't read
		}

		// Extract pub fn declarations
		pattern := regexp.MustCompile(`pub\s+fn\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\([^)]*\)(?:\s*->\s*[^{]+)?`)
		matches := pattern.FindAllStringSubmatch(string(content), -1)

		for _, match := range matches {
			if len(match) > 1 {
				functions = append(functions, &rustFunction{
					name:      match[1],
					signature: match[0],
				})
			}
		}

		return nil
	})

	if err != nil {
		return nil, err
	}

	return functions, nil
}
