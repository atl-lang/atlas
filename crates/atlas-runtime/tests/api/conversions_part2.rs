use super::*;

#[test]
fn test_vec_from_atlas_wrong_type() {
    let value = Value::Number(42.0);
    let result: Result<Vec<f64>, _> = FromAtlas::from_atlas(&value);
    assert!(result.is_err());
}

#[test]
fn test_vec_from_atlas_element_type_mismatch() {
    let arr = vec![
        Value::Number(1.0),
        Value::String(Arc::new("oops".to_string())),
    ];
    let value = Value::array(arr);
    let result: Result<Vec<f64>, _> = FromAtlas::from_atlas(&value);
    assert!(result.is_err());
    match result.unwrap_err() {
        ConversionError::ArrayElementTypeMismatch {
            index,
            expected,
            found,
        } => {
            assert_eq!(index, 1);
            assert_eq!(expected, "number");
            assert_eq!(found, "string");
        }
        _ => panic!("Expected ArrayElementTypeMismatch error"),
    }
}

// Nested Conversion Tests

#[test]
fn test_nested_vec_option_f64() {
    let data = vec![Some(1.0), None, Some(3.0)];
    let value = data.to_atlas();

    // Convert back
    let result: Vec<Option<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, vec![Some(1.0), None, Some(3.0)]);
}

#[test]
fn test_nested_vec_option_string() {
    let data = vec![Some("hello".to_string()), None, Some("world".to_string())];
    let value = data.to_atlas();

    // Convert back
    let result: Vec<Option<String>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(
        result,
        vec![Some("hello".to_string()), None, Some("world".to_string())]
    );
}

#[test]
fn test_nested_option_vec_f64() {
    let data: Option<Vec<f64>> = Some(vec![1.0, 2.0, 3.0]);
    let value = data.to_atlas();

    // Convert back
    let result: Option<Vec<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, Some(vec![1.0, 2.0, 3.0]));
}

#[test]
fn test_nested_option_vec_none() {
    let data: Option<Vec<f64>> = None;
    let value = data.to_atlas();

    // Convert back
    let result: Option<Vec<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(result, None);
}

// HashMap Conversion Tests

#[test]
fn test_hashmap_to_atlas_creates_json() {
    let mut map = HashMap::new();
    map.insert("x".to_string(), 1.0);
    map.insert("y".to_string(), 2.0);

    let value = map.to_atlas();
    assert!(matches!(value, Value::JsonValue(_)));
}

#[test]
fn test_hashmap_string_to_atlas() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), "Alice".to_string());
    map.insert("city".to_string(), "Boston".to_string());

    let value = map.to_atlas();
    assert!(matches!(value, Value::JsonValue(_)));
}

// Bidirectional Roundtrip Tests

#[test]
fn test_roundtrip_f64() {
    let original = 42.5;
    let value = original.to_atlas();
    let result: f64 = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_string() {
    let original = "hello world".to_string();
    let value = original.clone().to_atlas();
    let result: String = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_bool_true() {
    let original = true;
    let value = original.to_atlas();
    let result: bool = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_bool_false() {
    let original = false;
    let value = original.to_atlas();
    let result: bool = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_option_some() {
    let original = Some(42.0);
    let value = original.to_atlas();
    let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_option_none() {
    let original: Option<f64> = None;
    let value = original.to_atlas();
    let result: Option<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_vec_f64() {
    let original = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let value = original.clone().to_atlas();
    let result: Vec<f64> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_vec_string() {
    let original = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let value = original.clone().to_atlas();
    let result: Vec<String> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

#[test]
fn test_roundtrip_vec_option_f64() {
    let original = vec![Some(1.0), None, Some(3.0), None, Some(5.0)];
    let value = original.clone().to_atlas();
    let result: Vec<Option<f64>> = FromAtlas::from_atlas(&value).unwrap();
    assert_eq!(original, result);
}

// Error Message Quality Tests

#[test]
fn test_conversion_error_display_type_mismatch() {
    let error = ConversionError::TypeMismatch {
        expected: "number".to_string(),
        found: "string".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("number"));
    assert!(message.contains("string"));
    assert!(message.contains("mismatch"));
}

#[test]
fn test_conversion_error_display_array_element() {
    let error = ConversionError::ArrayElementTypeMismatch {
        index: 5,
        expected: "number".to_string(),
        found: "bool".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("5"));
    assert!(message.contains("number"));
    assert!(message.contains("bool"));
    assert!(message.contains("Array"));
}

#[test]
fn test_conversion_error_display_object_value() {
    let error = ConversionError::ObjectValueTypeMismatch {
        key: "name".to_string(),
        expected: "string".to_string(),
        found: "number".to_string(),
    };
    let message = format!("{}", error);
    assert!(message.contains("name"));
    assert!(message.contains("string"));
    assert!(message.contains("number"));
    assert!(message.contains("Object"));
}

