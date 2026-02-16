package api

import (
	"bufio"
	"fmt"
	"os"
	"strings"
)

// APIDoc represents parsed API documentation
type APIDoc struct {
	Title      string      `json:"title"`
	RawContent string      `json:"raw_content,omitempty"`
	Functions  []*Function `json:"functions"`
	Types      []*TypeDef  `json:"types,omitempty"`
	Examples   []string    `json:"examples,omitempty"`
}

// Function represents a documented function
type Function struct {
	Name       string       `json:"name"`
	Signature  string       `json:"signature"`
	Module     string       `json:"module,omitempty"`
	Parameters []*Parameter `json:"parameters,omitempty"`
	Returns    string       `json:"returns,omitempty"`
	Description string      `json:"description,omitempty"`
	Examples   []string     `json:"examples,omitempty"`
}

// Parameter represents a function parameter
type Parameter struct {
	Name string `json:"name"`
	Type string `json:"type"`
	Desc string `json:"desc,omitempty"`
}

// TypeDef represents a type definition
type TypeDef struct {
	Name string `json:"name"`
	Kind string `json:"kind"` // struct, enum, trait, type
	Desc string `json:"desc,omitempty"`
}

// Parse parses API documentation from markdown
func Parse(path string) (*APIDoc, error) {
	// Read raw content first
	rawBytes, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read API file: %w", err)
	}
	rawContent := string(rawBytes)

	file, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("failed to open API file: %w", err)
	}
	defer file.Close()

	doc := &APIDoc{
		RawContent: rawContent,
		Functions:  []*Function{},
		Types:      []*TypeDef{},
		Examples:   []string{},
	}

	scanner := bufio.NewScanner(file)
	var currentFunc *Function
	var inCode bool
	var codeLines []string

	for scanner.Scan() {
		line := scanner.Text()
		trimmed := strings.TrimSpace(line)

		// Extract title
		if strings.HasPrefix(trimmed, "# ") && doc.Title == "" {
			doc.Title = strings.TrimPrefix(trimmed, "# ")
			continue
		}

		// Detect function heading
		if strings.HasPrefix(trimmed, "## ") || strings.HasPrefix(trimmed, "### ") {
			// Save previous function
			if currentFunc != nil {
				doc.Functions = append(doc.Functions, currentFunc)
			}

			// Start new function
			funcName := strings.TrimPrefix(trimmed, "### ")
			funcName = strings.TrimPrefix(funcName, "## ")
			funcName = strings.Trim(funcName, "`")

			currentFunc = &Function{
				Name:       funcName,
				Parameters: []*Parameter{},
				Examples:   []string{},
			}
			continue
		}

		// Extract signature (usually in first code block or after "Signature:")
		if currentFunc != nil && (strings.HasPrefix(trimmed, "**Signature:**") || strings.HasPrefix(trimmed, "```")) {
			if strings.HasPrefix(trimmed, "```") {
				if inCode {
					// End of code block
					code := strings.Join(codeLines, "\n")
					if currentFunc.Signature == "" {
						currentFunc.Signature = extractSignature(code)
					} else {
						currentFunc.Examples = append(currentFunc.Examples, code)
					}
					inCode = false
					codeLines = []string{}
				} else {
					inCode = true
				}
				continue
			}
		}

		if inCode {
			codeLines = append(codeLines, line)
			continue
		}

		// Extract description
		if currentFunc != nil && trimmed != "" && !strings.HasPrefix(trimmed, "**") && !strings.HasPrefix(trimmed, "#") {
			if currentFunc.Description == "" {
				currentFunc.Description = trimmed
			} else {
				currentFunc.Description += " " + trimmed
			}
		}
	}

	// Add last function
	if currentFunc != nil {
		doc.Functions = append(doc.Functions, currentFunc)
	}

	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("failed to scan API file: %w", err)
	}

	return doc, nil
}

// extractSignature extracts function signature from code
func extractSignature(code string) string {
	lines := strings.Split(code, "\n")
	for _, line := range lines {
		trimmed := strings.TrimSpace(line)
		// Look for function definition
		if strings.Contains(trimmed, "fn ") || strings.Contains(trimmed, "func ") || strings.Contains(trimmed, "function ") {
			return trimmed
		}
	}
	return strings.TrimSpace(code)
}

// FindFunction finds a function by name
func (d *APIDoc) FindFunction(name string) *Function {
	for _, fn := range d.Functions {
		if fn.Name == name {
			return fn
		}
	}
	return nil
}
