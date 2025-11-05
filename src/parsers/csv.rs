use anyhow::{Context, Result};
use csv::{ReaderBuilder, Terminator};
use serde_json::Map;
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use crate::value_conversion::convert_field_value;

use super::Parser;

pub struct CsvParser {
    pub delimiter: u8,
    pub quote: u8,
    pub escape: Option<u8>,
    pub terminator: Terminator,
}

impl CsvParser {
    pub fn new(delimiter: u8, quote: u8, escape: Option<u8>, terminator: Terminator) -> Self {
        Self {
            delimiter,
            quote,
            escape,
            terminator,
        }
    }
}

impl Parser for CsvParser {
    /// Converts CSV to NDJSON with streaming to handle large files
    fn convert_to_ndjson(
        &self,
        input_path: &Path,
        output_path: Option<&Path>,
        no_type_conversion: bool,
        string_fields: &[String],
    ) -> Result<()> {
        // Open input file
        let file = File::open(input_path)
            .context(format!("Failed to open input file: {:?}", input_path))?;

        // Build CSV reader with detected/specified format
        let mut builder = ReaderBuilder::new();
        builder
            .delimiter(self.delimiter)
            .quote(self.quote)
            .flexible(true) // Handle varying column counts
            .has_headers(true);

        // Configure escape handling
        if let Some(esc) = self.escape {
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

                let value =
                    convert_field_value(field, &header_name, no_type_conversion, string_fields);

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
}
