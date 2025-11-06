use anyhow::{Context, Result};
use csv::Terminator;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileFormat {
    Csv,
    Xlsx,
}

/// Detects the file format based on extension and content
pub fn detect_file_format(file_path: &Path) -> Result<FileFormat> {
    // First check by extension
    if let Some(ext) = file_path.extension() {
        let ext_str = ext.to_string_lossy().to_lowercase();
        match ext_str.as_str() {
            "xlsx" | "xlsm" | "xlsb" | "xls" => return Ok(FileFormat::Xlsx),
            "csv" | "tsv" | "txt" => return Ok(FileFormat::Csv),
            _ => {}
        }
    }

    // If extension is unclear, try to detect by content (magic bytes)
    let mut file = File::open(file_path).context("Failed to open file for format detection")?;
    let mut magic = [0u8; 4];

    use std::io::Read;
    if file.read_exact(&mut magic).is_ok() {
        // XLSX files are ZIP archives starting with PK
        if magic[0..2] == [0x50, 0x4B] {
            return Ok(FileFormat::Xlsx);
        }
        // XLS files start with D0 CF (OLE2)
        if magic[0..2] == [0xD0, 0xCF] {
            return Ok(FileFormat::Xlsx);
        }
    }

    // Default to CSV
    Ok(FileFormat::Csv)
}

/// Detects the CSV format by analyzing a sample of the file
pub fn detect_csv_format(file_path: &Path) -> Result<(u8, u8, Option<u8>, Terminator)> {
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

    let possible_delimiters = vec![b',', b';', b'\t', b'|'];

    #[derive(Debug)]
    struct DelimiterScore {
        delimiter: u8,
        total_score: f64,
    }

    let mut delimiter_scores: Vec<DelimiterScore> = Vec::new();

    for &delim in &possible_delimiters {
        // Count occurrences across all non-empty lines
        let mut counts: Vec<usize> = Vec::new();
        for line in &lines {
            if !line.is_empty() {
                let count = line.as_bytes().iter().filter(|&&c| c == delim).count();
                counts.push(count);
            }
        }

        if counts.is_empty() || counts.iter().all(|&c| c == 0) {
            continue;
        }

        let total: usize = counts.iter().sum();
        let avg_count = total as f64 / counts.len() as f64;

        let mut count_freq: std::collections::HashMap<usize, usize> =
            std::collections::HashMap::new();
        for &count in &counts {
            *count_freq.entry(count).or_insert(0) += 1;
        }
        let most_common_count_freq = count_freq.values().max().unwrap_or(&0);
        let consistency_ratio = *most_common_count_freq as f64 / counts.len() as f64;

        let total_score = avg_count * (0.7 + 0.3 * consistency_ratio);

        delimiter_scores.push(DelimiterScore {
            delimiter: delim,
            total_score,
        });
    }

    delimiter_scores.sort_by(|a, b| b.total_score.partial_cmp(&a.total_score).unwrap());
    let delimiter = delimiter_scores
        .first()
        .map(|s| s.delimiter)
        .unwrap_or(b',');

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
