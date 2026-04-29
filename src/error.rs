//! Error types for catalog loading, version parsing, and probe operations.

use thiserror::Error;

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors this crate can return.
#[derive(Debug, Error)]
pub enum Error {
    /// Catalog YAML failed to parse or didn't match the schema.
    #[error("catalog parse error in {file}: {source}")]
    CatalogParse {
        /// Source filename (vendor catalog) where the parse failed.
        file: String,
        /// Underlying serde_yaml error.
        #[source]
        source: serde_yaml::Error,
    },

    /// I/O error reading a catalog file.
    #[error("catalog I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Requested vendor not present in the catalog.
    #[error("unknown vendor: {0}")]
    UnknownVendor(String),

    /// Version range expression failed to parse.
    #[error("invalid version range '{expr}': {reason}")]
    BadVersionRange {
        /// The unparseable expression.
        expr: String,
        /// Why it failed.
        reason: String,
    },

    /// Firmware version string couldn't be parsed.
    #[error("invalid firmware version '{0}'")]
    BadFirmwareVersion(String),

    /// No catalog entry matched the (vendor, firmware) combo for a command type.
    #[error("no entry for {vendor} {firmware:?} {command:?}")]
    NoMatchingEntry {
        /// Vendor identifier.
        vendor: String,
        /// Firmware string the lookup was attempted with.
        firmware: Option<String>,
        /// Abstract command type that failed to resolve.
        command: crate::CommandType,
    },

    /// The requested command type is explicitly NOT_SUPPORTED on this vendor.
    #[error("{command:?} is not supported on {vendor}: {reason}")]
    NotSupported {
        /// Vendor where this command isn't supported.
        vendor: String,
        /// Abstract command type.
        command: crate::CommandType,
        /// Reason from the catalog's `notes` field.
        reason: String,
    },
}
