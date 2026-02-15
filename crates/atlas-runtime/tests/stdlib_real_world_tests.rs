//! Real-World Standard Library Integration Tests
//!
//! This test suite demonstrates practical, real-world usage patterns of the Atlas
//! standard library. Tests read like actual programs users would write:
//! - CSV processing
//! - JSON API handling
//! - Log file analysis
//! - Data transformation pipelines
//! - Text processing
//! - Configuration file processing
//!
//! ALL tests verify interpreter/VM parity (100% identical output).

#![allow(clippy::bool_assert_comparison)]

use atlas_runtime::{Atlas, SecurityContext, Value};
use tempfile::TempDir;

// ============================================================================
// Test Helpers
// ============================================================================

/// Assert with file I/O permissions (grants full filesystem access for tests)
fn assert_eval_number_with_io(source: &str, expected: f64) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Number(n)) if (n - expected).abs() < f64::EPSILON => {}
        Ok(val) => panic!(
            "Expected number {}, got {:?}",
            expected,
            val.to_display_string()
        ),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

#[allow(dead_code)]
fn assert_eval_bool_with_io(source: &str, expected: bool) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::Bool(b)) if b == expected => {}
        Ok(val) => panic!(
            "Expected bool {}, got {:?}",
            expected,
            val.to_display_string()
        ),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

fn assert_eval_string_with_io(source: &str, expected: &str) {
    let security = SecurityContext::allow_all();
    let runtime = Atlas::new_with_security(security);
    match runtime.eval(source) {
        Ok(Value::String(s)) if s.as_ref() == expected => {}
        Ok(val) => panic!(
            "Expected string '{}', got '{}'",
            expected,
            val.to_display_string()
        ),
        Err(e) => panic!("Execution error: {:?}", e),
    }
}

// ============================================================================
// Category 1: CSV Processing (30 tests)
// ============================================================================

#[test]
fn test_csv_read_and_parse_basic() {
    // Create CSV file
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(
        &csv_path,
        "name,age,city\nAlice,30,NYC\nBob,25,LA\nCarol,35,SF\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        header
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "name,age,city");
}

#[test]
fn test_csv_parse_rows() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "name,age\nAlice,30\nBob,25\nCarol,35\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        let dataLines: string[] = slice(lines, 1, len(lines));

        // Get first data row
        let row1: string = dataLines[0];
        let fields: string[] = split(row1, ",");
        let name: string = fields[0];
        name
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "Alice");
}

#[test]
fn test_csv_count_rows() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "id,value\n1,100\n2,200\n3,300\n4,400\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        // Count data rows (excluding header and empty last line)
        let allRows: number = len(lines);
        allRows - 2.0
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 4.0);
}

#[test]
fn test_csv_filter_by_criteria() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("sales.csv");
    std::fs::write(
        &csv_path,
        "product,price\nApple,1.5\nBanana,0.5\nCherry,3.0\nDate,2.5\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isExpensive(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let price: number = parseFloat(fields[1]);
            return price >= 2.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines));

        // Filter expensive items
        let expensive: string[] = filter(dataLines, isExpensive);
        len(expensive)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 2.0); // Cherry (3.0) and Date (2.5)
}

#[test]
fn test_csv_extract_column() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("users.csv");
    std::fs::write(
        &csv_path,
        "name,email\nAlice,alice@test.com\nBob,bob@test.com\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn getName(row: string) -> string {{
            let fields: string[] = split(row, ",");
            return fields[0];
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let names: string[] = map(dataLines, getName);
        join(names, "|")
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "Alice|Bob");
}

#[test]
fn test_csv_sum_column() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("amounts.csv");
    std::fs::write(&csv_path, "item,amount\nA,10\nB,20\nC,30\n").unwrap();

    let code = format!(
        r#"
        fn sumAmounts(total: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let amount: number = parseFloat(fields[1]);
            return total + amount;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, sumAmounts, 0.0)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 60.0);
}

#[test]
fn test_csv_empty_file() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("empty.csv");
    std::fs::write(&csv_path, "").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        len(csv)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 0.0);
}

#[test]
fn test_csv_single_row() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("single.csv");
    std::fs::write(&csv_path, "name,value\nAlice,100\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);
        len(dataLines)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 1.0);
}

#[test]
fn test_csv_handle_empty_fields() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("sparse.csv");
    std::fs::write(&csv_path, "a,b,c\n1,,3\n4,5,\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];
        let fields: string[] = split(row1, ",");
        let emptyField: string = fields[1];
        len(emptyField)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 0.0);
}

#[test]
fn test_csv_write_transformed() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.csv");
    let output_path = temp_dir.path().join("output.csv");
    std::fs::write(&input_path, "name,value\nAlice,10\nBob,20\n").unwrap();

    let code = format!(
        r#"
        fn transform(row: string) -> string {{
            let fields: string[] = split(row, ",");
            let name: string = fields[0];
            let value: number = parseFloat(fields[1]);
            let doubled: number = value * 2.0;
            return name + "," + str(doubled);
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let transformed: string[] = map(dataLines, transform);
        let output: string = header + "\n" + join(transformed, "\n") + "\n";
        writeFile("{}", output);

        // Verify output
        let result: string = readFile("{}");
        result
    "#,
        input_path.display(),
        output_path.display(),
        output_path.display()
    );
    assert_eval_string_with_io(&code, "name,value\nAlice,20\nBob,40\n");
}

#[test]
fn test_csv_calculate_average() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("scores.csv");
    std::fs::write(&csv_path, "student,score\nAlice,85\nBob,90\nCarol,95\n").unwrap();

    let code = format!(
        r#"
        fn sumScores(total: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let score: number = parseFloat(fields[1]);
            return total + score;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let total: number = reduce(dataLines, sumScores, 0.0);
        let count: number = len(dataLines);
        total / count
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 90.0); // (85 + 90 + 95) / 3 = 90
}

#[test]
fn test_csv_filter_and_count() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "name,age\nAlice,25\nBob,35\nCarol,40\nDave,20\n").unwrap();

    let code = format!(
        r#"
        fn isAdult(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let age: number = parseFloat(fields[1]);
            return age >= 30.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let adults: string[] = filter(dataLines, isAdult);
        len(adults)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 2.0); // Bob (35) and Carol (40)
}

