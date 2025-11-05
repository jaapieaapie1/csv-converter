pub mod csv;
pub mod xlsx;

use anyhow::Result;
use std::path::Path;

/// Common trait for all file parsers
pub trait Parser {
    /// Convert the input file to NDJSON format
    fn convert_to_ndjson(
        &self,
        input_path: &Path,
        output_path: Option<&Path>,
        no_type_conversion: bool,
        string_fields: &[String],
    ) -> Result<()>;
}
