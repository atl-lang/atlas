package spec

import (
	"fmt"
	"regexp"
	"strings"
)

// GrammarValidation represents validation results for EBNF grammar
type GrammarValidation struct {
	Valid       bool     `json:"valid"`
	TotalRules  int      `json:"total_rules"`
	Errors      []string `json:"errors,omitempty"`
	Warnings    []string `json:"warnings,omitempty"`
	Undefined   []string `json:"undefined,omitempty"`
	Circular    []string `json:"circular,omitempty"`
}

// ValidateGrammar validates EBNF grammar rules
func ValidateGrammar(rules []*GrammarRule) *GrammarValidation {
	validation := &GrammarValidation{
		Valid:      true,
		TotalRules: len(rules),
		Errors:     []string{},
		Warnings:   []string{},
		Undefined:  []string{},
		Circular:   []string{},
	}

	if len(rules) == 0 {
		validation.Errors = append(validation.Errors, "no grammar rules found")
		validation.Valid = false
		return validation
	}

	// Build map of defined rules
	defined := make(map[string]bool)
	for _, rule := range rules {
		defined[rule.Name] = true
	}

	// Check each rule
	for _, rule := range rules {
		// Validate EBNF syntax
		if err := validateEBNFSyntax(rule.Definition); err != nil {
			validation.Errors = append(validation.Errors,
				fmt.Sprintf("rule %s: %v", rule.Name, err))
			validation.Valid = false
		}

		// Extract referenced non-terminals
		refs := extractNonTerminals(rule.Definition)
		for _, ref := range refs {
			if !defined[ref] {
				// Check if it's a built-in terminal
				if !isBuiltinTerminal(ref) {
					validation.Undefined = append(validation.Undefined, ref)
					validation.Errors = append(validation.Errors,
						fmt.Sprintf("rule %s references undefined non-terminal: %s", rule.Name, ref))
					validation.Valid = false
				}
			}
		}
	}

	// Check for circular dependencies
	circular := findCircularDependencies(rules)
	if len(circular) > 0 {
		validation.Circular = circular
		validation.Warnings = append(validation.Warnings,
			fmt.Sprintf("circular dependencies detected: %v", circular))
	}

	return validation
}

// validateEBNFSyntax validates basic EBNF syntax
func validateEBNFSyntax(definition string) error {
	// Check for balanced brackets
	brackets := map[rune]rune{
		'[': ']',
		'{': '}',
		'(': ')',
	}

	stack := []rune{}
	for _, c := range definition {
		if _, isOpen := brackets[c]; isOpen {
			stack = append(stack, c)
		} else {
			for open, close := range brackets {
				if c == close {
					if len(stack) == 0 {
						return fmt.Errorf("unmatched closing bracket: %c", c)
					}
					last := stack[len(stack)-1]
					if last != open {
						return fmt.Errorf("mismatched brackets: expected %c, got %c", brackets[last], c)
					}
					stack = stack[:len(stack)-1]
					break
				}
			}
		}
	}

	if len(stack) > 0 {
		return fmt.Errorf("unmatched opening bracket: %c", stack[0])
	}

	// Check for invalid characters (basic check)
	// EBNF allows: letters, numbers, _, -, |, *, +, ?, =, ;, (), [], {}, ", ', spaces
	validPattern := regexp.MustCompile(`^[a-zA-Z0-9_\-|*+?=;(){}\[\]"'\s<>,.]*$`)
	if !validPattern.MatchString(definition) {
		return fmt.Errorf("invalid characters in definition")
	}

	return nil
}

// extractNonTerminals extracts non-terminal references from definition
func extractNonTerminals(definition string) []string {
	refs := []string{}

	// Match identifiers (non-terminals are typically identifiers)
	// This is a simplified approach - proper parsing would be more accurate
	pattern := regexp.MustCompile(`\b([a-z_][a-zA-Z0-9_]*)\b`)
	matches := pattern.FindAllStringSubmatch(definition, -1)

	seen := make(map[string]bool)
	for _, match := range matches {
		if len(match) > 1 {
			ref := match[1]
			// Skip common EBNF keywords
			if !isEBNFKeyword(ref) && !seen[ref] {
				refs = append(refs, ref)
				seen[ref] = true
			}
		}
	}

	return refs
}

// isEBNFKeyword checks if a word is an EBNF keyword
func isEBNFKeyword(word string) bool {
	keywords := map[string]bool{
		"empty": true,
		"or":    true,
		"and":   true,
	}
	return keywords[strings.ToLower(word)]
}

// isBuiltinTerminal checks if a reference is a built-in terminal
func isBuiltinTerminal(ref string) bool {
	builtins := map[string]bool{
		"digit":       true,
		"letter":      true,
		"char":        true,
		"whitespace":  true,
		"newline":     true,
		"identifier":  true,
		"string":      true,
		"number":      true,
		"boolean":     true,
		"null":        true,
		"true":        true,
		"false":       true,
	}
	return builtins[strings.ToLower(ref)]
}

// findCircularDependencies detects circular dependencies in grammar
func findCircularDependencies(rules []*GrammarRule) []string {
	// Build dependency graph
	deps := make(map[string][]string)
	for _, rule := range rules {
		refs := extractNonTerminals(rule.Definition)
		deps[rule.Name] = refs
	}

	// Find cycles using DFS
	circular := []string{}
	visited := make(map[string]bool)
	inStack := make(map[string]bool)

	var dfs func(node string) bool
	dfs = func(node string) bool {
		visited[node] = true
		inStack[node] = true

		for _, neighbor := range deps[node] {
			if !visited[neighbor] {
				if dfs(neighbor) {
					return true
				}
			} else if inStack[neighbor] {
				circular = append(circular, node+" -> "+neighbor)
				return true
			}
		}

		inStack[node] = false
		return false
	}

	for _, rule := range rules {
		if !visited[rule.Name] {
			if dfs(rule.Name) {
				break // Stop after finding first cycle
			}
		}
	}

	return circular
}

// CompareToParser compares grammar to parser implementation (stub)
func CompareToParser(rules []*GrammarRule, parserPath string) (map[string]interface{}, error) {
	// This would require parsing the parser code to extract what it parses
	// For now, return a placeholder
	return map[string]interface{}{
		"status": "not_implemented",
		"msg":    "parser comparison not yet implemented",
	}, nil
}
