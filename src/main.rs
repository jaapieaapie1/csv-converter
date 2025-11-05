use anyhow::Result;
use clap::Parser;
use csv::Terminator;
use std::path::PathBuf;

use csv_converter::{convert_csv_to_ndjson, detect_csv_format};

#[derive(Parser, Debug)]
#[command(
    name = "csv-converter",
    about = "Converts CSV files to newline-delimited JSON with automatic format detection"
)]
struct Args {
    /// Input CSV file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output NDJSON file path (optional, defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Override delimiter detection (e.g., ',', ';', '\t')
    #[arg(short, long)]
    delimiter: Option<char>,

    /// Override quote character detection (default: '"')
    #[arg(short, long)]
    quote: Option<char>,

    /// Override escape character detection (e.g., '\\' for backslash escaping, or none for "" escaping)
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
}

fn main() -> Result<()> {
    let args = Args::parse();

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
    convert_csv_to_ndjson(
        &args.input,
        args.output.as_ref(),
        delimiter,
        quote,
        escape,
        terminator,
        args.no_type_conversion,
        &args.string_fields,
    )?;

    Ok(())
}
