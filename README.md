# CSV to NDJSON Converter

[![CI](https://github.com/jaapieaapie1/csv-converter/actions/workflows/ci.yml/badge.svg)](https://github.com/jaapieaapie1/csv-converter/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, memory-efficient CSV (and XLSX) to newline-delimited JSON (NDJSON) converter written in Rust. This tool automatically detects CSV formats and streams data to handle files of any size.

## Why NDJSON instead of JSON
Well, with NDJSON (each object being on a new line instead of being in 1 json array) it's significantly easier to proces rows in batches/in parellel because you can buffer your reads of disk and paralise the json deserialization.

## Features

- **Automatic Format Detection**: Automatically detects delimiter (comma, semicolon, tab, pipe), quote characters, and line terminators
- **Streaming Processing**: Memory-efficient streaming means it can handle files hundreds of megabytes or larger
- **Flexible Input**: Handles various CSV formats including:
  - Different delimiters (`,`, `;`, `\t`, `|`)
  - Quote escaping: both RFC 4180 (`""`) and backslash (`\"`) styles
  - Auto-detects escape method or can be manually specified
  - Various row delimiters (CRLF, LF)
- **Smart Type Conversion**: Automatically converts values to appropriate JSON types:
  - Integers and floats to numbers (unless they have leading zeros)
  - "true"/"false" to booleans
  - Empty fields to null
  - Fields with leading zeros kept as strings (preserves zipcodes like "02134")
  - Option to specify fields that should always be strings (`--string-fields`)
  - Option to disable all type conversion (`--no-type-conversion`)

## Installation

Build the project:

```bash
cargo build --release
```

The binary will be available at `./target/release/csv-converter`

## Usage

### Basic Usage

Convert CSV to NDJSON (output to stdout):
```bash
csv-converter --input data.csv
```

Convert CSV to NDJSON file:
```bash
csv-converter --input data.csv --output data.ndjson
```

### Type Conversion Options

Keep specific fields as strings (useful for zipcodes, phone numbers, etc.):
```bash
csv-converter --input data.csv --string-fields zipcode,phone
```

Disable all type conversion (keep everything as strings):
```bash
csv-converter --input data.csv --no-type-conversion
```

### CSV Format Options

Override delimiter detection:
```bash
csv-converter --input data.csv --delimiter ';'
```

Override quote character:
```bash
csv-converter --input data.csv --quote "'"
```

Override escape character (for backslash-style escaping):
```bash
csv-converter --input data.csv --escape '\'
```

Disable auto-detection (use standard CSV format):
```bash
csv-converter --input data.csv --no-auto-detect
```

### Help

```bash
csv-converter --help
```

## Examples

### Example 1: Preserving Leading Zeros
The converter automatically preserves leading zeros in fields like zipcodes and phone numbers:

Input (`contacts.csv`):
```csv
name,zipcode,phone,age
Alice,02134,0123456789,25
Bob,10001,5551234567,30
```

Command:
```bash
csv-converter --input contacts.csv
```

Output:
```json
{"name":"Alice","zipcode":"02134","phone":"0123456789","age":25}
{"name":"Bob","zipcode":10001,"phone":5551234567,"age":30}
```

Note: Bob's zipcode and phone became numbers because they don't have leading zeros. To force them as strings:
```bash
csv-converter --input contacts.csv --string-fields zipcode,phone
```

Output:
```json
{"name":"Alice","zipcode":"02134","phone":"0123456789","age":25}
{"name":"Bob","zipcode":"10001","phone":"5551234567","age":30}
```

### Example 2: Standard CSV
Input (`data.csv`):
```csv
name,age,email,active
John Doe,30,john@example.com,true
Jane Smith,25,jane@example.com,false
```

Command:
```bash
csv-converter --input data.csv
```

Output:
```json
{"name":"John Doe","age":30,"email":"john@example.com","active":true}
{"name":"Jane Smith","age":25,"email":"jane@example.com","active":false}
```

### Example 3: Semicolon-delimited with quotes
Input (`data.csv`):
```csv
product;price;quantity
"Widget ""Pro""";19.99;100
Gadget;29.50;50
```

Command:
```bash
csv-converter --input data.csv --output output.ndjson
```

Output (`output.ndjson`):
```json
{"product":"Widget \"Pro\"","price":19.99,"quantity":100}
{"product":"Gadget","price":29.5,"quantity":50}
```

### Example 4: Quote Escaping Styles

The converter auto-detects and handles both quote escaping styles:

**RFC 4180 style (double-quote escaping `""`)**:
```csv
name,quote
"Bob ""Bobby"" Smith","He said ""Hello"""
```
Result: `{"name":"Bob \"Bobby\" Smith","quote":"He said \"Hello\""}`

**Backslash escaping (`\"`)** - use `--escape '\'`:
```csv
name,quote
"Bob \"Bobby\" Smith","He said \"Hello\""
```
Result: `{"name":"Bob \"Bobby\" Smith","quote":"He said \"Hello\""}`

Both produce the same JSON output. The tool auto-detects the style or you can specify it manually.

## Performance

The converter is highly optimized for speed and memory efficiency:

### Benchmark Results

Tested on Apple Silicon (M-series) with standard CSV files (8 columns, mixed types):

| Rows     | Time    | Throughput      | Memory |
|----------|---------|-----------------|--------|
| 10K      | 0.13s   | ~76K rows/sec   | <5 MB  |
| 100K     | 0.17s   | ~585K rows/sec  | <5 MB  |
| 500K     | 0.77s   | ~648K rows/sec  | <5 MB  |
| 2M       | ~3s     | ~667K rows/sec  | <5 MB  |

**Key Performance Features:**
- ⚡ **~600K rows/sec** sustained throughput
- 💾 **Constant memory usage** - streams data, never loads entire file
- 🔥 **CPU-bound** - maxes out single core (84-87% CPU usage)
- 📝 **Buffered I/O** - I/O overhead is negligible on modern SSDs
- 🎯 **Zero-copy parsing** where possible

### Performance Characteristics

- **Memory:** Fixed ~5MB regardless of file size (streaming architecture)
- **Bottleneck:** CPU (CSV parsing + type inference), not I/O
- **Scaling:** Linear - 2x file size = 2x processing time
- **Best use case:** Single large files on fast storage (SSD)

### Performance Tips

**Fastest:** Disable type conversion
```bash
csv-converter --input huge.csv --no-type-conversion
# ~10-20% faster
```

**Fast:** Keep specific fields as strings
```bash
csv-converter --input huge.csv --string-fields zipcode,phone
# Reduces type checking overhead
```

**Standard:** Full auto-detection and type inference (default)
```bash
csv-converter --input huge.csv
# Most flexible, still very fast
```

## How It Works

1. **Format Detection**: Samples the first 250 lines (or entire file if smaller) to detect:
   - Delimiter (by counting occurrences and checking consistency)
   - Quote character (typically `"`)
   - Escape method (looks for `\"` vs `""` patterns)
2. **Streaming Processing**: Opens input file with buffered reader and processes one row at a time
3. **Type Inference**: Attempts to parse each field as number or boolean, preserving leading zeros
4. **NDJSON Output**: Each CSV row becomes a single-line JSON object

## Testing

The project includes comprehensive test coverage:

### Run All Tests
```bash
cargo test
```

### Run Specific Test Suites

**Unit tests** (type conversion logic):
```bash
cargo test --lib
```

**Format detection tests**:
```bash
cargo test --test format_detection_tests
```

**Integration tests** (end-to-end conversion):
```bash
cargo test --test integration_tests
```

### Test Coverage

- **10 unit tests** - Type conversion logic (integers, floats, booleans, leading zeros, etc.)
- **15 format detection tests** - Delimiter, quote, and escape character detection
- **18 integration tests** - Full end-to-end conversion with various CSV formats

All tests are located in:
- `src/lib.rs` - Unit tests for core functions
- `tests/format_detection_tests.rs` - Format detection tests
- `tests/integration_tests.rs` - Integration tests

### Todos
 - [x] Add xlsx support
 - [ ] Add multi-threaded processing

## License

This project is open source.
