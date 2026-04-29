//! # mediacast-netcatalog
//!
//! Vendor command catalog + version matcher + protocol probe for
//! multi-vendor network automation.
//!
//! See the [README](https://github.com/Mediacastnet/mediacast-netcatalog)
//! for an overview and the `catalog/` directory for the YAML data files.
//!
//! ## Status
//!
//! v0.1 — scaffold. The catalog YAML is research-grade; the Rust API
//! is unstable until v0.2.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod catalog;
pub mod command_types;
pub mod error;
pub mod version;

#[cfg(feature = "bin")]
pub mod probe;

#[cfg(feature = "python")]
mod python;

pub use catalog::{Catalog, CommandEntry, ProtocolAlternatives, VendorFile};
pub use command_types::CommandType;
pub use error::{Error, Result};
pub use version::{FirmwareVersion, VersionRange};
