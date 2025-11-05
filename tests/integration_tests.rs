use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Helper to create a temporary CSV file for testing
fn create_temp_csv(name: &str, content: &str) -> PathBuf {
    let path = PathBuf::from(format!("tests/fixtures/{}", name));
    fs::create_dir_all("tests/fixtures").unwrap();
    let mut file = File::create(&path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    path
}

/// Helper to run the converter and get output
fn run_converter(args: &[&str]) -> String {
    let output = Command::new("./target/release/csv-converter")
        .args(args)
        .output()
        .expect("Failed to run converter");

    String::from_utf8(output.stdout).unwrap()
}

/// Helper to clean up a specific temp file
fn cleanup_temp_file(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_basic_csv_conversion() {
    let csv_content = "name,age,active\nAlice,30,true\nBob,25,false\n";
    let input = create_temp_csv("basic.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""name":"Alice""#));
    assert!(output.contains(r#""age":30"#));
    assert!(output.contains(r#""active":true"#));
    assert!(output.contains(r#""name":"Bob""#));
    assert!(output.contains(r#""age":25"#));
    assert!(output.contains(r#""active":false"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_semicolon_delimiter_detection() {
    let csv_content = "name;price;quantity\nWidget A;19.99;100\nWidget B;29.50;50\n";
    let input = create_temp_csv("semicolon.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""name":"Widget A""#));
    assert!(output.contains(r#""price":19.99"#));
    assert!(output.contains(r#""quantity":100"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_tab_delimiter_detection() {
    let csv_content = "name\tprice\tquantity\nWidget A\t19.99\t100\nWidget B\t29.50\t50\n";
    let input = create_temp_csv("tab.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""name":"Widget A""#));
    assert!(output.contains(r#""price":19.99"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_double_quote_escaping() {
    let csv_content = r#"name,description
"Bob ""Bobby"" Smith","He said ""Hello"""
"#;
    let input = create_temp_csv("double_quote.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""name":"Bob \"Bobby\" Smith""#));
    assert!(output.contains(r#""description":"He said \"Hello\"""#));

    cleanup_temp_file(&input);
}

#[test]
fn test_backslash_escaping() {
    let csv_content = r#"name,description
"Bob \"Bobby\" Smith","He said \"Hello\""
"#;
    let input = create_temp_csv("backslash.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap(), "--escape", "\\"]);

    assert!(output.contains(r#""name":"Bob \"Bobby\" Smith""#));
    assert!(output.contains(r#""description":"He said \"Hello\"""#));

    cleanup_temp_file(&input);
}

#[test]
fn test_leading_zero_preservation() {
    let csv_content = "zipcode,phone,age\n02134,0123456789,30\n10001,5551234567,25\n";
    let input = create_temp_csv("leading_zeros.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    // Leading zeros should be preserved
    assert!(output.contains(r#""zipcode":"02134""#));
    assert!(output.contains(r#""phone":"0123456789""#));
    // Regular numbers should be numbers
    assert!(output.contains(r#""age":30"#));
    // Zipcode without leading zero becomes number (unless --string-fields is used)
    assert!(output.contains(r#""zipcode":10001"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_string_fields_option() {
    let csv_content = "zipcode,phone,age\n02134,0123456789,30\n10001,5551234567,25\n";
    let input = create_temp_csv("string_fields.csv", csv_content);

    let output = run_converter(&[
        "--input",
        input.to_str().unwrap(),
        "--string-fields",
        "zipcode,phone",
    ]);

    // All zipcodes and phones should be strings
    assert!(output.contains(r#""zipcode":"02134""#));
    assert!(output.contains(r#""zipcode":"10001""#));
    assert!(output.contains(r#""phone":"0123456789""#));
    assert!(output.contains(r#""phone":"5551234567""#));
    // Age should still be a number
    assert!(output.contains(r#""age":30"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_no_type_conversion() {
    let csv_content = "name,age,price,active\nAlice,30,19.99,true\n";
    let input = create_temp_csv("no_conversion.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap(), "--no-type-conversion"]);

    // Everything should be strings
    assert!(output.contains(r#""name":"Alice""#));
    assert!(output.contains(r#""age":"30""#));
    assert!(output.contains(r#""price":"19.99""#));
    assert!(output.contains(r#""active":"true""#));

    cleanup_temp_file(&input);
}

#[test]
fn test_empty_fields_to_null() {
    let csv_content = "name,email,phone\nAlice,alice@example.com,\nBob,,555-1234\n";
    let input = create_temp_csv("empty_fields.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    // Empty fields should be null
    assert!(output.contains(r#""phone":null"#));
    assert!(output.contains(r#""email":null"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_quoted_fields_with_commas() {
    let csv_content = r#"name,address,city
"Smith, John","123 Main St, Apt 4","Boston, MA"
"#;
    let input = create_temp_csv("quoted_commas.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""name":"Smith, John""#));
    assert!(output.contains(r#""address":"123 Main St, Apt 4""#));
    assert!(output.contains(r#""city":"Boston, MA""#));

    cleanup_temp_file(&input);
}

#[test]
fn test_pipe_delimiter() {
    let csv_content = "name|age|city\nAlice|30|Boston\nBob|25|NYC\n";
    let input = create_temp_csv("pipe.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""name":"Alice""#));
    assert!(output.contains(r#""age":30"#));
    assert!(output.contains(r#""city":"Boston""#));

    cleanup_temp_file(&input);
}

#[test]
fn test_type_inference_integers() {
    let csv_content = "int_col\n42\n-100\n0\n";
    let input = create_temp_csv("integers.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""int_col":42"#));
    assert!(output.contains(r#""int_col":-100"#));
    assert!(output.contains(r#""int_col":0"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_type_inference_floats() {
    let csv_content = "float_col\n3.14\n-2.5\n0.0\n0.99\n";
    let input = create_temp_csv("floats.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    assert!(output.contains(r#""float_col":3.14"#));
    assert!(output.contains(r#""float_col":-2.5"#));
    assert!(output.contains(r#""float_col":0.0"#));
    assert!(output.contains(r#""float_col":0.99"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_type_inference_booleans() {
    let csv_content = "bool_col\ntrue\nfalse\nTrue\nFalse\nTRUE\nFALSE\n";
    let input = create_temp_csv("booleans.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    // All variations should become booleans
    assert!(output.contains(r#""bool_col":true"#));
    assert!(output.contains(r#""bool_col":false"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_decimal_with_leading_zero() {
    let csv_content = "value\n0.5\n0.99\n0.1\n";
    let input = create_temp_csv("decimals.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap()]);

    // Decimals starting with 0. should still be numbers
    assert!(output.contains(r#""value":0.5"#));
    assert!(output.contains(r#""value":0.99"#));
    assert!(output.contains(r#""value":0.1"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_output_to_file() {
    let csv_content = "name,age\nAlice,30\n";
    let input = create_temp_csv("output_test.csv", csv_content);
    let output_path = "tests/fixtures/output.ndjson";

    let _ = run_converter(&["--input", input.to_str().unwrap(), "--output", output_path]);

    // Read the output file
    let output_content = fs::read_to_string(output_path).unwrap();
    assert!(output_content.contains(r#""name":"Alice""#));
    assert!(output_content.contains(r#""age":30"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_manual_delimiter_override() {
    let csv_content = "name;age;city\nAlice;30;Boston\n";
    let input = create_temp_csv("manual_delim.csv", csv_content);

    let output = run_converter(&["--input", input.to_str().unwrap(), "--delimiter", ";"]);

    assert!(output.contains(r#""name":"Alice""#));
    assert!(output.contains(r#""age":30"#));

    cleanup_temp_file(&input);
}

#[test]
fn test_complex_csv_with_all_features() {
    let csv_content = r#"name,zipcode,phone,price,description,active
"Alice ""A"" Smith",02134,0123456789,19.99,"A product with, commas",true
Bob,10001,,29.50,"Another ""quoted"" item",false
Charlie,00501,5551234567,0.99,,true
"#;
    let input = create_temp_csv("complex.csv", csv_content);

    let output = run_converter(&[
        "--input",
        input.to_str().unwrap(),
        "--string-fields",
        "zipcode,phone",
    ]);

    // Check name with quotes
    assert!(output.contains(r#""name":"Alice \"A\" Smith""#));
    // Check leading zero preservation
    assert!(output.contains(r#""zipcode":"02134""#));
    assert!(output.contains(r#""zipcode":"10001""#)); // String due to --string-fields
                                                      // Check empty field
    assert!(output.contains(r#""phone":null"#));
    // Check number
    assert!(output.contains(r#""price":19.99"#));
    // Check quoted description with comma
    assert!(output.contains(r#""description":"A product with, commas""#));
    // Check boolean
    assert!(output.contains(r#""active":true"#));

    cleanup_temp_file(&input);
}
