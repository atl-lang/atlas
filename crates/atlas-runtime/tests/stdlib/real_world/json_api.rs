use super::super::*;
// ============================================================================

#[test]
fn test_json_parse_simple_object() {
    let code = r#"
        let jsonStr: string = "{\"name\": \"Alice\", \"age\": 30}";
        let data: json = parseJSON(jsonStr);
        let name: string = data["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "Alice");
}

#[test]
fn test_json_parse_nested_object() {
    let code = r#"
        let jsonStr: string = "{\"user\": {\"name\": \"Bob\", \"email\": \"bob@test.com\"}}";
        let data: json = parseJSON(jsonStr);
        let user: json = data["user"];
        let email: string = user["email"].as_string();
        email
    "#;
    assert_eval_string_with_io(code, "bob@test.com");
}

#[test]
fn test_json_parse_array() {
    let code = r#"
        let jsonStr: string = "[1, 2, 3, 4, 5]";
        let arr: json = parseJSON(jsonStr);
        let first: number = arr[0].as_number();
        first
    "#;
    assert_eval_number_with_io(code, 1.0);
}

#[test]
fn test_json_nested_array_access() {
    let code = r#"
        let jsonStr: string = "{\"numbers\": [10, 20, 30]}";
        let data: json = parseJSON(jsonStr);
        let numbers: json = data["numbers"];
        let second: number = numbers[1].as_number();
        second
    "#;
    assert_eval_number_with_io(code, 20.0);
}

#[test]
fn test_json_api_extract_users() {
    let code = r#"
        let jsonStr: string = "{\"users\": [{\"name\": \"Alice\"}, {\"name\": \"Bob\"}]}";
        let response: json = parseJSON(jsonStr);
        let users: json = response["users"];
        let firstUser: json = users[0];
        let name: string = firstUser["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "Alice");
}

#[test]
fn test_json_extract_multiple_fields() {
    let code = r#"
        let jsonStr: string = "{\"id\": 123, \"name\": \"Product\", \"price\": 29.99}";
        let data: json = parseJSON(jsonStr);
        let id: number = data["id"].as_number();
        let name: string = data["name"].as_string();
        let price: number = data["price"].as_number();
        name + ":" + str(price)
    "#;
    assert_eval_string_with_io(code, "Product:29.99");
}

#[test]
fn test_json_deep_nesting() {
    let code = r#"
        let jsonStr: string = "{\"data\": {\"user\": {\"profile\": {\"name\": \"Charlie\"}}}}";
        let response: json = parseJSON(jsonStr);
        let data: json = response["data"];
        let user: json = data["user"];
        let profile: json = user["profile"];
        let name: string = profile["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "Charlie");
}

#[test]
fn test_json_array_of_objects() {
    let code = r#"
        let jsonStr: string = "[{\"id\": 1}, {\"id\": 2}, {\"id\": 3}]";
        let arr: json = parseJSON(jsonStr);
        let item2: json = arr[1];
        let id: number = item2["id"].as_number();
        id
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_json_boolean_extraction() {
    let code = r#"
        let jsonStr: string = "{\"active\": true, \"verified\": false}";
        let data: json = parseJSON(jsonStr);
        let active: bool = data["active"].as_bool();
        active
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_null_check() {
    let code = r#"
        let jsonStr: string = "{\"value\": null}";
        let data: json = parseJSON(jsonStr);
        let value: json = data["value"];
        jsonIsNull(value)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_missing_key_returns_null() {
    let code = r#"
        let jsonStr: string = "{\"name\": \"Test\"}";
        let data: json = parseJSON(jsonStr);
        let missing: json = data["nonexistent"];
        jsonIsNull(missing)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_build_from_parts() {
    let code = r#"
        let name: string = "Alice";
        let age: number = 30.0;
        let jsonStr: string = "{\"name\":\"" + name + "\",\"age\":" + str(age) + "}";
        let parsed: json = parseJSON(jsonStr);
        let extractedAge: number = parsed["age"].as_number();
        extractedAge
    "#;
    assert_eval_number_with_io(code, 30.0);
}

#[test]
fn test_json_array_length_via_iteration() {
    let code = r#"
        let jsonStr: string = "[1, 2, 3, 4, 5]";
        let arr: json = parseJSON(jsonStr);
        // Access elements to count
        let v0: number = arr[0].as_number();
        let v1: number = arr[1].as_number();
        let v2: number = arr[2].as_number();
        let v3: number = arr[3].as_number();
        let v4: number = arr[4].as_number();
        v0 + v1 + v2 + v3 + v4
    "#;
    assert_eval_number_with_io(code, 15.0);
}

#[test]
fn test_json_mixed_types_in_object() {
    let code = r#"
        let jsonStr: string = "{\"str\": \"hello\", \"num\": 42, \"bool\": true}";
        let data: json = parseJSON(jsonStr);
        let s: string = data["str"].as_string();
        let n: number = data["num"].as_number();
        let b: bool = data["bool"].as_bool();
        s + ":" + str(n) + ":" + str(b)
    "#;
    assert_eval_string_with_io(code, "hello:42:true");
}

#[test]
fn test_json_empty_object() {
    let code = r#"
        let jsonStr: string = "{}";
        let data: json = parseJSON(jsonStr);
        let missing: json = data["anything"];
        jsonIsNull(missing)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_empty_array() {
    let code = r#"
        let jsonStr: string = "[]";
        let arr: json = parseJSON(jsonStr);
        let missing: json = arr[0];
        jsonIsNull(missing)
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_prettify_output() {
    let code = r#"
        let jsonStr: string = "{\"name\":\"Alice\",\"age\":30}";
        let data: json = parseJSON(jsonStr);
        let pretty: string = prettifyJSON(jsonStr, 2.0);
        includes(pretty, "  ")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_validate_before_parse() {
    let code = r#"
        let validJson: string = "{\"test\": true}";
        let invalidJson: string = "{invalid}";
        let valid: bool = isValidJSON(validJson);
        let invalid: bool = isValidJSON(invalidJson);
        valid && !invalid
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_to_json_round_trip() {
    let code = r#"
        let original: string = "{\"key\":\"value\"}";
        let parsed: json = parseJSON(original);
        let serialized: string = toJSON(parsed);
        includes(serialized, "key") && includes(serialized, "value")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_numeric_precision() {
    let code = r#"
        let jsonStr: string = "{\"value\": 123.456}";
        let data: json = parseJSON(jsonStr);
        let value: number = data["value"].as_number();
        value
    "#;
    assert_eval_number_with_io(code, 123.456);
}

#[test]
fn test_json_github_api_style() {
    let code = r#"
        let response: string = "{\"data\": {\"repository\": {\"name\": \"atlas\", \"stars\": 100}}}";
        let json: json = parseJSON(response);
        let data: json = json["data"];
        let repo: json = data["repository"];
        let name: string = repo["name"].as_string();
        let stars: number = repo["stars"].as_number();
        name + ":" + str(stars)
    "#;
    assert_eval_string_with_io(code, "atlas:100");
}

#[test]
fn test_json_array_filter_pattern() {
    let code = r#"
        let jsonStr: string = "[{\"active\":true},{\"active\":false},{\"active\":true}]";
        let arr: json = parseJSON(jsonStr);
        let item0: json = arr[0];
        let item1: json = arr[1];
        let item2: json = arr[2];
        let a0: bool = item0["active"].as_bool();
        let a1: bool = item1["active"].as_bool();
        let a2: bool = item2["active"].as_bool();
        // Count active
        var count: number = 0.0;
        if (a0) { count = count + 1.0; }
        if (a1) { count = count + 1.0; }
        if (a2) { count = count + 1.0; }
        count
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_json_string_escaping() {
    let code = r#"
        let jsonStr: string = "{\"message\": \"Hello\\nWorld\"}";
        let data: json = parseJSON(jsonStr);
        let msg: string = data["message"].as_string();
        includes(msg, "Hello")
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_number_as_string() {
    let code = r#"
        let jsonStr: string = "{\"id\": \"12345\"}";
        let data: json = parseJSON(jsonStr);
        let id: string = data["id"].as_string();
        id
    "#;
    assert_eval_string_with_io(code, "12345");
}

#[test]
fn test_json_nested_arrays() {
    let code = r#"
        let jsonStr: string = "{\"matrix\": [[1,2],[3,4]]}";
        let data: json = parseJSON(jsonStr);
        let matrix: json = data["matrix"];
        let row0: json = matrix[0];
        let val: number = row0[1].as_number();
        val
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_json_api_pagination_meta() {
    let code = r#"
        let response: string = "{\"data\": [], \"meta\": {\"page\": 1, \"total\": 100}}";
        let json: json = parseJSON(response);
        let meta: json = json["meta"];
        let page: number = meta["page"].as_number();
        let total: number = meta["total"].as_number();
        page + total
    "#;
    assert_eval_number_with_io(code, 101.0);
}

#[test]
fn test_json_error_response() {
    let code = r#"
        let response: string = "{\"error\": {\"code\": 404, \"message\": \"Not Found\"}}";
        let json: json = parseJSON(response);
        let error: json = json["error"];
        let code: number = error["code"].as_number();
        let message: string = error["message"].as_string();
        str(code) + ":" + message
    "#;
    assert_eval_string_with_io(code, "404:Not Found");
}

#[test]
fn test_json_transform_data() {
    let code = r#"
        let input: string = "{\"firstName\": \"John\", \"lastName\": \"Doe\"}";
        let data: json = parseJSON(input);
        let first: string = data["firstName"].as_string();
        let last: string = data["lastName"].as_string();
        // Build new structure
        let fullName: string = first + " " + last;
        let output: string = "{\"name\":\"" + fullName + "\"}";
        let result: json = parseJSON(output);
        let name: string = result["name"].as_string();
        name
    "#;
    assert_eval_string_with_io(code, "John Doe");
}

#[test]
fn test_json_conditional_field_access() {
    let code = r#"
        let jsonStr: string = "{\"premium\": true, \"features\": {\"advanced\": true}}";
        let data: json = parseJSON(jsonStr);
        let premium: bool = data["premium"].as_bool();
        var result: bool = false;
        if (premium) {
            let features: json = data["features"];
            let advanced: bool = features["advanced"].as_bool();
            result = advanced;
        }
        result
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_json_minify_compact() {
    let code = r#"
        let jsonStr: string = "{  \"name\" :  \"test\"  }";
        let minified: string = minifyJSON(jsonStr);
        !includes(minified, "  ")
    "#;
    assert_eval_bool_with_io(code, true);
}
