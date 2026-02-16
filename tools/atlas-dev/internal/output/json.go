package output

import (
	"encoding/json"
	"os"
)

// Success outputs compact JSON with ok=true
func Success(data map[string]interface{}) error {
	// Add ok=true
	data["ok"] = true

	// Remove null/empty fields
	cleaned := removeEmpty(data)

	// Compact encoding (no spaces)
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetEscapeHTML(false)
	return encoder.Encode(cleaned)
}

// Error outputs compact JSON with ok=false and error message
func Error(err error, details map[string]interface{}) error {
	data := map[string]interface{}{
		"ok":  false,
		"err": err.Error(),
	}

	// Add details
	for k, v := range details {
		data[k] = v
	}

	// Remove null/empty fields
	cleaned := removeEmpty(data)

	// Compact encoding
	encoder := json.NewEncoder(os.Stdout)
	encoder.SetEscapeHTML(false)
	return encoder.Encode(cleaned)
}

// removeEmpty removes null/empty values
func removeEmpty(m map[string]interface{}) map[string]interface{} {
	result := make(map[string]interface{})
	for k, v := range m {
		if !isEmpty(v) {
			result[k] = v
		}
	}
	return result
}

// isEmpty checks if value is empty/null
func isEmpty(v interface{}) bool {
	if v == nil {
		return true
	}

	switch val := v.(type) {
	case string:
		return val == ""
	case []interface{}:
		return len(val) == 0
	case map[string]interface{}:
		return len(val) == 0
	default:
		return false
	}
}
