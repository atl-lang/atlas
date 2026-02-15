//! Standard Library Performance Benchmarks
//!
//! Establishes performance baselines for stdlib operations to detect regressions.
//! Benchmarks cover:
//! - String operations (split, join, replace)
//! - Array operations (map, filter, reduce, sort, indexOf)
//! - Math operations (arithmetic, functions)
//! - JSON operations (parse, stringify, nested access)
//! - File I/O operations (read, write)
//!
//! Run with: cargo bench --bench stdlib_benchmarks

use atlas_runtime::{Atlas, SecurityContext};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use tempfile::TempDir;

// ============================================================================
// String Benchmarks
// ============================================================================

fn bench_string_split(c: &mut Criterion) {
    c.bench_function("string_split_1000_elements", |b| {
        let runtime = Atlas::new();
        // Create a string with 1000 comma-separated elements
        let code = r#"
            let data = "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y,z,a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t";
            split(data, ",")
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_string_join(c: &mut Criterion) {
    c.bench_function("string_join_1000_elements", |b| {
        let runtime = Atlas::new();
        // Create array with 1000 elements and join them
        let code = r#"
            let arr = ["a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z","a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t"];
            join(arr, ",")
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_string_replace(c: &mut Criterion) {
    c.bench_function("string_replace_large", |b| {
        let runtime = Atlas::new();
        // Replace in a large string (multiple occurrences)
        let code = r#"
            let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Lorem ipsum dolor sit amet, consectetur adipiscing elit.";
            replace(text, "Lorem", "Atlas")
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

// ============================================================================
// Array Benchmarks
// ============================================================================

fn bench_array_map(c: &mut Criterion) {
    c.bench_function("array_map_10k_elements", |b| {
        let runtime = Atlas::new();
        // Build array programmatically and map over it
        let code = r#"
            fn double(x: number) -> number {
                return x * 2;
            }

            fn buildArray(n: number) -> number[] {
                let arr: number[] = [];
                let i = 0;
                while (i < n) {
                    arr = push(arr, i);
                    i = i + 1;
                }
                return arr;
            }

            let arr = buildArray(10000);
            map(arr, double)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_array_filter(c: &mut Criterion) {
    c.bench_function("array_filter_10k_elements", |b| {
        let runtime = Atlas::new();
        // Filter 10K elements (keep evens)
        let code = r#"
            fn isEven(x: number) -> bool {
                return x % 2 == 0;
            }

            fn buildArray(n: number) -> number[] {
                let arr: number[] = [];
                let i = 0;
                while (i < n) {
                    arr = push(arr, i);
                    i = i + 1;
                }
                return arr;
            }

            let arr = buildArray(10000);
            filter(arr, isEven)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_array_reduce(c: &mut Criterion) {
    c.bench_function("array_reduce_10k_elements", |b| {
        let runtime = Atlas::new();
        // Sum 10K numbers
        let code = r#"
            fn add(sum: number, x: number) -> number {
                return sum + x;
            }

            fn buildArray(n: number) -> number[] {
                let arr: number[] = [];
                let i = 0;
                while (i < n) {
                    arr = push(arr, i);
                    i = i + 1;
                }
                return arr;
            }

            let arr = buildArray(10000);
            reduce(arr, add, 0)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_array_sort(c: &mut Criterion) {
    c.bench_function("array_sort_1000_numbers", |b| {
        let runtime = Atlas::new();
        // Sort 1000 random numbers
        let code = r#"
            fn buildArray(n: number) -> number[] {
                let arr: number[] = [];
                let i = 0;
                while (i < n) {
                    arr = push(arr, random() * 1000);
                    i = i + 1;
                }
                return arr;
            }

            let arr = buildArray(1000);
            sort(arr)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_array_indexof(c: &mut Criterion) {
    c.bench_function("array_indexOf_search", |b| {
        let runtime = Atlas::new();
        // Search for element in large array (worst case: not found)
        let code = r#"
            fn buildArray(n: number) -> number[] {
                let arr: number[] = [];
                let i = 0;
                while (i < n) {
                    arr = push(arr, i);
                    i = i + 1;
                }
                return arr;
            }

            let arr = buildArray(1000);
            indexOf(arr, 999)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

// ============================================================================
// Math Benchmarks
// ============================================================================

fn bench_math_arithmetic(c: &mut Criterion) {
    c.bench_function("math_arithmetic_10k_ops", |b| {
        let runtime = Atlas::new();
        // 10K arithmetic operations
        let code = r#"
            fn compute(n: number) -> number {
                let sum = 0;
                let i = 0;
                while (i < n) {
                    sum = sum + (i * 2) - (i / 2) + (i % 10);
                    i = i + 1;
                }
                return sum;
            }

            compute(10000)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_math_functions(c: &mut Criterion) {
    c.bench_function("math_functions_10k_calls", |b| {
        let runtime = Atlas::new();
        // 10K math function calls
        let code = r#"
            fn compute(n: number) -> number {
                let sum = 0;
                let i = 0;
                while (i < n) {
                    sum = sum + sqrt(abs(i)) + pow(2, 3) + floor(i / 2);
                    i = i + 1;
                }
                return sum;
            }

            compute(10000)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

// ============================================================================
// JSON Benchmarks
// ============================================================================

fn bench_json_parse(c: &mut Criterion) {
    c.bench_function("json_parse_10kb", |b| {
        let runtime = Atlas::new();
        // Parse ~10KB JSON document with nested objects
        let code = r#"
            let json = "{\"users\":[{\"id\":1,\"name\":\"Alice\",\"email\":\"alice@example.com\",\"age\":30,\"active\":true},{\"id\":2,\"name\":\"Bob\",\"email\":\"bob@example.com\",\"age\":25,\"active\":false},{\"id\":3,\"name\":\"Charlie\",\"email\":\"charlie@example.com\",\"age\":35,\"active\":true},{\"id\":4,\"name\":\"Diana\",\"email\":\"diana@example.com\",\"age\":28,\"active\":true},{\"id\":5,\"name\":\"Eve\",\"email\":\"eve@example.com\",\"age\":32,\"active\":false}],\"metadata\":{\"total\":5,\"page\":1,\"perPage\":10,\"hasMore\":false}}";
            parseJSON(json)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_json_stringify(c: &mut Criterion) {
    c.bench_function("json_stringify_large_object", |b| {
        let runtime = Atlas::new();
        // Stringify large object
        let code = r#"
            let data = parseJSON("{\"users\":[{\"id\":1,\"name\":\"Alice\",\"email\":\"alice@example.com\",\"age\":30,\"active\":true},{\"id\":2,\"name\":\"Bob\",\"email\":\"bob@example.com\",\"age\":25,\"active\":false},{\"id\":3,\"name\":\"Charlie\",\"email\":\"charlie@example.com\",\"age\":35,\"active\":true},{\"id\":4,\"name\":\"Diana\",\"email\":\"diana@example.com\",\"age\":28,\"active\":true},{\"id\":5,\"name\":\"Eve\",\"email\":\"eve@example.com\",\"age\":32,\"active\":false}],\"metadata\":{\"total\":5,\"page\":1,\"perPage\":10,\"hasMore\":false}}");
            stringifyJSON(data)
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

fn bench_json_nested_access(c: &mut Criterion) {
    c.bench_function("json_nested_access_deep", |b| {
        let runtime = Atlas::new();
        // Access deeply nested JSON data
        let code = r#"
            let json = parseJSON("{\"level1\":{\"level2\":{\"level3\":{\"level4\":{\"level5\":{\"value\":42}}}}}}");
            let val = json["level1"]["level2"]["level3"]["level4"]["level5"]["value"];
            val.as_number()
        "#;
        b.iter(|| {
            let _ = runtime.eval(black_box(code));
        });
    });
}

// ============================================================================
// File I/O Benchmarks
// ============================================================================

fn bench_file_read(c: &mut Criterion) {
    // Create a 1MB test file
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_1mb.txt");
    let content = "x".repeat(1024 * 1024); // 1MB of 'x'
    fs::write(&file_path, content).unwrap();

    c.bench_function("file_read_1mb", |b| {
        let mut security = SecurityContext::new();
        security.grant_filesystem_read(temp_dir.path(), true);
        let runtime = Atlas::new_with_security(security);

        let code = format!(r#"readFile("{}")"#, file_path.display());
        b.iter(|| {
            let _ = runtime.eval(black_box(&code));
        });
    });
}

fn bench_file_write(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test_write.txt");

    c.bench_function("file_write_1mb", |b| {
        let mut security = SecurityContext::new();
        security.grant_filesystem_write(temp_dir.path(), true);
        let runtime = Atlas::new_with_security(security);

        // Write 1MB of data
        let data = "x".repeat(1024 * 1024);
        let code = format!(r#"writeFile("{}", "{}")"#, file_path.display(), data);
        b.iter(|| {
            let _ = runtime.eval(black_box(&code));
        });
    });
}

// ============================================================================
// Criterion Groups
// ============================================================================

criterion_group!(
    string_benches,
    bench_string_split,
    bench_string_join,
    bench_string_replace
);

criterion_group!(
    array_benches,
    bench_array_map,
    bench_array_filter,
    bench_array_reduce,
    bench_array_sort,
    bench_array_indexof
);

criterion_group!(math_benches, bench_math_arithmetic, bench_math_functions);

criterion_group!(
    json_benches,
    bench_json_parse,
    bench_json_stringify,
    bench_json_nested_access
);

criterion_group!(file_benches, bench_file_read, bench_file_write);

criterion_main!(
    string_benches,
    array_benches,
    math_benches,
    json_benches,
    file_benches
);
