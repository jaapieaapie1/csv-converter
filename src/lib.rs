#![allow(clippy::approx_constant)]

use anyhow::{Context, Result};
use csv::{ReaderBuilder, Terminator};
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

/// Detects the CSV format by analyzing a sample of the file
pub fn detect_csv_format(file_path: &PathBuf) -> Result<(u8, u8, Option<u8>, Terminator)> {
    let file = File::open(file_path).context("Failed to open file for format detection")?;
    let reader = BufReader::new(file);

    // Read first 250 lines for detection (or until EOF)
    // This gives us a better chance to detect escape characters
    let mut lines: Vec<String> = Vec::new();
    for (i, line) in reader.lines().enumerate() {
        if i >= 250 {
            break;
        }
        lines.push(line?);
    }

    if lines.is_empty() {
        return Ok((b',', b'"', None, Terminator::CRLF));
    }

    // Detect delimiter by counting common delimiters
    let possible_delimiters = vec![b',', b';', b'\t', b'|'];
    let mut delimiter_scores: Vec<(u8, usize)> = Vec::new();

    for &delim in &possible_delimiters {
        // Count occurrences in first non-empty line
        if let Some(first_line) = lines.first() {
            let count = first_line
                .as_bytes()
                .iter()
                .filter(|&&c| c == delim)
                .count();
            if count > 0 {
                // Check if count is consistent across lines
                let mut consistent = true;
                for line in &lines[1..] {
                    let line_count = line.as_bytes().iter().filter(|&&c| c == delim).count();
                    if line_count != count && !line.is_empty() {
                        consistent = false;
                        break;
                    }
                }
                if consistent {
                    delimiter_scores.push((delim, count));
                }
            }
        }
    }

    // Choose delimiter with highest count
    delimiter_scores.sort_by(|a, b| b.1.cmp(&a.1));
    let delimiter = delimiter_scores.first().map(|(d, _)| *d).unwrap_or(b',');

    // Detect line terminator
    let terminator = if lines.iter().any(|_| true) {
        // Default to CRLF for Windows compatibility, but csv crate handles both
        Terminator::CRLF
    } else {
        Terminator::CRLF
    };

    // Quote character is typically double quote
    let quote = b'"';

    // Detect escape character: look for \" (backslash escaping) vs "" (double quote escaping)
    let mut has_backslash_escape = false;
    let mut has_double_quote_escape = false;

    for line in &lines {
        // Look for \" pattern (backslash escaping)
        if line.contains("\\\"") {
            has_backslash_escape = true;
        }
        // Look for "" pattern inside quoted fields (double quote escaping)
        // This is trickier - look for patterns like "text""more"
        if line.contains("\"\"") {
            has_double_quote_escape = true;
        }
    }

    // Determine escape character
    let escape = if has_backslash_escape && !has_double_quote_escape {
        Some(b'\\')
    } else {
        // Default to None, which means use double-quote escaping (RFC 4180 standard)
        None
    };

    Ok((delimiter, quote, escape, terminator))
}

/// Converts a field value to appropriate JSON Value based on type inference
pub fn convert_field_value(
    field: &str,
    header_name: &str,
    no_type_conversion: bool,
    string_fields: &[String],
) -> Value {
    if no_type_conversion {
        // No type conversion - keep everything as strings except empty fields
        if field.is_empty() {
            Value::Null
        } else {
            Value::String(field.to_string())
        }
    } else if string_fields.iter().any(|f| f == header_name) {
        // Field is in the string_fields list - always keep as string
        if field.is_empty() {
            Value::Null
        } else {
            Value::String(field.to_string())
        }
    } else {
        // Smart type conversion, but preserve leading zeros (zipcodes, phone numbers, etc)
        let has_leading_zero =
            field.starts_with('0') && field.len() > 1 && !field.starts_with("0.");

        if field.is_empty() {
            Value::Null
        } else if field.eq_ignore_ascii_case("true") {
            Value::Bool(true)
        } else if field.eq_ignore_ascii_case("false") {
            Value::Bool(false)
        } else if !has_leading_zero {
            // Only try to parse as number if no leading zero
            if let Ok(num) = field.parse::<i64>() {
                Value::Number(num.into())
            } else if let Ok(num) = field.parse::<f64>() {
                if let Some(n) = serde_json::Number::from_f64(num) {
                    Value::Number(n)
                } else {
                    Value::String(field.to_string())
                }
            } else {
                Value::String(field.to_string())
            }
        } else {
            // Has leading zero - keep as string to preserve it
            Value::String(field.to_string())
        }
    }
}

