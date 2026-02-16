package spec

import (
	"bufio"
	"fmt"
	"os"
	"regexp"
	"strings"
)

// Spec represents a parsed specification document
type Spec struct {
	Title         string          `json:"title"`
	Version       string          `json:"version,omitempty"`
	Status        string          `json:"status,omitempty"`
	RawContent    string          `json:"raw_content,omitempty"`
	Outline       []string        `json:"outline,omitempty"`
	Sections      []*Section      `json:"sections"`
	CodeBlocks    []*CodeBlock    `json:"code_blocks,omitempty"`
	GrammarRules  []*GrammarRule  `json:"grammar_rules,omitempty"`
	Grammar       []*GrammarRule  `json:"grammar,omitempty"`
	References    []string        `json:"references,omitempty"`
}

// Section represents a section in the spec
type Section struct {
	Level       int          `json:"level"`
	Title       string       `json:"title"`
	Content     string       `json:"content,omitempty"`
	CodeBlocks  []*CodeBlock `json:"code_blocks,omitempty"`
	Subsections []*Section   `json:"subsections,omitempty"`
}

// CodeBlock represents a code block in the spec
type CodeBlock struct {
	Language string `json:"language"`
	Code     string `json:"code"`
	Section  string `json:"section,omitempty"`
}

// GrammarRule represents an EBNF grammar rule
type GrammarRule struct {
	Name       string `json:"name"`
	Definition string `json:"definition"`
	Line       int    `json:"line,omitempty"`
}

// Parse parses a specification markdown file
func Parse(path string) (*Spec, error) {
	// Read raw content first
	rawBytes, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("failed to read spec file: %w", err)
	}
	rawContent := string(rawBytes)

	file, err := os.Open(path)
	if err != nil {
		return nil, fmt.Errorf("failed to open spec file: %w", err)
	}
	defer file.Close()

	spec := &Spec{
		RawContent:   rawContent,
		Outline:      []string{},
		Sections:     []*Section{},
		CodeBlocks:   []*CodeBlock{},
		GrammarRules: []*GrammarRule{},
		References:   []string{},
	}

	scanner := bufio.NewScanner(file)
	lineNum := 0

	var (
		currentSection *Section
		sectionStack   []*Section
		inCodeBlock    bool
		codeLanguage   string
		codeLines      []string
		inMetadata     bool
	)

	for scanner.Scan() {
		line := scanner.Text()
		lineNum++
		trimmed := strings.TrimSpace(line)

		// Extract title
		if strings.HasPrefix(trimmed, "# ") && spec.Title == "" {
			spec.Title = strings.TrimPrefix(trimmed, "# ")
			continue
		}

		// Extract metadata
		if strings.HasPrefix(trimmed, "**Version:**") {
			spec.Version = extractValue(trimmed, "**Version:**")
			inMetadata = true
			continue
		}
		if strings.HasPrefix(trimmed, "**Status:**") {
			spec.Status = extractValue(trimmed, "**Status:**")
			continue
		}
		if inMetadata && trimmed == "---" {
			inMetadata = false
			continue
		}

		// Detect code block start/end
		if strings.HasPrefix(trimmed, "```") {
			if inCodeBlock {
				// End of code block
				code := strings.Join(codeLines, "\n")
				block := &CodeBlock{
					Language: codeLanguage,
					Code:     code,
				}
				if currentSection != nil {
					block.Section = currentSection.Title
					currentSection.CodeBlocks = append(currentSection.CodeBlocks, block)
				}
				spec.CodeBlocks = append(spec.CodeBlocks, block)

				// Check if it's a grammar rule (EBNF)
				if codeLanguage == "ebnf" || codeLanguage == "grammar" {
					rules := parseGrammarRules(code, lineNum-len(codeLines))
					spec.GrammarRules = append(spec.GrammarRules, rules...)
				}

				inCodeBlock = false
				codeLines = []string{}
			} else {
				// Start of code block
				inCodeBlock = true
				codeLanguage = strings.TrimPrefix(trimmed, "```")
			}
			continue
		}

		if inCodeBlock {
			codeLines = append(codeLines, line)
			continue
		}

		// Parse headings (sections)
		if strings.HasPrefix(trimmed, "#") {
			level := 0
			for _, c := range trimmed {
				if c == '#' {
					level++
				} else {
					break
				}
			}

			if level > 1 { // Skip level 1 (title)
				title := strings.TrimSpace(strings.TrimPrefix(trimmed, strings.Repeat("#", level)))

				// Add to outline
				spec.Outline = append(spec.Outline, title)

				section := &Section{
					Level:       level,
					Title:       title,
					Content:     "",
					CodeBlocks:  []*CodeBlock{},
					Subsections: []*Section{},
				}

				// Build section hierarchy
				for len(sectionStack) > 0 && sectionStack[len(sectionStack)-1].Level >= level {
					sectionStack = sectionStack[:len(sectionStack)-1]
				}

				if len(sectionStack) > 0 {
					parent := sectionStack[len(sectionStack)-1]
					parent.Subsections = append(parent.Subsections, section)
				} else {
					spec.Sections = append(spec.Sections, section)
				}

				sectionStack = append(sectionStack, section)
				currentSection = section
			}
			continue
		}

		// Extract references [text](path)
		refs := extractReferences(line)
		spec.References = append(spec.References, refs...)

		// Collect section content
		if currentSection != nil && trimmed != "" && !strings.HasPrefix(trimmed, "**") {
			if currentSection.Content != "" {
				currentSection.Content += " "
			}
			currentSection.Content += trimmed
		}
	}

	if err := scanner.Err(); err != nil {
		return nil, fmt.Errorf("failed to scan spec file: %w", err)
	}

	// Validate
	if spec.Title == "" {
		return nil, fmt.Errorf("missing spec title")
	}

	// Set Grammar field (for backward compatibility)
	spec.Grammar = spec.GrammarRules

	return spec, nil
}

