# StateStore - Session State Persistence

**Status**: ✅ Complete

## Overview

StateStore provides session state persistence for Hydra Atlas. It manages reading and writing session state to disk as JSON files.

## API

### `new_store(session_id: string) -> string`

Creates a state store path for a session.

```atlas
let store_path: string = new_store("my-session");
// Returns: ".hydra/sessions/my-session.json"
```

### `save_state(store_path: string, json_str: string) -> bool`

Saves state to disk as a JSON string. Automatically creates the directory if it doesn't exist.

```atlas
let json: string = "{\"count\":42,\"active\":true}";
let success: bool = save_state(store_path, json);
```

### `load_state(store_path: string) -> string`

Loads state from disk and returns the JSON string.

```atlas
let json: string = load_state(store_path);
```

### `has_state(store_path: string) -> bool`

Checks if a state file exists.

```atlas
if (has_state(store_path)) {
    // State exists
}
```

### `clear_state(store_path: string) -> bool`

Clears state by writing an empty JSON object.

```atlas
let success: bool = clear_state(store_path);
// File now contains: {}
```

### `delete_state(store_path: string) -> bool`

Deletes the state file.

```atlas
let success: bool = delete_state(store_path);
```

## Usage Example

```atlas
import { new_store, save_state, load_state, has_state } from "./statestore";

fn main() -> void {
    // Create store
    let store: string = new_store("session-123");

    // Save state
    let state_json: string = "{\"server\":\"mcp-1\",\"restarts\":0}";
    save_state(store, state_json);

    // Check if exists
    if (has_state(store)) {
        // Load state
        let loaded: string = load_state(store);

        // Parse JSON
        let parse_result: Result<json, string> = parseJSON(loaded);
        let data: json = match parse_result {
            Ok(val) => val,
            Err(_) => null
        };

        // Modify and save back
        // (In real code, you'd modify the JSON object)
        let updated: string = toJSON(data);
        save_state(store, updated);
    }
}
```

## Implementation Notes

- Uses Atlas file I/O: `readFile()`, `writeFile()`, `fsIsFile()`, `fsIsDir()`, `createDir()`
- Automatically creates `.hydra/sessions/` directory on first write
- All operations use JSON strings - use `parseJSON()` and `toJSON()` for object manipulation
- Simple error handling: throws on file I/O errors

## Tests

Run tests:

```bash
cd statestore
atlas run test_simple.atl  # Basic string API test
atlas run test_json.atl    # JSON parse/stringify test
```

All tests passing ✅

## Battle Test Results

StateStore demonstrates:

- ✅ File I/O operations (`readFile`, `writeFile`)
- ✅ Directory operations (`fsIsDir`, `createDir`)
- ✅ JSON serialization (`parseJSON`, `toJSON`)
- ✅ Conditional logic and error handling
- ✅ Module exports and imports

**Conclusion**: Atlas v0.2 is fully capable of file-based state management.