/// Converts CSV to NDJSON with streaming to handle large files
pub fn convert_csv_to_ndjson(
    input_path: &PathBuf,
    output_path: Option<&PathBuf>,
    delimiter: u8,
    quote: u8,
    escape: Option<u8>,
    _terminator: Terminator,
    no_type_conversion: bool,
    string_fields: &[String],
) -> Result<()> {
    // Open input file
    let file =
        File::open(input_path).context(format!("Failed to open input file: {:?}", input_path))?;

    // Build CSV reader with detected/specified format
    let mut builder = ReaderBuilder::new();
    builder
        .delimiter(delimiter)
        .quote(quote)
        .flexible(true) // Handle varying column counts
        .has_headers(true);

    // Configure escape handling
    if let Some(esc) = escape {
        // Use explicit escape character (e.g., backslash)
        builder.escape(Some(esc)).double_quote(false);
    } else {
        // Use double-quote escaping (RFC 4180 standard: "" for literal quotes)
        builder.double_quote(true);
    }

    let mut reader = builder.from_reader(BufReader::with_capacity(32 * 1024, file));

    // Get headers
    let headers = reader
        .headers()
        .context("Failed to read CSV headers")?
        .clone();

    // Open output writer (file or stdout)
    let mut writer: Box<dyn Write> = if let Some(output) = output_path {
        Box::new(BufWriter::new(
            File::create(output).context("Failed to create output file")?,
        ))
    } else {
        Box::new(BufWriter::new(std::io::stdout()))
    };

    // Stream through records and convert each to JSON
    let mut record_count = 0;
    for result in reader.records() {
        let record = result.context("Failed to read CSV record")?;

        // Build JSON object from record
        let mut json_obj = Map::new();
        for (i, field) in record.iter().enumerate() {
            // Get header name or create a default one
            let header_name = headers
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("column_{}", i));

            let value = convert_field_value(field, &header_name, no_type_conversion, string_fields);

            json_obj.insert(header_name, value);
        }

        // Write JSON object as a single line
        let json_line = serde_json::to_string(&json_obj).context("Failed to serialize JSON")?;
        writeln!(writer, "{}", json_line).context("Failed to write output")?;

        record_count += 1;

        // Progress indicator for large files (every 10k records)
        if record_count % 10000 == 0 {
            eprintln!("Processed {} records...", record_count);
        }
    }

    writer.flush().context("Failed to flush output")?;
    eprintln!("Conversion complete! Processed {} records.", record_count);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_field_value_integers() {
        let value = convert_field_value("42", "age", false, &[]);
        assert!(matches!(value, Value::Number(_)));
        if let Value::Number(n) = value {
            assert_eq!(n.as_i64(), Some(42));
        }
    }

    #[test]
    fn test_convert_field_value_floats() {
        let value = convert_field_value("3.14", "price", false, &[]);
        assert!(matches!(value, Value::Number(_)));
        if let Value::Number(n) = value {
            assert_eq!(n.as_f64(), Some(3.14));
        }
    }

    #[test]
    fn test_convert_field_value_booleans() {
        let value_true = convert_field_value("true", "active", false, &[]);
        assert_eq!(value_true, Value::Bool(true));

        let value_false = convert_field_value("FALSE", "active", false, &[]);
        assert_eq!(value_false, Value::Bool(false));
    }

    #[test]
    fn test_convert_field_value_leading_zeros() {
        let value = convert_field_value("02134", "zipcode", false, &[]);
        assert_eq!(value, Value::String("02134".to_string()));
    }

    #[test]
    fn test_convert_field_value_decimal_leading_zero() {
        let value = convert_field_value("0.5", "score", false, &[]);
        assert!(matches!(value, Value::Number(_)));
    }

    #[test]
    fn test_convert_field_value_empty_to_null() {
        let value = convert_field_value("", "field", false, &[]);
        assert_eq!(value, Value::Null);
    }

    #[test]
    fn test_convert_field_value_string_fields() {
        let string_fields = vec!["zipcode".to_string()];
        let value = convert_field_value("12345", "zipcode", false, &string_fields);
        assert_eq!(value, Value::String("12345".to_string()));
    }

    #[test]
    fn test_convert_field_value_no_type_conversion() {
        let value = convert_field_value("42", "age", true, &[]);
        assert_eq!(value, Value::String("42".to_string()));

        let value = convert_field_value("true", "active", true, &[]);
        assert_eq!(value, Value::String("true".to_string()));
    }

    #[test]
    fn test_convert_field_value_strings() {
        let value = convert_field_value("Hello World", "name", false, &[]);
        assert_eq!(value, Value::String("Hello World".to_string()));
    }

    #[test]
    fn test_convert_field_value_negative_numbers() {
        let value = convert_field_value("-42", "temp", false, &[]);
        assert!(matches!(value, Value::Number(_)));
        if let Value::Number(n) = value {
            assert_eq!(n.as_i64(), Some(-42));
        }
    }
}