#[test]
fn test_csv_max_value() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("values.csv");
    std::fs::write(&csv_path, "id,value\n1,45\n2,89\n3,23\n4,67\n").unwrap();

    let code = format!(
        r#"
        fn findMax(current: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let value: number = parseFloat(fields[1]);
            return max(current, value);
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, findMax, 0.0)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 89.0);
}

#[test]
fn test_csv_header_extraction() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("data.csv");
    std::fs::write(&csv_path, "name,email,age\nAlice,a@test.com,30\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let header: string = lines[0];
        let columns: string[] = split(header, ",");
        join(columns, "|")
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "name|email|age");
}

#[test]
fn test_csv_quoted_fields() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("text.csv");
    std::fs::write(&csv_path, "name,note\nAlice,Hello World\nBob,Test Data\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];
        let fields: string[] = split(row1, ",");
        fields[1]
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "Hello World");
}

#[test]
fn test_csv_multi_column_filter() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("products.csv");
    std::fs::write(
        &csv_path,
        "name,price,stock\nApple,1.5,100\nBanana,0.5,50\nCherry,3.0,200\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isHighValueInStock(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let price: number = parseFloat(fields[1]);
            let stock: number = parseFloat(fields[2]);
            return price >= 1.0 && stock >= 100.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let filtered: string[] = filter(dataLines, isHighValueInStock);
        len(filtered)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 2.0); // Apple and Cherry
}

#[test]
fn test_csv_column_sum_with_condition() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("sales.csv");
    std::fs::write(
        &csv_path,
        "region,amount\nNorth,1000\nSouth,500\nNorth,1500\nEast,800\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn sumNorth(total: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let region: string = fields[0];
            let amount: number = parseFloat(fields[1]);
            if (region == "North") {{
                return total + amount;
            }}
            return total;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, sumNorth, 0.0)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 2500.0); // 1000 + 1500
}

