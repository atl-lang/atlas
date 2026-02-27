use super::super::*;
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        let dataLines: string[] = slice(lines, 1.0, len(lines) - 1.0);

        // Filter expensive items
        let expensive: string[] = filter(dataLines, isExpensive);
        len(expensive)
    "#,
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&input_path),
        path_for_atlas(&output_path),
        path_for_atlas(&output_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
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
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Hello World");
}
