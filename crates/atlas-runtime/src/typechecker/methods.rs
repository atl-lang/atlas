//! Method table infrastructure for method resolution

use crate::types::Type;
use std::collections::HashMap;

/// Signature of a method
#[derive(Debug, Clone)]
pub struct MethodSignature {
    /// Argument types (excluding the receiver/target)
    pub arg_types: Vec<Type>,
    /// Return type
    pub return_type: Type,
}

/// Method table for resolving method calls
pub struct MethodTable {
    /// Map of (type_name, method_name) -> MethodSignature
    methods: HashMap<(String, String), MethodSignature>,
}

impl MethodTable {
    /// Create a new method table with built-in methods registered
    pub fn new() -> Self {
        let mut table = Self {
            methods: HashMap::new(),
        };
        table.populate_builtin_methods();
        table
    }

    /// Register a method for a type
    pub fn register(
        &mut self,
        type_name: &str,
        method_name: &str,
        arg_types: Vec<Type>,
        return_type: Type,
    ) {
        let key = (type_name.to_string(), method_name.to_string());
        let sig = MethodSignature {
            arg_types,
            return_type,
        };
        self.methods.insert(key, sig);
    }

    /// Return all method names registered for `type_name` (used for typo suggestions).
    pub fn method_names_for_type_str<'a>(
        &'a self,
        type_name: &str,
    ) -> impl Iterator<Item = &'a str> {
        let owned = type_name.to_owned();
        self.methods
            .keys()
            .filter(move |(t, _)| *t == owned)
            .map(|(_, m)| m.as_str())
    }

    /// Look up a method for a type
    pub fn lookup(&self, receiver_type: &Type, method_name: &str) -> Option<MethodSignature> {
        let receiver_type = receiver_type.normalized();
        // Convert Type to string for lookup
        let type_name = match receiver_type {
            Type::JsonValue => "json",
            Type::String => "string",
            Type::Number => "number",
            Type::Bool => "bool",
            Type::Array(elem) => return self.array_method_signature(method_name, &elem),
            Type::Generic {
                ref name,
                ref type_args,
            } if name == "HashMap" => {
                return self.hashmap_method_signature(
                    method_name,
                    type_args.first().unwrap_or(&Type::Unknown),
                    type_args.get(1).unwrap_or(&Type::Unknown),
                )
            }
            Type::Generic {
                ref name,
                ref type_args,
            } if name == "HashSet" => {
                return self.hashset_method_signature(
                    method_name,
                    type_args.first().unwrap_or(&Type::Unknown),
                )
            }
            Type::Generic {
                ref name,
                ref type_args,
            } if name == "Queue" => {
                return self.queue_method_signature(
                    method_name,
                    type_args.first().unwrap_or(&Type::Unknown),
                )
            }
            Type::Generic {
                ref name,
                ref type_args,
            } if name == "Stack" => {
                return self.stack_method_signature(
                    method_name,
                    type_args.first().unwrap_or(&Type::Unknown),
                )
            }
            Type::Generic {
                ref name,
                ref type_args,
            } if name == "Option" => {
                return self.option_method_signature(
                    method_name,
                    type_args.first().unwrap_or(&Type::Unknown),
                )
            }
            Type::Generic {
                ref name,
                ref type_args,
            } if name == "Result" => {
                return self.result_method_signature(
                    method_name,
                    type_args.first().unwrap_or(&Type::Unknown),
                    type_args.get(1).unwrap_or(&Type::Unknown),
                )
            }
            // H-231: DateTime, Regex, HttpResponse instance methods
            Type::Generic { ref name, .. } if name == "DateTime" => "DateTime",
            Type::Generic { ref name, .. } if name == "Regex" => "Regex",
            Type::Generic { ref name, .. } if name == "HttpResponse" => "HttpResponse",
            // B18: ProcessOutput instance methods
            Type::Generic { ref name, .. } if name == "ProcessOutput" => "ProcessOutput",
            // B33: Future instance methods
            Type::Generic { ref name, .. } if name == "Future" => "Future",
            _ => return None,
        };

        let key = (type_name.to_string(), method_name.to_string());
        self.methods.get(&key).cloned()
    }

    /// Populate built-in methods for stdlib types
    fn populate_builtin_methods(&mut self) {
        // JSON extraction methods
        self.register("json", "as_string", vec![], Type::String);
        self.register("json", "as_number", vec![], Type::Number);
        self.register("json", "as_bool", vec![], Type::Bool);
        self.register("json", "is_null", vec![], Type::Bool);

        // number instance methods (H-260 — D-021 TypeScript parity)
        self.register("number", "toString", vec![], Type::String);
        self.register("number", "toFixed", vec![Type::Number], Type::String);
        self.register("number", "toInt", vec![], Type::Number);

        // bool instance methods (H-260 — D-021 TypeScript parity)
        self.register("bool", "toString", vec![], Type::String);

        // String methods
        // Core methods
        self.register("string", "len", vec![], Type::Number);
        self.register("string", "length", vec![], Type::Number);
        self.register(
            "string",
            "charAt",
            vec![Type::Number],
            Type::Generic {
                name: "Option".to_string(),
                type_args: vec![Type::String],
            },
        );
        self.register(
            "string",
            "substring",
            vec![Type::Number, Type::Number],
            Type::String,
        );
        self.register(
            "string",
            "slice",
            vec![Type::Number, Type::Number],
            Type::String,
        );
        // Search methods
        self.register(
            "string",
            "indexOf",
            vec![Type::String],
            Type::Generic {
                name: "Option".to_string(),
                type_args: vec![Type::Number],
            },
        );
        self.register(
            "string",
            "lastIndexOf",
            vec![Type::String],
            Type::Generic {
                name: "Option".to_string(),
                type_args: vec![Type::Number],
            },
        );
        self.register("string", "includes", vec![Type::String], Type::Bool);
        self.register("string", "startsWith", vec![Type::String], Type::Bool);
        self.register("string", "endsWith", vec![Type::String], Type::Bool);
        // Transform methods
        self.register("string", "toUpperCase", vec![], Type::String);
        self.register("string", "toLowerCase", vec![], Type::String);
        self.register("string", "trim", vec![], Type::String);
        self.register("string", "trimStart", vec![], Type::String);
        self.register("string", "trimEnd", vec![], Type::String);
        self.register("string", "repeat", vec![Type::Number], Type::String);
        self.register(
            "string",
            "replace",
            vec![Type::String, Type::String],
            Type::String,
        );
        self.register(
            "string",
            "replaceAll",
            vec![Type::String, Type::String],
            Type::String,
        );
        self.register(
            "string",
            "split",
            vec![Type::String],
            Type::Array(Box::new(Type::String)),
        );
        // Padding methods
        self.register(
            "string",
            "padStart",
            vec![Type::Number, Type::String],
            Type::String,
        );
        self.register(
            "string",
            "padEnd",
            vec![Type::Number, Type::String],
            Type::String,
        );

        // H-231: DateTime instance methods
        let datetime_ty = Type::Generic {
            name: "DateTime".to_string(),
            type_args: vec![],
        };
        self.register("DateTime", "year", vec![], Type::Number);
        self.register("DateTime", "month", vec![], Type::Number);
        self.register("DateTime", "day", vec![], Type::Number);
        self.register("DateTime", "hour", vec![], Type::Number);
        self.register("DateTime", "minute", vec![], Type::Number);
        self.register("DateTime", "second", vec![], Type::Number);
        self.register("DateTime", "weekday", vec![], Type::Number);
        self.register("DateTime", "dayOfYear", vec![], Type::Number);
        self.register("DateTime", "timestamp", vec![], Type::Number);
        self.register("DateTime", "toIso", vec![], Type::String);
        self.register("DateTime", "toRfc3339", vec![], Type::String);
        self.register("DateTime", "toRfc2822", vec![], Type::String);
        self.register("DateTime", "format", vec![Type::String], Type::String);
        self.register(
            "DateTime",
            "addSeconds",
            vec![Type::Number],
            datetime_ty.clone(),
        );
        self.register(
            "DateTime",
            "addMinutes",
            vec![Type::Number],
            datetime_ty.clone(),
        );
        self.register(
            "DateTime",
            "addHours",
            vec![Type::Number],
            datetime_ty.clone(),
        );
        self.register(
            "DateTime",
            "addDays",
            vec![Type::Number],
            datetime_ty.clone(),
        );
        self.register("DateTime", "diff", vec![datetime_ty.clone()], Type::Number);
        self.register("DateTime", "compare", vec![datetime_ty], Type::Number);

        // H-231: HttpResponse instance methods
        let headers_ty = Type::Generic {
            name: "HashMap".to_string(),
            type_args: vec![Type::String, Type::String],
        };
        self.register("HttpResponse", "status", vec![], Type::Number);
        self.register("HttpResponse", "body", vec![], Type::String);
        self.register("HttpResponse", "headers", vec![], headers_ty);
        self.register(
            "HttpResponse",
            "header",
            vec![Type::String],
            Type::Generic {
                name: "Option".to_string(),
                type_args: vec![Type::String],
            },
        );
        self.register("HttpResponse", "url", vec![], Type::String);
        self.register("HttpResponse", "isSuccess", vec![], Type::Bool);

        // B18: ProcessOutput instance methods
        self.register("ProcessOutput", "stdout", vec![], Type::String);
        self.register("ProcessOutput", "stderr", vec![], Type::String);
        self.register("ProcessOutput", "exitCode", vec![], Type::Number);
        self.register("ProcessOutput", "success", vec![], Type::Bool);

        // B33: Future instance methods
        let future_type = Type::Generic {
            name: "Future".to_string(),
            type_args: vec![],
        };
        self.register("Future", "isResolved", vec![], Type::Bool);
        self.register("Future", "isPending", vec![], Type::Bool);
        self.register("Future", "isRejected", vec![], Type::Bool);
        self.register(
            "Future",
            "then",
            vec![Type::any_placeholder()],
            future_type.clone(),
        );
        self.register(
            "Future",
            "catch",
            vec![Type::any_placeholder()],
            future_type.clone(),
        );
        self.register(
            "Future",
            "finally",
            vec![Type::any_placeholder()],
            future_type.clone(),
        );
        self.register("Future", "await", vec![], Type::Unknown);

        // H-231: Regex instance methods
        self.register("Regex", "test", vec![Type::String], Type::Bool);
        self.register("Regex", "isMatch", vec![Type::String], Type::Bool);
        self.register(
            "Regex",
            "find",
            vec![Type::String],
            Type::Generic {
                name: "Option".to_string(),
                type_args: vec![Type::String],
            },
        );
        self.register(
            "Regex",
            "findAll",
            vec![Type::String],
            Type::Array(Box::new(Type::String)),
        );
        self.register(
            "Regex",
            "replace",
            vec![Type::String, Type::String],
            Type::String,
        );
        self.register(
            "Regex",
            "replaceAll",
            vec![Type::String, Type::String],
            Type::String,
        );
        self.register(
            "Regex",
            "split",
            vec![Type::String],
            Type::Array(Box::new(Type::String)),
        );
    }

    fn array_method_signature(&self, method_name: &str, elem: &Type) -> Option<MethodSignature> {
        let elem_norm = elem.normalized();
        let array_of_elem = Type::Array(Box::new(elem_norm.clone()));

        let (arg_types, return_type) = match method_name {
            // Mutating collection methods — return updated array
            "push" | "unshift" => (vec![elem_norm.clone()], array_of_elem.clone()),
            "reverse" | "sort" => (vec![], array_of_elem.clone()),
            "sortBy" => (vec![Type::Unknown], array_of_elem.clone()),
            // Mutating pair methods — return extracted element
            "pop" | "shift" => (vec![], elem_norm.clone()),
            // Non-mutating methods — return new value
            "len" | "length" => (vec![], Type::Number),
            "isEmpty" => (vec![], Type::Bool),
            "includes" => (vec![elem_norm.clone()], Type::Bool),
            "indexOf" | "lastIndexOf" => (vec![elem_norm.clone()], Type::Number),
            "find" => (
                vec![Type::Unknown],
                Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![elem_norm.clone()],
                },
            ),
            "findIndex" => (vec![Type::Unknown], Type::Number),
            "some" | "every" => (vec![Type::Unknown], Type::Bool),
            "forEach" => (vec![Type::Unknown], Type::Null),
            "map" => (vec![Type::Unknown], Type::Array(Box::new(Type::Unknown))),
            "filter" => (vec![Type::Unknown], array_of_elem.clone()),
            "reduce" => (vec![Type::Unknown, Type::Unknown], Type::Unknown),
            "slice" => (vec![Type::Number, Type::Number], array_of_elem.clone()),
            "concat" => (vec![array_of_elem.clone()], array_of_elem.clone()),
            "flat" | "flatten" => match elem_norm {
                Type::Array(inner) => (vec![], Type::Array(inner)),
                other => (vec![], Type::Array(Box::new(other))),
            },
            "flatMap" => (vec![Type::Unknown], Type::Array(Box::new(Type::Unknown))),
            "join" => (vec![Type::String], Type::String),
            "enumerate" => (
                vec![],
                Type::Array(Box::new(Type::Tuple(vec![Type::Number, elem_norm.clone()]))),
            ),
            _ => return None,
        };

        Some(MethodSignature {
            arg_types,
            return_type,
        })
    }

    /// Return the type signature of a HashMap method call.
    /// K and V are the key/value type parameters.
    fn hashmap_method_signature(
        &self,
        method_name: &str,
        key_type: &Type,
        val_type: &Type,
    ) -> Option<MethodSignature> {
        let k = key_type.clone();
        let v = val_type.clone();
        let hashmap_type = Type::Generic {
            name: "HashMap".to_string(),
            type_args: vec![k.clone(), v.clone()],
        };
        let option_v = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![v.clone()],
        };
        let key_array = Type::Array(Box::new(k.clone()));
        let val_array = Type::Array(Box::new(v.clone()));

        let (arg_types, return_type) = match method_name {
            // Read methods
            "get" => (vec![k.clone()], option_v.clone()),
            "has" | "containsKey" => (vec![k.clone()], Type::Bool),
            "size" | "len" => (vec![], Type::Number),
            "isEmpty" => (vec![], Type::Bool),
            "keys" => (vec![], key_array),
            "values" => (vec![], val_array),
            "entries" => (
                vec![],
                Type::Array(Box::new(Type::Array(Box::new(Type::Unknown)))),
            ),
            "forEach" => (vec![Type::Unknown], Type::Null),
            "map" => (
                vec![Type::Unknown],
                Type::Generic {
                    name: "HashMap".to_string(),
                    type_args: vec![k.clone(), Type::Unknown],
                },
            ),
            "filter" => (vec![Type::Unknown], hashmap_type.clone()),
            // Mutating methods — CoW, return new HashMap
            "set" | "put" => (vec![k.clone(), v.clone()], hashmap_type.clone()),
            "remove" | "delete" => (vec![k.clone()], option_v),
            "clear" => (vec![], hashmap_type),
            _ => return None,
        };

        Some(MethodSignature {
            arg_types,
            return_type,
        })
    }

    fn hashset_method_signature(&self, method_name: &str, elem: &Type) -> Option<MethodSignature> {
        let e = elem.clone();
        let set_type = Type::Generic {
            name: "HashSet".to_string(),
            type_args: vec![e.clone()],
        };
        let (arg_types, return_type) = match method_name {
            "add" => (vec![e.clone()], set_type.clone()),
            "remove" | "delete" => (vec![e.clone()], set_type.clone()),
            "has" | "contains" => (vec![e.clone()], Type::Bool),
            "size" | "len" => (vec![], Type::Number),
            "isEmpty" => (vec![], Type::Bool),
            "toArray" => (vec![], Type::Array(Box::new(e.clone()))),
            "forEach" => (vec![Type::Unknown], Type::Null),
            "clear" => (vec![], set_type),
            _ => return None,
        };
        Some(MethodSignature {
            arg_types,
            return_type,
        })
    }

    fn queue_method_signature(&self, method_name: &str, elem: &Type) -> Option<MethodSignature> {
        let e = elem.clone();
        let queue_type = Type::Generic {
            name: "Queue".to_string(),
            type_args: vec![e.clone()],
        };
        let (arg_types, return_type) = match method_name {
            "enqueue" | "push" => (vec![e.clone()], queue_type.clone()),
            "dequeue" | "pop" => (vec![], e.clone()),
            "peek" => (
                vec![],
                Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![e.clone()],
                },
            ),
            "size" | "len" => (vec![], Type::Number),
            "isEmpty" => (vec![], Type::Bool),
            "toArray" => (vec![], Type::Array(Box::new(e.clone()))),
            "clear" => (vec![], queue_type),
            _ => return None,
        };
        Some(MethodSignature {
            arg_types,
            return_type,
        })
    }

    fn stack_method_signature(&self, method_name: &str, elem: &Type) -> Option<MethodSignature> {
        let e = elem.clone();
        let stack_type = Type::Generic {
            name: "Stack".to_string(),
            type_args: vec![e.clone()],
        };
        let (arg_types, return_type) = match method_name {
            "push" => (vec![e.clone()], stack_type.clone()),
            "pop" => (vec![], e.clone()),
            "peek" => (
                vec![],
                Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![e.clone()],
                },
            ),
            "size" | "len" => (vec![], Type::Number),
            "isEmpty" => (vec![], Type::Bool),
            "toArray" => (vec![], Type::Array(Box::new(e.clone()))),
            "clear" => (vec![], stack_type),
            _ => return None,
        };
        Some(MethodSignature {
            arg_types,
            return_type,
        })
    }

    /// Return the type signature of an Option<T> method call.
    fn option_method_signature(&self, method_name: &str, inner: &Type) -> Option<MethodSignature> {
        let t = inner.clone();
        let option_t = Type::Generic {
            name: "Option".to_string(),
            type_args: vec![t.clone()],
        };
        let (arg_types, return_type) = match method_name {
            "unwrap" => (vec![], t.clone()),
            "unwrapOr" => (vec![t.clone()], t.clone()),
            "isSome" | "isNone" => (vec![], Type::Bool),
            "map" => (
                vec![Type::Unknown], // closure: T -> U (we can't express U here)
                Type::Generic {
                    name: "Option".to_string(),
                    type_args: vec![Type::Unknown],
                },
            ),
            _ => return None,
        };
        let _ = option_t; // suppress unused warning
        Some(MethodSignature {
            arg_types,
            return_type,
        })
    }

    /// Return the type signature of a Result<T,E> method call.
    fn result_method_signature(
        &self,
        method_name: &str,
        ok_type: &Type,
        err_type: &Type,
    ) -> Option<MethodSignature> {
        let t = ok_type.clone();
        let e = err_type.clone();
        let (arg_types, return_type) = match method_name {
            "unwrap" => (vec![], t.clone()),
            "unwrapOr" => (vec![t.clone()], t.clone()),
            "isOk" | "isErr" => (vec![], Type::Bool),
            "map" => (
                vec![Type::Unknown], // closure: T -> U
                Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![Type::Unknown, e.clone()],
                },
            ),
            "mapErr" => (
                vec![Type::Unknown], // closure: E -> F
                Type::Generic {
                    name: "Result".to_string(),
                    type_args: vec![t.clone(), Type::Unknown],
                },
            ),
            _ => return None,
        };
        Some(MethodSignature {
            arg_types,
            return_type,
        })
    }
}

impl Default for MethodTable {
    fn default() -> Self {
        Self::new()
    }
}