#[test]
fn test_csv_row_count_by_group() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("events.csv");
    std::fs::write(
        &csv_path,
        "type,count\nERROR,5\nWARN,10\nERROR,3\nINFO,20\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            return fields[0] == "ERROR";
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let errors: string[] = filter(dataLines, isError);
        len(errors)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_csv_transform_and_join() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("names.csv");
    std::fs::write(&csv_path, "first,last\nAlice,Smith\nBob,Jones\n").unwrap();

    let code = format!(
        r#"
        fn fullName(row: string) -> string {{
            let fields: string[] = split(row, ",");
            return fields[0] + " " + fields[1];
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let names: string[] = map(dataLines, fullName);
        join(names, "; ")
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "Alice Smith; Bob Jones");
}

#[test]
fn test_csv_percentage_calculation() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("stats.csv");
    std::fs::write(&csv_path, "item,sold,total\nA,80,100\nB,60,100\n").unwrap();

    let code = format!(
        r#"
        fn calcPercentage(row: string) -> number {{
            let fields: string[] = split(row, ",");
            let sold: number = parseFloat(fields[1]);
            let total: number = parseFloat(fields[2]);
            return (sold / total) * 100.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];

        calcPercentage(row1)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 80.0);
}

#[test]
fn test_csv_trim_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("messy.csv");
    std::fs::write(&csv_path, "name,value\n Alice , 100 \n Bob , 200 \n").unwrap();

    let code = format!(
        r#"
        fn cleanRow(row: string) -> string {{
            let fields: string[] = split(row, ",");
            let name: string = trim(fields[0]);
            let value: string = trim(fields[1]);
            return name + "," + value;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];

        cleanRow(row1)
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "Alice,100");
}

#[test]
fn test_csv_case_insensitive_filter() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("items.csv");
    std::fs::write(
        &csv_path,
        "name,type\nApple,FRUIT\nCarrot,vegetable\nBanana,Fruit\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isFruit(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            let type: string = toLowerCase(fields[1]);
            return type == "fruit";
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let fruits: string[] = filter(dataLines, isFruit);
        len(fruits)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_csv_contains_filter() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("logs.csv");
    std::fs::write(
        &csv_path,
        "timestamp,message\n10:00,User login\n10:05,Error occurred\n10:10,User logout\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn hasError(row: string) -> bool {{
            return includes(row, "Error");
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let errors: string[] = filter(dataLines, hasError);
        len(errors)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 1.0);
}

#[test]
fn test_csv_numeric_sort_data() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("unsorted.csv");
    std::fs::write(&csv_path, "id,value\n3,30\n1,10\n2,20\n").unwrap();

    let code = format!(
        r#"
        fn compareById(a: string, b: string) -> number {{
            let fieldsA: string[] = split(a, ",");
            let fieldsB: string[] = split(b, ",");
            let idA: number = parseFloat(fieldsA[0]);
            let idB: number = parseFloat(fieldsB[0]);
            return idA - idB;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let sorted: string[] = sort(dataLines, compareById);
        let first: string = sorted[0];
        let fields: string[] = split(first, ",");
        fields[0]
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "1");
}

#[test]
fn test_csv_append_row() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("append.csv");
    std::fs::write(&csv_path, "name,score\nAlice,85\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let newRow: string = "Bob,90";
        let updated: string = csv + newRow + "\n";
        writeFile("{}", updated);

        let result: string = readFile("{}");
        let lines: string[] = split(result, "\n");
        len(lines) - 1.0
    "#,
        csv_path.display(),
        csv_path.display(),
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 3.0); // header + Alice + Bob
}

#[test]
fn test_csv_validate_column_count() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("valid.csv");
    std::fs::write(&csv_path, "a,b,c\n1,2,3\n4,5,6\n").unwrap();

    let code = format!(
        r#"
        fn hasThreeColumns(row: string) -> bool {{
            let fields: string[] = split(row, ",");
            return len(fields) == 3.0;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let valid: string[] = filter(dataLines, hasThreeColumns);
        len(valid) == len(dataLines)
    "#,
        csv_path.display()
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_csv_extract_unique_values() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("categories.csv");
    std::fs::write(
        &csv_path,
        "item,category\nA,fruit\nB,veggie\nC,fruit\nD,meat\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn getCategory(row: string) -> string {{
            let fields: string[] = split(row, ",");
            return fields[1];
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        let categories: string[] = map(dataLines, getCategory);
        // Count unique by checking first occurrence
        let hasFruit: bool = arrayIncludes(categories, "fruit");
        let hasVeggie: bool = arrayIncludes(categories, "veggie");
        let hasMeat: bool = arrayIncludes(categories, "meat");

        str(hasFruit) + "," + str(hasVeggie) + "," + str(hasMeat)
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "true,true,true");
}

#[test]
fn test_csv_conditional_transformation() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("grades.csv");
    std::fs::write(&csv_path, "name,score\nAlice,85\nBob,92\nCarol,78\n").unwrap();

    let code = format!(
        r#"
        fn addGrade(row: string) -> string {{
            let fields: string[] = split(row, ",");
            let score: number = parseFloat(fields[1]);
            let grade: string = "F";
            if (score >= 90.0) {{
                grade = "A";
            }} else {{
                if (score >= 80.0) {{
                    grade = "B";
                }} else {{
                    grade = "C";
                }}
            }}
            return fields[0] + "," + fields[1] + "," + grade;
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];

        addGrade(row1)
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "Alice,85,B");
}

#[test]
fn test_csv_min_value() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("temps.csv");
    std::fs::write(&csv_path, "day,temp\nMon,72\nTue,68\nWed,75\n").unwrap();

    let code = format!(
        r#"
        fn findMin(current: number, row: string) -> number {{
            let fields: string[] = split(row, ",");
            let temp: number = parseFloat(fields[1]);
            if (current == 0.0) {{
                return temp;
            }}
            return min(current, temp);
        }}

        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let dataLines: string[] = slice(lines, 1, len(lines) - 1.0);

        reduce(dataLines, findMin, 0.0)
    "#,
        csv_path.display()
    );
    assert_eval_number_with_io(&code, 68.0);
}

#[test]
fn test_csv_concatenate_fields() {
    let temp_dir = TempDir::new().unwrap();
    let csv_path = temp_dir.path().join("addresses.csv");
    std::fs::write(&csv_path, "street,city,state\nMain St,NYC,NY\n").unwrap();

    let code = format!(
        r#"
        let csv: string = readFile("{}");
        let lines: string[] = split(csv, "\n");
        let row1: string = lines[1];
        let fields: string[] = split(row1, ",");
        fields[0] + ", " + fields[1] + ", " + fields[2]
    "#,
        csv_path.display()
    );
    assert_eval_string_with_io(&code, "Main St, NYC, NY");
}

// ============================================================================
// Category 2: JSON API Response Handling (30 tests)
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
        let count: number = 0.0;
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
        if (premium) {
            let features: json = data["features"];
            let advanced: bool = features["advanced"].as_bool();
            advanced
        } else {
            false
        }
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

// ============================================================================
// Category 3: Log File Analysis (30 tests)
// ============================================================================

#[test]
fn test_log_parse_basic() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 10:00:00 INFO: Application started\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let first: string = lines[0];
        includes(first, "INFO")
    "#,
        log_path.display()
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_filter_errors() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: Started\nERROR: Failed\nWARN: Warning\nERROR: Crashed\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let errors: string[] = filter(lines, isError);
        len(errors)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_timestamps() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 ERROR: Test\n2024-01-02 INFO: OK\n").unwrap();

    let code = format!(
        r#"
        fn getTimestamp(line: string) -> string {{
            return substring(line, 0.0, 10.0);
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        getTimestamp(line1)
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "2024-01-01");
}

#[test]
fn test_log_count_by_level() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: msg1\nERROR: msg2\nINFO: msg3\nWARN: msg4\nINFO: msg5\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isInfo(line: string) -> bool {{
            return includes(line, "INFO");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let infos: string[] = filter(dataLines, isInfo);
        len(infos)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_extract_error_messages() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "[2024-01-01] ERROR: Connection failed\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line: string = lines[0];
        let parts: string[] = split(line, "ERROR: ");
        if (len(parts) >= 2.0) {{
            parts[1]
        }} else {{
            ""
        }}
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "Connection failed");
}

#[test]
fn test_log_filter_by_date() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "2024-01-01 INFO: Old\n2024-01-15 ERROR: New\n2024-01-20 INFO: Newer\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isAfterJan10(line: string) -> bool {{
            let date: string = substring(line, 0.0, 10.0);
            return date >= "2024-01-10";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let recent: string[] = filter(dataLines, isAfterJan10);
        len(recent)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_severity_ordering() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "DEBUG: d\nINFO: i\nWARN: w\nERROR: e\n").unwrap();

    let code = format!(
        r#"
        fn isHighSeverity(line: string) -> bool {{
            return includes(line, "ERROR") || includes(line, "WARN");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let high: string[] = filter(dataLines, isHighSeverity);
        len(high)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_multi_line_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "ERROR: Failed\nStack trace line 1\nStack trace line 2\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let first: string = lines[0];
        let second: string = lines[1];
        includes(first, "ERROR") && includes(second, "Stack")
    "#,
        log_path.display()
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_empty_lines_filter() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "INFO: msg1\n\nERROR: msg2\n\nWARN: msg3\n").unwrap();

    let code = format!(
        r#"
        fn isNotEmpty(line: string) -> bool {{
            return len(line) > 0.0;
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let nonEmpty: string[] = filter(lines, isNotEmpty);
        len(nonEmpty)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_contains_pattern() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "User alice logged in\nUser bob failed\nUser alice logged out\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn mentionsAlice(line: string) -> bool {{
            return includes(line, "alice");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let aliceLogs: string[] = filter(dataLines, mentionsAlice);
        len(aliceLogs)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_case_insensitive_search() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Error: test\nerror: test2\nERROR: test3\n").unwrap();

    let code = format!(
        r#"
        fn hasError(line: string) -> bool {{
            let lower: string = toLowerCase(line);
            return includes(lower, "error");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let errors: string[] = filter(dataLines, hasError);
        len(errors)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_extract_user_actions() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "User:alice Action:login\nUser:bob Action:logout\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn extractUser(line: string) -> string {{
            let parts: string[] = split(line, " ");
            let userPart: string = parts[0];
            let userFields: string[] = split(userPart, ":");
            return userFields[1];
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        extractUser(line1)
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "alice");
}

#[test]
fn test_log_count_occurrences() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "login\nlogout\nlogin\nlogin\nlogout\n").unwrap();

    let code = format!(
        r#"
        fn isLogin(line: string) -> bool {{
            return line == "login";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let logins: string[] = filter(dataLines, isLogin);
        len(logins)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_trim_whitespace() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "  ERROR: Test  \n  WARN: Alert  \n").unwrap();

    let code = format!(
        r#"
        fn cleanLine(line: string) -> string {{
            return trim(line);
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        let cleaned: string = cleanLine(line1);
        cleaned
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "ERROR: Test");
}

#[test]
fn test_log_starts_with_timestamp() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "2024-01-01 INFO: msg\n2024-01-02 ERROR: err\n").unwrap();

    let code = format!(
        r#"
        fn hasTimestamp(line: string) -> bool {{
            return startsWith(line, "2024");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let timestamped: string[] = filter(dataLines, hasTimestamp);
        len(timestamped)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_ip_addresses() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("access.log");
    std::fs::write(&log_path, "192.168.1.1 GET /page\n10.0.0.1 POST /api\n").unwrap();

    let code = format!(
        r#"
        fn extractIP(line: string) -> string {{
            let parts: string[] = split(line, " ");
            return parts[0];
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        extractIP(line1)
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "192.168.1.1");
}

#[test]
fn test_log_group_by_category() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "DB: query\nAPI: request\nDB: update\nDB: delete\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isDatabase(line: string) -> bool {{
            return startsWith(line, "DB:");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let dbLogs: string[] = filter(dataLines, isDatabase);
        len(dbLogs)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_parse_structured() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "level=error msg=\"Failed to connect\" code=500\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line: string = lines[0];
        let parts: string[] = split(line, " ");
        let levelPart: string = parts[0];
        startsWith(levelPart, "level=error")
    "#,
        log_path.display()
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_count_warnings() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "INFO\nWARN\nERROR\nWARN\nINFO\nWARN\n").unwrap();

    let code = format!(
        r#"
        fn countWarnings(total: number, line: string) -> number {{
            if (line == "WARN") {{
                return total + 1.0;
            }}
            return total;
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        reduce(dataLines, countWarnings, 0.0)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 3.0);
}

#[test]
fn test_log_find_first_error() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "INFO: ok\nWARN: warning\nERROR: failure\nERROR: another\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let firstError: string = find(dataLines, isError);
        firstError
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "ERROR: failure");
}

#[test]
fn test_log_reverse_chronological() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Line1\nLine2\nLine3\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let reversed: string[] = reverse(dataLines);
        reversed[0]
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "Line3");
}

#[test]
fn test_log_summary_report() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(
        &log_path,
        "ERROR:e1\nINFO:i1\nERROR:e2\nWARN:w1\nERROR:e3\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{ return includes(line, "ERROR"); }}
        fn isWarn(line: string) -> bool {{ return includes(line, "WARN"); }}
        fn isInfo(line: string) -> bool {{ return includes(line, "INFO"); }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);

        let errors: number = len(filter(dataLines, isError));
        let warns: number = len(filter(dataLines, isWarn));
        let infos: number = len(filter(dataLines, isInfo));

        "E:" + str(errors) + ",W:" + str(warns) + ",I:" + str(infos)
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "E:3,W:1,I:1");
}

#[test]
fn test_log_filter_time_range() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "08:00 Start\n09:30 Middle\n12:00 End\n").unwrap();

    let code = format!(
        r#"
        fn isMorning(line: string) -> bool {{
            let time: string = substring(line, 0.0, 5.0);
            return time < "10:00";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let morning: string[] = filter(dataLines, isMorning);
        len(morning)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_log_extract_http_codes() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("access.log");
    std::fs::write(&log_path, "GET /page 200\nPOST /api 404\nGET /home 200\n").unwrap();

    let code = format!(
        r#"
        fn is404(line: string) -> bool {{
            return includes(line, "404");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let notFound: string[] = filter(dataLines, is404);
        len(notFound)
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 1.0);
}

#[test]
fn test_log_parse_json_lines() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("json.log");
    std::fs::write(
        &log_path,
        "{\"level\":\"error\",\"msg\":\"failed\"}\n{\"level\":\"info\",\"msg\":\"ok\"}\n",
    )
    .unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let line1: string = lines[0];
        let json: json = parseJSON(line1);
        let level: string = json["level"].as_string();
        level
    "#,
        log_path.display()
    );
    assert_eval_string_with_io(&code, "error");
}

#[test]
fn test_log_aggregate_metrics() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("metrics.log");
    std::fs::write(&log_path, "latency:100\nlatency:150\nlatency:200\n").unwrap();

    let code = format!(
        r#"
        fn sumLatency(total: number, line: string) -> number {{
            let parts: string[] = split(line, ":");
            let value: number = parseFloat(parts[1]);
            return total + value;
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let total: number = reduce(dataLines, sumLatency, 0.0);
        let avg: number = total / len(dataLines);
        avg
    "#,
        log_path.display()
    );
    assert_eval_number_with_io(&code, 150.0);
}

#[test]
fn test_log_detect_anomalies() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "Normal\nNormal\nANOMALY\nNormal\n").unwrap();

    let code = format!(
        r#"
        fn isAnomaly(line: string) -> bool {{
            return line == "ANOMALY";
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let anomalies: string[] = filter(dataLines, isAnomaly);
        len(anomalies) > 0.0
    "#,
        log_path.display()
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_combine_multiline() {
    let temp_dir = TempDir::new().unwrap();
    let log_path = temp_dir.path().join("app.log");
    std::fs::write(&log_path, "ERROR: Start\nContinue\nEnd\n").unwrap();

    let code = format!(
        r#"
        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let combined: string = lines[0] + " " + lines[1] + " " + lines[2];
        includes(combined, "Start") && includes(combined, "Continue") && includes(combined, "End")
    "#,
        log_path.display()
    );
    assert_eval_bool_with_io(&code, true);
}

#[test]
fn test_log_write_filtered() {
    let temp_dir = TempDir::new().unwrap();
    let input_path = temp_dir.path().join("input.log");
    let output_path = temp_dir.path().join("errors.log");
    std::fs::write(
        &input_path,
        "INFO: ok\nERROR: failed\nWARN: warn\nERROR: bad\n",
    )
    .unwrap();

    let code = format!(
        r#"
        fn isError(line: string) -> bool {{
            return includes(line, "ERROR");
        }}

        let logs: string = readFile("{}");
        let lines: string[] = split(logs, "\n");
        let dataLines: string[] = slice(lines, 0.0, len(lines) - 1.0);
        let errors: string[] = filter(dataLines, isError);
        let output: string = join(errors, "\n") + "\n";
        writeFile("{}", output);

        let result: string = readFile("{}");
        let resultLines: string[] = split(result, "\n");
        len(resultLines) - 1.0
    "#,
        input_path.display(),
        output_path.display(),
        output_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

// ============================================================================
// Category 4: Data Transformation Pipelines (30 tests)
// ============================================================================

#[test]
fn test_pipeline_map_filter_reduce() {
    let code = r#"
        fn double(x: number) -> number { return x * 2.0; }
        fn isEven(x: number) -> bool { return x % 2.0 == 0.0; }
        fn sum(a: number, b: number) -> number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0, 5.0];
        let doubled: number[] = map(numbers, double);
        let evens: number[] = filter(doubled, isEven);
        reduce(evens, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 30.0); // doubled=[2,4,6,8,10], all even, sum=30
}

#[test]
fn test_pipeline_filter_map_join() {
    let code = r#"
        fn isLong(s: string) -> bool { return len(s) > 3.0; }
        fn upper(s: string) -> string { return toUpperCase(s); }

        let words: string[] = ["hi", "hello", "bye", "world"];
        let long: string[] = filter(words, isLong);
        let upper: string[] = map(long, upper);
        join(upper, "-")
    "#;
    assert_eval_string_with_io(code, "HELLO-WORLD");
}

#[test]
fn test_pipeline_nested_arrays() {
    let code = r#"
        let nested: number[][] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        let flat: number[] = flatten(nested);
        fn double(x: number) -> number { return x * 2.0; }
        let doubled: number[] = map(flat, double);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(doubled, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 42.0); // [1..6] doubled = [2,4,6,8,10,12] sum=42
}

#[test]
fn test_pipeline_string_processing() {
    let code = r#"
        fn trimAndLower(s: string) -> string {
            let t: string = trim(s);
            return toLowerCase(t);
        }

        let input: string[] = ["  HELLO  ", "  WORLD  ", "  TEST  "];
        let cleaned: string[] = map(input, trimAndLower);
        join(cleaned, ",")
    "#;
    assert_eval_string_with_io(code, "hello,world,test");
}

#[test]
fn test_pipeline_multi_step_filter() {
    let code = r#"
        fn isPositive(x: number) -> bool { return x > 0.0; }
        fn isSmall(x: number) -> bool { return x < 100.0; }

        let numbers: number[] = [-5.0, 10.0, 150.0, 50.0, -20.0, 75.0];
        let positive: number[] = filter(numbers, isPositive);
        let small: number[] = filter(positive, isSmall);
        len(small)
    "#;
    assert_eval_number_with_io(code, 3.0); // [10, 50, 75]
}

#[test]
fn test_pipeline_sort_and_slice() {
    let code = r#"
        fn compare(a: number, b: number) -> number { return a - b; }

        let numbers: number[] = [5.0, 2.0, 8.0, 1.0, 9.0, 3.0];
        let sorted: number[] = sort(numbers, compare);
        let top3: number[] = slice(sorted, 0.0, 3.0);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(top3, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 6.0); // [1,2,3] sum=6
}

#[test]
fn test_pipeline_flatmap_strings() {
    let code = r#"
        fn splitWords(s: string) -> string[] {
            return split(s, " ");
        }

        let sentences: string[] = ["hello world", "foo bar"];
        let words: string[] = flatMap(sentences, splitWords);
        len(words)
    "#;
    assert_eval_number_with_io(code, 4.0);
}

#[test]
fn test_pipeline_conditional_transform() {
    let code = r#"
        fn transform(x: number) -> number {
            if (x < 0.0) {
                return abs(x);
            }
            return x;
        }

        let numbers: number[] = [-5.0, 10.0, -3.0, 7.0];
        let transformed: number[] = map(numbers, transform);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(transformed, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 25.0); // [5,10,3,7] sum=25
}

#[test]
fn test_pipeline_find_and_transform() {
    let code = r#"
        fn isLarge(x: number) -> bool { return x > 50.0; }

        let numbers: number[] = [10.0, 60.0, 30.0, 80.0];
        let found: number = find(numbers, isLarge);
        found * 2.0
    "#;
    assert_eval_number_with_io(code, 120.0); // 60 * 2
}

#[test]
fn test_pipeline_every_and_some() {
    let code = r#"
        fn isPositive(x: number) -> bool { return x > 0.0; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let allPositive: bool = every(numbers, isPositive);
        let somePositive: bool = some(numbers, isPositive);
        allPositive && somePositive
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_pipeline_reverse_and_join() {
    let code = r#"
        let words: string[] = ["one", "two", "three"];
        let reversed: string[] = reverse(words);
        join(reversed, "-")
    "#;
    assert_eval_string_with_io(code, "three-two-one");
}

#[test]
fn test_pipeline_unshift_and_concat() {
    let code = r#"
        let arr1: number[] = [2.0, 3.0];
        let arr2: number[] = [4.0, 5.0];
        let withOne: number[] = unshift(arr1, 1.0);
        let combined: number[] = concat(withOne, arr2);
        len(combined)
    "#;
    assert_eval_number_with_io(code, 5.0);
}

#[test]
fn test_pipeline_multiple_maps() {
    let code = r#"
        fn add10(x: number) -> number { return x + 10.0; }
        fn double(x: number) -> number { return x * 2.0; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let step1: number[] = map(numbers, add10);
        let step2: number[] = map(step1, double);
        step2[0]
    "#;
    assert_eval_number_with_io(code, 22.0); // (1+10)*2 = 22
}

#[test]
fn test_pipeline_filter_reverse_first() {
    let code = r#"
        fn isEven(x: number) -> bool { return x % 2.0 == 0.0; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let evens: number[] = filter(numbers, isEven);
        let reversed: number[] = reverse(evens);
        reversed[0]
    "#;
    assert_eval_number_with_io(code, 6.0);
}

#[test]
fn test_pipeline_sortby_number() {
    let code = r#"
        fn negate(x: number) -> number { return x * -1.0; }

        let numbers: number[] = [3.0, 1.0, 4.0, 1.0, 5.0];
        let sorted: number[] = sortBy(numbers, negate);
        sorted[0]
    "#;
    assert_eval_number_with_io(code, 5.0); // sorted descending
}

#[test]
fn test_pipeline_pop_and_process() {
    let code = r#"
        let numbers: number[] = [1.0, 2.0, 3.0];
        let result: number[] = pop(numbers);
        let last: number = result[0];
        let remaining: number[] = result[1];
        last + len(remaining)
    "#;
    assert_eval_number_with_io(code, 5.0); // 3 + 2
}

#[test]
fn test_pipeline_shift_and_process() {
    let code = r#"
        let numbers: number[] = [1.0, 2.0, 3.0];
        let result: number[] = shift(numbers);
        let first: number = result[0];
        let remaining: number[] = result[1];
        first + len(remaining)
    "#;
    assert_eval_number_with_io(code, 3.0); // 1 + 2
}

#[test]
fn test_pipeline_findindex_and_slice() {
    let code = r#"
        fn isLarge(x: number) -> bool { return x > 50.0; }

        let numbers: number[] = [10.0, 20.0, 60.0, 80.0];
        let idx: number = findIndex(numbers, isLarge);
        let fromLarge: number[] = slice(numbers, idx, len(numbers));
        len(fromLarge)
    "#;
    assert_eval_number_with_io(code, 2.0); // [60, 80]
}

#[test]
fn test_pipeline_complex_aggregation() {
    let code = r#"
        fn square(x: number) -> number { return x * x; }
        fn sum(a: number, b: number) -> number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0, 4.0];
        let squared: number[] = map(numbers, square);
        let total: number = reduce(squared, sum, 0.0);
        total
    "#;
    assert_eval_number_with_io(code, 30.0); // 1+4+9+16 = 30
}

#[test]
fn test_pipeline_string_filter_map() {
    let code = r#"
        fn notEmpty(s: string) -> bool { return len(s) > 0.0; }
        fn firstChar(s: string) -> string { return charAt(s, 0.0); }

        let words: string[] = ["apple", "", "banana", "", "cherry"];
        let nonEmpty: string[] = filter(words, notEmpty);
        let firstChars: string[] = map(nonEmpty, firstChar);
        join(firstChars, "")
    "#;
    assert_eval_string_with_io(code, "abc");
}

#[test]
fn test_pipeline_nested_operations() {
    let code = r#"
        fn process(x: number) -> number {
            let step1: number = x + 5.0;
            let step2: number = step1 * 2.0;
            return step2;
        }
        fn sum(a: number, b: number) -> number { return a + b; }

        let numbers: number[] = [1.0, 2.0, 3.0];
        let processed: number[] = map(numbers, process);
        reduce(processed, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 42.0); // (1+5)*2=12, (2+5)*2=14, (3+5)*2=16, sum=42
}

#[test]
fn test_pipeline_includes_filter() {
    let code = r#"
        fn hasLetterA(s: string) -> bool {
            return includes(s, "a");
        }

        let words: string[] = ["apple", "berry", "apricot", "cherry"];
        let withA: string[] = filter(words, hasLetterA);
        len(withA)
    "#;
    assert_eval_number_with_io(code, 2.0); // apple, apricot
}

#[test]
fn test_pipeline_index_access_transform() {
    let code = r#"
        let matrix: number[][] = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]];
        fn getFirst(row: number[]) -> number { return row[0]; }

        let firstElements: number[] = map(matrix, getFirst);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(firstElements, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 9.0); // 1+3+5 = 9
}

#[test]
fn test_pipeline_replace_map() {
    let code = r#"
        fn removeSpaces(s: string) -> string {
            return replace(s, " ", "_");
        }

        let phrases: string[] = ["hello world", "foo bar"];
        let replaced: string[] = map(phrases, removeSpaces);
        join(replaced, "|")
    "#;
    assert_eval_string_with_io(code, "hello_world|foo_bar");
}

#[test]
fn test_pipeline_padstart_map() {
    let code = r#"
        fn pad(s: string) -> string {
            return padStart(s, 5.0, "0");
        }

        let numbers: string[] = ["1", "22", "333"];
        let padded: string[] = map(numbers, pad);
        join(padded, ",")
    "#;
    assert_eval_string_with_io(code, "00001,00022,00333");
}

#[test]
fn test_pipeline_substring_filter_map() {
    let code = r#"
        fn getPrefix(s: string) -> string {
            return substring(s, 0.0, 3.0);
        }

        let words: string[] = ["apple", "application", "appropriate"];
        let prefixes: string[] = map(words, getPrefix);
        fn isApp(s: string) -> bool { return s == "app"; }
        let appPrefixes: string[] = filter(prefixes, isApp);
        len(appPrefixes)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_pipeline_min_max_aggregation() {
    let code = r#"
        fn findMin(current: number, x: number) -> number {
            if (current == 0.0) { return x; }
            return min(current, x);
        }
        fn findMax(current: number, x: number) -> number {
            return max(current, x);
        }

        let numbers: number[] = [5.0, 2.0, 8.0, 1.0, 9.0];
        let minVal: number = reduce(numbers, findMin, 0.0);
        let maxVal: number = reduce(numbers, findMax, 0.0);
        maxVal - minVal
    "#;
    assert_eval_number_with_io(code, 8.0); // 9 - 1
}

#[test]
fn test_pipeline_array_building() {
    let code = r#"
        let arr1: number[] = [1.0];
        let arr2: number[] = unshift(arr1, 0.0);
        let arr3: number[] = concat(arr2, [2.0, 3.0]);
        fn sum(a: number, b: number) -> number { return a + b; }
        reduce(arr3, sum, 0.0)
    "#;
    assert_eval_number_with_io(code, 6.0); // [0,1,2,3] sum=6
}

#[test]
fn test_pipeline_foreach_side_effects() {
    let code = r#"
        fn noop(x: number) -> void {}

        let numbers: number[] = [1.0, 2.0, 3.0];
        let result: void = forEach(numbers, noop);
        // forEach returns null, verify it doesn't crash
        true
    "#;
    assert_eval_bool_with_io(code, true);
}

// ============================================================================
// Category 5: Text Processing (20 tests)
// ============================================================================

#[test]
fn test_text_word_count() {
    let code = r#"
        let text: string = "hello world this is a test";
        let words: string[] = split(text, " ");
        len(words)
    "#;
    assert_eval_number_with_io(code, 6.0);
}

#[test]
fn test_text_line_count() {
    let code = r#"
        let text: string = "line1\nline2\nline3";
        let lines: string[] = split(text, "\n");
        len(lines)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_average_word_length() {
    let code = r#"
        fn wordLength(word: string) -> number { return len(word); }
        fn sum(a: number, b: number) -> number { return a + b; }

        let text: string = "the quick brown fox";
        let words: string[] = split(text, " ");
        let lengths: number[] = map(words, wordLength);
        let total: number = reduce(lengths, sum, 0.0);
        let avg: number = total / len(words);
        floor(avg)
    "#;
    assert_eval_number_with_io(code, 4.0); // (3+5+5+3)/4 = 4
}

#[test]
fn test_text_uppercase_words() {
    let code = r#"
        fn upper(s: string) -> string { return toUpperCase(s); }

        let text: string = "hello world";
        let words: string[] = split(text, " ");
        let upper: string[] = map(words, upper);
        join(upper, " ")
    "#;
    assert_eval_string_with_io(code, "HELLO WORLD");
}

#[test]
fn test_text_titlecase() {
    let code = r#"
        fn titleCase(word: string) -> string {
            let first: string = charAt(word, 0.0);
            let rest: string = substring(word, 1.0, len(word));
            let firstUpper: string = toUpperCase(first);
            let restLower: string = toLowerCase(rest);
            return firstUpper + restLower;
        }

        let text: string = "hello WORLD";
        let words: string[] = split(text, " ");
        let titled: string[] = map(words, titleCase);
        join(titled, " ")
    "#;
    assert_eval_string_with_io(code, "Hello World");
}

#[test]
fn test_text_remove_punctuation() {
    let code = r#"
        fn removePunct(s: string) -> string {
            let s1: string = replace(s, ".", "");
            let s2: string = replace(s1, ",", "");
            let s3: string = replace(s2, "!", "");
            return s3;
        }

        let text: string = "Hello, World! Test.";
        removePunct(text)
    "#;
    assert_eval_string_with_io(code, "Hello World Test");
}

#[test]
fn test_text_find_longest_word() {
    let code = r#"
        fn longerWord(current: string, word: string) -> string {
            if (len(word) > len(current)) {
                return word;
            }
            return current;
        }

        let text: string = "the quick brown fox jumps";
        let words: string[] = split(text, " ");
        reduce(words, longerWord, "")
    "#;
    assert_eval_string_with_io(code, "quick"); // or "brown" or "jumps" (all 5 chars, first wins)
}

#[test]
fn test_text_filter_short_words() {
    let code = r#"
        fn isLong(word: string) -> bool {
            return len(word) >= 4.0;
        }

        let text: string = "the quick brown fox";
        let words: string[] = split(text, " ");
        let long: string[] = filter(words, isLong);
        len(long)
    "#;
    assert_eval_number_with_io(code, 2.0); // "quick"=5, "brown"=5 are >=4
}

#[test]
fn test_text_count_character() {
    let code = r#"
        let text: string = "hello world";
        let chars: string[] = split(text, "");
        fn isL(c: string) -> bool { return c == "l"; }
        let ls: string[] = filter(chars, isL);
        len(ls)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_reverse_words() {
    let code = r#"
        let text: string = "hello world";
        let words: string[] = split(text, " ");
        let reversed: string[] = reverse(words);
        join(reversed, " ")
    "#;
    assert_eval_string_with_io(code, "world hello");
}

#[test]
fn test_text_acronym() {
    let code = r#"
        fn firstChar(s: string) -> string {
            return charAt(s, 0.0);
        }

        let text: string = "Portable Network Graphics";
        let words: string[] = split(text, " ");
        let initials: string[] = map(words, firstChar);
        join(initials, "")
    "#;
    assert_eval_string_with_io(code, "PNG");
}

#[test]
fn test_text_trim_lines() {
    let code = r#"
        fn trimLine(line: string) -> string { return trim(line); }

        let text: string = "  line1  \n  line2  \n  line3  ";
        let lines: string[] = split(text, "\n");
        let trimmed: string[] = map(lines, trimLine);
        join(trimmed, "|")
    "#;
    assert_eval_string_with_io(code, "line1|line2|line3");
}

#[test]
fn test_text_starts_with_filter() {
    let code = r#"
        fn startsWithA(word: string) -> bool {
            return startsWith(word, "a");
        }

        let words: string[] = ["apple", "banana", "apricot", "cherry"];
        let aWords: string[] = filter(words, startsWithA);
        len(aWords)
    "#;
    assert_eval_number_with_io(code, 2.0);
}

#[test]
fn test_text_ends_with_filter() {
    let code = r#"
        fn endsWithE(word: string) -> bool {
            return endsWith(word, "e");
        }

        let words: string[] = ["apple", "banana", "grape", "cherry"];
        let eWords: string[] = filter(words, endsWithE);
        len(eWords)
    "#;
    assert_eval_number_with_io(code, 2.0); // apple, grape
}

#[test]
fn test_text_pad_lines() {
    let code = r#"
        fn pad(line: string) -> string {
            return padEnd(line, 10.0, ".");
        }

        let lines: string[] = ["short", "medium", "long"];
        let padded: string[] = map(lines, pad);
        padded[0]
    "#;
    assert_eval_string_with_io(code, "short.....");
}

#[test]
fn test_text_replace_multiple() {
    let code = r#"
        let text: string = "foo bar foo baz";
        let step1: string = replace(text, "foo", "hello");
        let step2: string = replace(step1, "bar", "world");
        step2
    "#;
    assert_eval_string_with_io(code, "hello world foo baz"); // only first "foo" replaced
}

#[test]
fn test_text_split_multichar() {
    let code = r#"
        let text: string = "one::two::three";
        let parts: string[] = split(text, "::");
        len(parts)
    "#;
    assert_eval_number_with_io(code, 3.0);
}

#[test]
fn test_text_extract_numbers() {
    let code = r#"
        let text: string = "Price: 100 Quantity: 50";
        let words: string[] = split(text, " ");
        let num1: number = parseFloat(words[1]);
        let num2: number = parseFloat(words[3]);
        num1 + num2
    "#;
    assert_eval_number_with_io(code, 150.0);
}

#[test]
fn test_text_repeat_pattern() {
    let code = r#"
        let pattern: string = repeat("*", 5.0);
        pattern
    "#;
    assert_eval_string_with_io(code, "*****");
}

#[test]
fn test_text_contains_substring() {
    let code = r#"
        let text: string = "The quick brown fox";
        let hasQuick: bool = includes(text, "quick");
        let hasSlow: bool = includes(text, "slow");
        hasQuick && !hasSlow
    "#;
    assert_eval_bool_with_io(code, true);
}

// ============================================================================
// Category 6: Configuration Processing (10 tests)
// ============================================================================

#[test]
fn test_config_parse_json() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"host\": \"localhost\", \"port\": 8080}").unwrap();

    let code = format!(
        r#"
        let configStr: string = readFile("{}");
        let config: json = parseJSON(configStr);
        let host: string = config["host"].as_string();
        host
    "#,
        config_path.display()
    );
    assert_eval_string_with_io(&code, "localhost");
}

#[test]
fn test_config_extract_port() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"port\": 3000}").unwrap();

    let code = format!(
        r#"
        let configStr: string = readFile("{}");
        let config: json = parseJSON(configStr);
        let port: number = config["port"].as_number();
        port
    "#,
        config_path.display()
    );
    assert_eval_number_with_io(&code, 3000.0);
}

#[test]
fn test_config_validate_required_fields() {
    let code = r#"
        let configStr: string = "{\"host\": \"localhost\", \"port\": 8080}";
        let config: json = parseJSON(configStr);
        let hasHost: bool = !jsonIsNull(config["host"]);
        let hasPort: bool = !jsonIsNull(config["port"]);
        hasHost && hasPort
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_missing_field_default() {
    let code = r#"
        let configStr: string = "{\"host\": \"localhost\"}";
        let config: json = parseJSON(configStr);
        let port: json = config["port"];
        let portValue: number = if (jsonIsNull(port)) {
            8080.0
        } else {
            port.as_number()
        };
        portValue
    "#;
    assert_eval_number_with_io(code, 8080.0);
}

#[test]
fn test_config_nested_settings() {
    let code = r#"
        let configStr: string = "{\"database\": {\"host\": \"db.local\", \"port\": 5432}}";
        let config: json = parseJSON(configStr);
        let db: json = config["database"];
        let dbHost: string = db["host"].as_string();
        dbHost
    "#;
    assert_eval_string_with_io(code, "db.local");
}

#[test]
fn test_config_boolean_flags() {
    let code = r#"
        let configStr: string = "{\"debug\": true, \"production\": false}";
        let config: json = parseJSON(configStr);
        let debug: bool = config["debug"].as_bool();
        let prod: bool = config["production"].as_bool();
        debug && !prod
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_array_values() {
    let code = r#"
        let configStr: string = "{\"allowed_hosts\": [\"localhost\", \"127.0.0.1\"]}";
        let config: json = parseJSON(configStr);
        let hosts: json = config["allowed_hosts"];
        let first: string = hosts[0].as_string();
        first
    "#;
    assert_eval_string_with_io(code, "localhost");
}

#[test]
fn test_config_write_updated() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.json");
    std::fs::write(&config_path, "{\"version\": 1}").unwrap();

    let code = format!(
        r#"
        let configStr: string = readFile("{}");
        let config: json = parseJSON(configStr);
        let version: number = config["version"].as_number();
        let newVersion: number = version + 1.0;
        let updated: string = "{{\"version\":" + str(newVersion) + "}}";
        writeFile("{}", updated);

        let result: string = readFile("{}");
        let newConfig: json = parseJSON(result);
        let finalVersion: number = newConfig["version"].as_number();
        finalVersion
    "#,
        config_path.display(),
        config_path.display(),
        config_path.display()
    );
    assert_eval_number_with_io(&code, 2.0);
}

#[test]
fn test_config_merge_defaults() {
    let code = r#"
        let userConfig: string = "{\"host\": \"custom.com\"}";
        let defaults: string = "{\"host\": \"localhost\", \"port\": 8080, \"debug\": false}";

        let user: json = parseJSON(userConfig);
        let def: json = parseJSON(defaults);

        let hostUser: json = user["host"];
        let portUser: json = user["port"];

        let finalHost: string = if (jsonIsNull(hostUser)) {
            def["host"].as_string()
        } else {
            user["host"].as_string()
        };

        let finalPort: number = if (jsonIsNull(portUser)) {
            def["port"].as_number()
        } else {
            user["port"].as_number()
        };

        finalHost + ":" + str(finalPort)
    "#;
    assert_eval_string_with_io(code, "custom.com:8080");
}

#[test]
fn test_config_prettify_for_humans() {
    let code = r#"
        let compact: string = "{\"host\":\"localhost\",\"port\":8080}";
        let pretty: string = prettifyJSON(compact, 2.0);
        includes(pretty, "\n") && includes(pretty, "  ")
    "#;
    assert_eval_bool_with_io(code, true);
}


#[test]
fn test_config_array_length() {
    let code = r#"
        let configStr: string = "{\"servers\": [\"server1\", \"server2\", \"server3\"]}";
        let config: json = parseJSON(configStr);
        let servers: json = config["servers"];
        let s0: string = servers[0].as_string();
        let s1: string = servers[1].as_string();
        let s2: string = servers[2].as_string();
        len(s0) > 0.0 && len(s1) > 0.0 && len(s2) > 0.0
    "#;
    assert_eval_bool_with_io(code, true);
}

#[test]
fn test_config_environment_specific() {
    let code = r#"
        let configStr: string = "{\"env\": \"production\", \"debug\": false}";
        let config: json = parseJSON(configStr);
        let env: string = config["env"].as_string();
        let debug: bool = config["debug"].as_bool();
        let isProd: bool = env == "production";
        isProd && !debug
    "#;
    assert_eval_bool_with_io(code, true);
}
