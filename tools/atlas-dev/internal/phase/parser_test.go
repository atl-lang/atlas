package phase

import (
	"testing"
)

func TestParse(t *testing.T) {
	tests := []struct {
		name         string
		input        string
		wantCategory string
		wantName     string
		wantErr      bool
	}{
		{
			"full path with phases dir",
			"phases/stdlib/phase-07b-hashset.md",
			"stdlib",
			"phase-07b-hashset",
			false,
		},
		{
			"path without phases dir",
			"stdlib/phase-07b-hashset.md",
			"stdlib",
			"phase-07b-hashset",
			false,
		},
		{
			"name only with extension",
			"phase-01-foundation.md",
			"foundation",
			"phase-01-foundation",
			false,
		},
		{
			"path without extension",
			"phases/stdlib/phase-07b-hashset",
			"stdlib",
			"phase-07b-hashset",
			false,
		},
		{
			"empty path",
			"",
			"",
			"",
			true,
		},
		{
			"multi-word category",
			"phases/bytecode-vm/phase-01-basic-vm.md",
			"bytecode-vm",
			"phase-01-basic-vm",
			false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			info, err := Parse(tt.input)

			if tt.wantErr {
				if err == nil {
					t.Error("expected error, got nil")
				}
				return
			}

			if err != nil {
				t.Errorf("unexpected error: %v", err)
				return
			}

			if info.Category != tt.wantCategory {
				t.Errorf("category = %q, want %q", info.Category, tt.wantCategory)
			}

			if info.Name != tt.wantName {
				t.Errorf("name = %q, want %q", info.Name, tt.wantName)
			}
		})
	}
}

func TestIsValidCategory(t *testing.T) {
	tests := []struct {
		category string
		want     bool
	}{
		{"foundation", true},
		{"stdlib", true},
		{"bytecode-vm", true},
		{"frontend", true},
		{"typing", true},
		{"interpreter", true},
		{"cli", true},
		{"lsp", true},
		{"polish", true},
		{"invalid", false},
		{"", false},
	}

	for _, tt := range tests {
		t.Run(tt.category, func(t *testing.T) {
			got := IsValidCategory(tt.category)
			if got != tt.want {
				t.Errorf("IsValidCategory(%q) = %v, want %v", tt.category, got, tt.want)
			}
		})
	}
}
