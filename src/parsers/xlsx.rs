use anyhow::{Context, Result};
use calamine::{open_workbook, DataType, Reader, Xlsx};
use serde_json::Map;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::value_conversion::convert_field_value;

use super::Parser;

pub struct XlsxParser {
    pub sheet_name: Option<String>,
}

impl XlsxParser {
    pub fn new() -> Self {
        Self { sheet_name: None }
    }

    pub fn with_sheet(sheet_name: String) -> Self {
        Self {
            sheet_name: Some(sheet_name),
        }
    }
}

impl Default for XlsxParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Parser for XlsxParser {
    /// Converts XLSX to NDJSON with streaming-like behavior
    fn convert_to_ndjson(
        &self,
        input_path: &Path,
        output_path: Option<&Path>,
        no_type_conversion: bool,
        string_fields: &[String],
    ) -> Result<()> {
        // Open the workbook
        let mut workbook: Xlsx<_> =
            open_workbook(input_path).context("Failed to open XLSX file")?;

        // Get the sheet to read from
        let sheet_name = if let Some(name) = &self.sheet_name {
            name.clone()
        } else {
            // Use the first sheet if no sheet name is specified
            workbook
                .sheet_names()
                .first()
                .context("No sheets found in workbook")?
                .clone()
        };

        eprintln!("Reading from sheet: {}", sheet_name);

        // Read the range from the sheet
        let range = workbook
            .worksheet_range(&sheet_name)
            .ok_or_else(|| anyhow::anyhow!("Sheet '{}' not found", sheet_name))?
            .context(format!("Failed to read sheet: {}", sheet_name))?;

        // Open output writer (file or stdout)
        let mut writer: Box<dyn Write> = if let Some(output) = output_path {
            Box::new(BufWriter::new(
                File::create(output).context("Failed to create output file")?,
            ))
        } else {
            Box::new(BufWriter::new(std::io::stdout()))
        };

        // Get dimensions
        let (rows, cols) = range.get_size();

        if rows == 0 {
            eprintln!("Sheet is empty, no records to process.");
            return Ok(());
        }

        // First row is headers
        let mut headers: Vec<String> = Vec::new();
        for col in 0..cols {
            let header = range
                .get_value((0, col as u32))
                .map(datatype_to_string)
                .unwrap_or_else(|| format!("column_{}", col));
            headers.push(header);
        }

        // Process each row (skip header row)
        let mut record_count = 0;
        for row in 1..rows {
            let mut json_obj = Map::new();

            for (col, header_name) in headers.iter().enumerate() {
                let cell_value = range.get_value((row as u32, col as u32));

                let value = match cell_value {
                    Some(DataType::Empty) | None => serde_json::Value::Null,
                    Some(cell) => {
                        let str_value = datatype_to_string(cell);
                        convert_field_value(
                            &str_value,
                            header_name,
                            no_type_conversion,
                            string_fields,
                        )
                    }
                };

                json_obj.insert(header_name.clone(), value);
            }

            // Write JSON object as a single line
            let json_line =
                serde_json::to_string(&json_obj).context("Failed to serialize JSON")?;
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
}

/// Convert calamine DataType to a string representation
fn datatype_to_string(data: &DataType) -> String {
    match data {
        DataType::Int(i) => i.to_string(),
        DataType::Float(f) => {
            // Handle float formatting - remove unnecessary decimal points
            if f.fract() == 0.0 && f.abs() < i64::MAX as f64 {
                format!("{:.0}", f)
            } else {
                f.to_string()
            }
        }
        DataType::String(s) => s.clone(),
        DataType::Bool(b) => b.to_string(),
        DataType::DateTime(dt) => format!("{}", dt),
        DataType::Duration(d) => format!("{}", d),
        DataType::DateTimeIso(dt) => dt.clone(),
        DataType::DurationIso(d) => d.clone(),
        DataType::Error(e) => format!("ERROR: {:?}", e),
        DataType::Empty => String::new(),
    }
}
