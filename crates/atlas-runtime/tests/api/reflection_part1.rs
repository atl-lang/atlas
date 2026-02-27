use super::*;

// --- Reflection ---

// Reflection API integration tests
//
// Tests reflection and introspection functionality with both
// interpreter and VM execution engines (100% parity required).

// ============================================================================
// Type Information Tests
// ============================================================================

#[test]
fn test_type_info_from_primitive_types() {
    let num_info = TypeInfo::from_type(&Type::Number);
    assert_eq!(num_info.name, "number");
    assert_eq!(num_info.kind, TypeKind::Number);
    assert!(num_info.is_primitive());

    let str_info = TypeInfo::from_type(&Type::String);
    assert_eq!(str_info.name, "string");
    assert_eq!(str_info.kind, TypeKind::String);
    assert!(str_info.is_primitive());

    let bool_info = TypeInfo::from_type(&Type::Bool);
    assert_eq!(bool_info.name, "bool");
    assert_eq!(bool_info.kind, TypeKind::Bool);
    assert!(bool_info.is_primitive());

    let null_info = TypeInfo::from_type(&Type::Null);
    assert_eq!(null_info.name, "null");
    assert_eq!(null_info.kind, TypeKind::Null);
    assert!(null_info.is_primitive());
}

#[test]
fn test_type_info_from_array_type() {
    let arr_type = Type::Array(Box::new(Type::Number));
    let info = TypeInfo::from_type(&arr_type);

    assert_eq!(info.name, "number[]");
    assert_eq!(info.kind, TypeKind::Array);
    assert!(info.is_array());
    assert!(!info.is_primitive());

    assert!(info.element_type.is_some());
    let elem = info.element_type.as_ref().unwrap();
    assert_eq!(elem.name, "number");
    assert_eq!(elem.kind, TypeKind::Number);
}

#[test]
fn test_type_info_from_function_type() {
    let func_type = Type::Function {
        type_params: vec![],
        params: vec![Type::Number, Type::String],
        return_type: Box::new(Type::Bool),
    };

    let info = TypeInfo::from_type(&func_type);

    assert_eq!(info.name, "function");
    assert_eq!(info.kind, TypeKind::Function);
    assert!(info.is_function());
    assert!(!info.is_primitive());

    assert_eq!(info.parameters.len(), 2);
    assert_eq!(info.parameters[0].name, "number");
    assert_eq!(info.parameters[1].name, "string");

    assert!(info.return_type.is_some());
    let ret = info.return_type.as_ref().unwrap();
    assert_eq!(ret.name, "bool");
}

#[test]
fn test_type_info_from_generic_type() {
    let gen_type = Type::Generic {
        name: "Result".to_string(),
        type_args: vec![Type::Number, Type::String],
    };

    let info = TypeInfo::from_type(&gen_type);

    assert_eq!(info.name, "Result<number, string>");
    assert_eq!(info.kind, TypeKind::Generic);
    assert!(info.is_generic());

    assert_eq!(info.type_args.len(), 2);
    assert_eq!(info.type_args[0].name, "number");
    assert_eq!(info.type_args[1].name, "string");
}

#[test]
fn test_type_info_function_signature() {
    let func_type = Type::Function {
        type_params: vec![],
        params: vec![Type::Number, Type::String],
        return_type: Box::new(Type::Bool),
    };

    let info = TypeInfo::from_type(&func_type);
    let sig = info.function_signature().unwrap();

    assert_eq!(sig, "(number, string) -> bool");
}

#[test]
fn test_type_info_describe() {
    let num_info = TypeInfo::from_type(&Type::Number);
    assert_eq!(num_info.describe(), "primitive number type");

    let arr_info = TypeInfo::from_type(&Type::Array(Box::new(Type::String)));
    assert_eq!(arr_info.describe(), "array of string");

    let func_type = Type::Function {
        type_params: vec![],
        params: vec![Type::Number],
        return_type: Box::new(Type::Void),
    };
    let func_info = TypeInfo::from_type(&func_type);
    assert_eq!(func_info.describe(), "function (number) -> void");
}

#[test]
fn test_type_info_nested_arrays() {
    // number[][]
    let nested = Type::Array(Box::new(Type::Array(Box::new(Type::Number))));
    let info = TypeInfo::from_type(&nested);

    assert_eq!(info.name, "number[][]");
    assert!(info.is_array());

    let outer_elem = info.element_type.as_ref().unwrap();
    assert_eq!(outer_elem.name, "number[]");
    assert!(outer_elem.is_array());

    let inner_elem = outer_elem.element_type.as_ref().unwrap();
    assert_eq!(inner_elem.name, "number");
    assert!(inner_elem.is_primitive());
}

#[test]
fn test_type_info_equality() {
    let info1 = TypeInfo::from_type(&Type::Number);
    let info2 = TypeInfo::from_type(&Type::Number);
    let info3 = TypeInfo::from_type(&Type::String);

    assert_eq!(info1, info2);
    assert_ne!(info1, info3);
}

// ============================================================================
// Value Information Tests
// ============================================================================

#[test]
fn test_value_info_type_name() {
    let num_info = ValueInfo::new(Value::Number(42.0));
    assert_eq!(num_info.type_name(), "number");

    let str_info = ValueInfo::new(Value::string("test"));
    assert_eq!(str_info.type_name(), "string");

    let arr_info = ValueInfo::new(Value::array(vec![]));
    assert_eq!(arr_info.type_name(), "array");
}

#[test]
fn test_value_info_get_length() {
    let arr = Value::array(vec![
        Value::Number(1.0),
        Value::Number(2.0),
        Value::Number(3.0),
    ]);
    let info = ValueInfo::new(arr);
    assert_eq!(info.get_length(), Some(3));

    let str_val = Value::string("hello");
    let info = ValueInfo::new(str_val);
    assert_eq!(info.get_length(), Some(5));

    let num = Value::Number(42.0);
    let info = ValueInfo::new(num);
    assert_eq!(info.get_length(), None);
}

#[test]
fn test_value_info_is_empty() {
    let empty_arr = Value::array(vec![]);
    assert!(ValueInfo::new(empty_arr).is_empty());

    let empty_str = Value::string("");
    assert!(ValueInfo::new(empty_str).is_empty());

    let non_empty = Value::array(vec![Value::Number(1.0)]);
    assert!(!ValueInfo::new(non_empty).is_empty());
}

#[test]
fn test_value_info_type_checks() {
    let num_info = ValueInfo::new(Value::Number(42.0));
    assert!(num_info.is_number());
    assert!(!num_info.is_string());
    assert!(!num_info.is_bool());
    assert!(!num_info.is_null());

    let str_info = ValueInfo::new(Value::string("test"));
    assert!(str_info.is_string());
    assert!(!str_info.is_number());

    let bool_info = ValueInfo::new(Value::Bool(true));
    assert!(bool_info.is_bool());
    assert!(!bool_info.is_number());

    let null_info = ValueInfo::new(Value::Null);
    assert!(null_info.is_null());
    assert!(!null_info.is_number());
}