// extractValue extracts value after a label
func extractValue(line, label string) string {
	parts := strings.SplitN(line, label, 2)
	if len(parts) < 2 {
		return ""
	}
	return strings.TrimSpace(parts[1])
}

// extractReferences extracts markdown references from a line
func extractReferences(line string) []string {
	refs := []string{}

	// Match [text](path) and [text](path#section)
	pattern := regexp.MustCompile(`\[([^\]]+)\]\(([^)]+)\)`)
	matches := pattern.FindAllStringSubmatch(line, -1)

	for _, match := range matches {
		if len(match) > 2 {
			refs = append(refs, match[2])
		}
	}

	return refs
}

// parseGrammarRules extracts EBNF grammar rules from code
func parseGrammarRules(code string, startLine int) []*GrammarRule {
	rules := []*GrammarRule{}
	lines := strings.Split(code, "\n")

	// Simple EBNF parser: rule_name = definition
	pattern := regexp.MustCompile(`^\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)`)

	for i, line := range lines {
		matches := pattern.FindStringSubmatch(line)
		if len(matches) > 2 {
			rule := &GrammarRule{
				Name:       matches[1],
				Definition: strings.TrimSpace(matches[2]),
				Line:       startLine + i,
			}
			rules = append(rules, rule)
		}
	}

	return rules
}

// FindSection finds a section by title (case-insensitive)
func (s *Spec) FindSection(title string) *Section {
	title = strings.ToLower(title)
	return findSectionRecursive(s.Sections, title)
}

func findSectionRecursive(sections []*Section, title string) *Section {
	for _, section := range sections {
		if strings.ToLower(section.Title) == title {
			return section
		}
		if found := findSectionRecursive(section.Subsections, title); found != nil {
			return found
		}
	}
	return nil
}

// GetOutline returns section titles for document outline
func (s *Spec) GetOutline() []string {
	outline := []string{}
	addSectionToOutline(s.Sections, &outline, "")
	return outline
}

func addSectionToOutline(sections []*Section, outline *[]string, prefix string) {
	for _, section := range sections {
		indent := strings.Repeat("  ", section.Level-2)
		*outline = append(*outline, indent+section.Title)
		addSectionToOutline(section.Subsections, outline, prefix+"  ")
	}
}
