#![allow(clippy::approx_constant)]

pub mod format_detection;
pub mod parsers;
pub mod value_conversion;

// Re-export commonly used items for convenience
pub use format_detection::{detect_csv_format, detect_file_format, FileFormat};
pub use parsers::{csv::CsvParser, xlsx::XlsxParser, Parser};
pub use value_conversion::convert_field_value;

use anyhow::{Context, Result};
use csv::Terminator;
use std::path::Path;

/// High-level function to convert any supported format to NDJSON
/// This function auto-detects the file format and uses the appropriate parser
pub fn convert_to_ndjson(
    input_path: &Path,
    output_path: Option<&Path>,
    no_type_conversion: bool,
    string_fields: &[String],
) -> Result<()> {
    let format = detect_file_format(input_path)?;

    match format {
        FileFormat::Csv => {
            let (delimiter, quote, escape, terminator) = detect_csv_format(input_path)?;
            let parser = CsvParser::new(delimiter, quote, escape, terminator);
            parser.convert_to_ndjson(input_path, output_path, no_type_conversion, string_fields)
        }
        FileFormat::Xlsx => {
            let parser = XlsxParser::new();
            parser.convert_to_ndjson(input_path, output_path, no_type_conversion, string_fields)
        }
    }
}

/// Legacy function for backwards compatibility - converts CSV to NDJSON
pub fn convert_csv_to_ndjson(
    input_path: &Path,
    output_path: Option<&Path>,
    delimiter: u8,
    quote: u8,
    escape: Option<u8>,
    terminator: Terminator,
    no_type_conversion: bool,
    string_fields: &[String],
) -> Result<()> {
    let parser = CsvParser::new(delimiter, quote, escape, terminator);
    parser
        .convert_to_ndjson(input_path, output_path, no_type_conversion, string_fields)
        .context("Failed to convert CSV to NDJSON")
}

/// Converts XLSX to NDJSON
pub fn convert_xlsx_to_ndjson(
    input_path: &Path,
    output_path: Option<&Path>,
    no_type_conversion: bool,
    string_fields: &[String],
    sheet_name: Option<String>,
) -> Result<()> {
    let parser = if let Some(name) = sheet_name {
        XlsxParser::with_sheet(name)
    } else {
        XlsxParser::new()
    };

    parser
        .convert_to_ndjson(input_path, output_path, no_type_conversion, string_fields)
        .context("Failed to convert XLSX to NDJSON")
}
