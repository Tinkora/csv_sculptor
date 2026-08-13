use thiserror::Error;

/// Stable error type for CSV sculpting operations.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("CSV/TSV parsing failed: {0}")]
    ParseError(String),

    #[error("Could not auto-detect delimiter")]
    DelimiterDetectionFailed,

    #[error("Column not found: {0}")]
    ColumnNotFound(String),

    #[error("Invalid filter operator: {0}")]
    InvalidFilterOperator(String),

    #[error("Table is empty")]
    EmptyTable,

    #[error("Export failed: {0}")]
    ExportError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Input exceeds the 10 MiB limit")]
    InputTooLarge,
}

impl CoreError {
    /// Returns a stable machine-readable error code for Web, CLI, and Agent consumers.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::ParseError(_) => "PARSE_ERROR",
            Self::DelimiterDetectionFailed => "DELIMITER_DETECTION_FAILED",
            Self::ColumnNotFound(_) => "COLUMN_NOT_FOUND",
            Self::InvalidFilterOperator(_) => "INVALID_FILTER_OPERATOR",
            Self::EmptyTable => "EMPTY_TABLE",
            Self::ExportError(_) => "EXPORT_ERROR",
            Self::SerializationError(_) => "SERIALIZATION_ERROR",
            Self::InvalidInput(_) => "INVALID_INPUT",
            Self::InputTooLarge => "INPUT_TOO_LARGE",
        }
    }
}
