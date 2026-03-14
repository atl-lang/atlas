use super::super::*;
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
        fn isHighValueInStock(borrow row: string): bool {{
            let fields: string[] = row.split(",");
            let price: number = unwrap(fields[1].toNumber());
            let stock: number = unwrap(fields[2].toNumber());
            return price >= 1.0 && stock >= 100.0;
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let filtered: string[] = filter(dataLines, isHighValueInStock);
        len(filtered)
    "#,
        path_for_atlas(&csv_path)
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
        fn sumNorth(borrow total: number, borrow row: string): number {{
            let fields: string[] = row.split(",");
            let region: string = fields[0];
            let amount: number = unwrap(fields[1].toNumber());
            if (region == "North") {{
                return total + amount;
            }}
            return total;
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        reduce(dataLines, sumNorth, 0.0)
    "#,
        path_for_atlas(&csv_path)
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
        fn isError(borrow row: string): bool {{
            let fields: string[] = row.split(",");
            return fields[0] == "ERROR";
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let errors: string[] = filter(dataLines, isError);
        len(errors)
    "#,
        path_for_atlas(&csv_path)
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
        fn fullName(borrow row: string): string {{
            let fields: string[] = row.split(",");
            return fields[0] + " " + fields[1];
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let names: string[] = map(dataLines, fullName);
        names.join("; ")
    "#,
        path_for_atlas(&csv_path)
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
        fn calcPercentage(borrow row: string): number {{
            let fields: string[] = row.split(",");
            let sold: number = unwrap(fields[1].toNumber());
            let total: number = unwrap(fields[2].toNumber());
            return (sold / total) * 100.0;
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let row1: string = lines[1];

        calcPercentage(row1)
    "#,
        path_for_atlas(&csv_path)
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
        fn cleanRow(borrow row: string): string {{
            let fields: string[] = row.split(",");
            let name: string = fields[0].trim();
            let value: string = fields[1].trim();
            return name + "," + value;
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let row1: string = lines[1];

        cleanRow(row1)
    "#,
        path_for_atlas(&csv_path)
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
        fn isFruit(borrow row: string): bool {{
            let fields: string[] = row.split(",");
            let kind: string = fields[1].toLowerCase();
            return kind == "fruit";
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let fruits: string[] = filter(dataLines, isFruit);
        len(fruits)
    "#,
        path_for_atlas(&csv_path)
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
        fn hasError(borrow row: string): bool {{
            return row.includes("Error");
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let errors: string[] = filter(dataLines, hasError);
        len(errors)
    "#,
        path_for_atlas(&csv_path)
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
        fn compareById(borrow a: string, borrow b: string): number {{
            let fieldsA: string[] = a.split(",");
            let fieldsB: string[] = b.split(",");
            let idA: number = unwrap(fieldsA[0].toNumber());
            let idB: number = unwrap(fieldsB[0].toNumber());
            return idA - idB;
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let sorted: string[] = sort(dataLines, compareById);
        let first: string = sorted[0];
        let fields: string[] = first.split(",");
        fields[0]
    "#,
        path_for_atlas(&csv_path)
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
        let csv: string = file.read("{}").unwrap();
        let newRow: string = "Bob,90";
        let updated: string = csv + newRow + "\n";
        file.write("{}", updated);

        let result: string = file.read("{}").unwrap();
        let lines: string[] = result.split("\n");
        len(lines) - 1.0
    "#,
        path_for_atlas(&csv_path),
        path_for_atlas(&csv_path),
        path_for_atlas(&csv_path)
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
        fn hasThreeColumns(borrow row: string): bool {{
            let fields: string[] = row.split(",");
            return len(fields) == 3.0;
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let valid: string[] = filter(dataLines, hasThreeColumns);
        len(valid) == len(dataLines)
    "#,
        path_for_atlas(&csv_path)
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
        fn getCategory(borrow row: string): string {{
            let fields: string[] = row.split(",");
            return fields[1];
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        let categories: string[] = map(dataLines, getCategory);
        // Count unique by checking first occurrence
        let hasFruit: bool = categories.includes("fruit");
        let hasVeggie: bool = categories.includes("veggie");
        let hasMeat: bool = categories.includes("meat");

        str(hasFruit) + "," + str(hasVeggie) + "," + str(hasMeat)
    "#,
        path_for_atlas(&csv_path)
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
        fn addGrade(borrow row: string): string {{
            let fields: string[] = row.split(",");
            let score: number = unwrap(fields[1].toNumber());
            let mut grade: string = "F";
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

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let row1: string = lines[1];

        addGrade(row1)
    "#,
        path_for_atlas(&csv_path)
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
        fn findMin(borrow current: number, borrow row: string): number {{
            let fields: string[] = row.split(",");
            let temp: number = unwrap(fields[1].toNumber());
            if (current == 0.0) {{
                return temp;
            }}
            return Math.min(current, temp);
        }}

        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let dataLines: string[] = lines.slice(1, len(lines) - 1.0);

        reduce(dataLines, findMin, 0.0)
    "#,
        path_for_atlas(&csv_path)
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
        let csv: string = file.read("{}").unwrap();
        let lines: string[] = csv.split("\n");
        let row1: string = lines[1];
        let fields: string[] = row1.split(",");
        fields[0] + ", " + fields[1] + ", " + fields[2]
    "#,
        path_for_atlas(&csv_path)
    );
    assert_eval_string_with_io(&code, "Main St, NYC, NY");
}
