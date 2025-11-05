use anyhow::Result;
use clap::Parser;
use csv::Terminator;
use std::path::PathBuf;

use csv_converter::{detect_csv_format, detect_file_format, CsvParser, FileFormat, XlsxParser};
use csv_converter::parsers::Parser as ParserTrait;

#[derive(clap::Parser, Debug)]
#[command(
    name = "csv-converter",
    about = "Converts CSV and XLSX files to newline-delimited JSON with automatic format detection"
)]
struct Args {
    /// Input file path (CSV or XLSX)
    #[arg(short, long)]
    input: PathBuf,

    /// Output NDJSON file path (optional, defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Override delimiter detection for CSV files (e.g., ',', ';', '\t')
    #[arg(short, long)]
    delimiter: Option<char>,

    /// Override quote character detection for CSV files (default: '"')
    #[arg(short, long)]
    quote: Option<char>,

    /// Override escape character detection for CSV files (e.g., '\\' for backslash escaping, or none for "" escaping)
    #[arg(short, long)]
    escape: Option<char>,

    /// Disable auto-detection and use standard CSV format
    #[arg(long)]
    no_auto_detect: bool,

    /// Keep all values as strings (disable type conversion)
    #[arg(long)]
    no_type_conversion: bool,

    /// Field names to keep as strings (comma-separated, e.g., "zipcode,phone")
    #[arg(long, value_delimiter = ',')]
    string_fields: Vec<String>,

    /// For XLSX files: specify which sheet to read (default: first sheet)
    #[arg(short, long)]
    sheet: Option<String>,

    /// Force format type (csv or xlsx) instead of auto-detection
    #[arg(long)]
    format: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Detect file format
    let format = if let Some(format_str) = &args.format {
        match format_str.to_lowercase().as_str() {
            "csv" => FileFormat::Csv,
            "xlsx" | "xls" => FileFormat::Xlsx,
            _ => {
                eprintln!("Unknown format '{}', auto-detecting...", format_str);
                detect_file_format(&args.input)?
            }
        }
    } else {
        detect_file_format(&args.input)?
    };

    match format {
        FileFormat::Csv => {
            eprintln!("Detected format: CSV");

            // Detect or use specified CSV format
            let (delimiter, quote, escape, terminator) = if args.no_auto_detect {
                (
                    args.delimiter.unwrap_or(',') as u8,
                    args.quote.unwrap_or('"') as u8,
                    args.escape.map(|c| c as u8),
                    Terminator::CRLF,
                )
            } else {
                let (detected_delim, detected_quote, detected_escape, detected_term) =
                    detect_csv_format(&args.input)?;
                (
                    args.delimiter.map(|c| c as u8).unwrap_or(detected_delim),
                    args.quote.map(|c| c as u8).unwrap_or(detected_quote),
                    args.escape.map(|c| c as u8).or(detected_escape),
                    detected_term,
                )
            };

            if let Some(esc) = escape {
                eprintln!(
                    "Using delimiter: '{}', quote: '{}', escape: '{}'",
                    delimiter as char, quote as char, esc as char
                );
            } else {
                eprintln!(
                    "Using delimiter: '{}', quote: '{}', escape: double-quote (\"\")",
                    delimiter as char, quote as char
                );
            }

            // Convert CSV to NDJSON
            let parser = CsvParser::new(delimiter, quote, escape, terminator);
            parser.convert_to_ndjson(
                &args.input,
                args.output.as_deref(),
                args.no_type_conversion,
                &args.string_fields,
            )?;
        }
        FileFormat::Xlsx => {
            eprintln!("Detected format: XLSX");

            if args.delimiter.is_some()
                || args.quote.is_some()
                || args.escape.is_some()
                || args.no_auto_detect
            {
                eprintln!(
                    "Warning: CSV-specific options (delimiter, quote, escape, no-auto-detect) are ignored for XLSX files"
                );
            }

            // Convert XLSX to NDJSON
            let parser = if let Some(sheet_name) = args.sheet {
                XlsxParser::with_sheet(sheet_name)
            } else {
                XlsxParser::new()
            };

            parser.convert_to_ndjson(
                &args.input,
                args.output.as_deref(),
                args.no_type_conversion,
                &args.string_fields,
            )?;
        }
    }

    Ok(())
}
