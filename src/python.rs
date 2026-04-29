//! PyO3 bindings — exposed as the `mediacast_netcatalog._native` extension.
//!
//! The Python-side `mediacast_netcatalog/` package re-exports these into a
//! more idiomatic surface; this file is the thin FFI seam.

use crate::catalog::Catalog as RustCatalog;
use crate::command_types::CommandType;
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;

/// Python-facing catalog handle.
#[pyclass(name = "Catalog", module = "mediacast_netcatalog._native")]
struct PyCatalog {
    inner: RustCatalog,
}

#[pymethods]
impl PyCatalog {
    /// Load the bundled catalog files.
    #[staticmethod]
    fn load_bundled() -> PyResult<Self> {
        RustCatalog::load_bundled()
            .map(|c| PyCatalog { inner: c })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// Load every `*.yaml` file in `path`.
    #[staticmethod]
    fn load_dir(path: &str) -> PyResult<Self> {
        RustCatalog::load_dir(path)
            .map(|c| PyCatalog { inner: c })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    /// All vendor identifiers.
    fn vendors(&self) -> Vec<String> {
        self.inner.vendors().map(|s| s.to_owned()).collect()
    }

    /// Look up `(vendor, firmware, command)`. Returns the CLI string or
    /// raises ValueError if the entry is missing / NOT_SUPPORTED.
    fn cli(&self, vendor: &str, firmware: &str, command: &str) -> PyResult<String> {
        let cmd = parse_command_type(command)?;
        match self.inner.lookup(vendor, firmware, cmd) {
            Ok(Some(entry)) => Ok(entry.cli.clone()),
            Ok(None) => Err(PyKeyError::new_err(format!("no entry for {} {:?}", vendor, cmd))),
            Err(e) => Err(PyValueError::new_err(e.to_string())),
        }
    }
}

fn parse_command_type(s: &str) -> PyResult<CommandType> {
    let v: serde_yaml::Value = serde_yaml::Value::String(s.to_owned());
    serde_yaml::from_value(v).map_err(|_| PyValueError::new_err(format!("unknown CommandType: {}", s)))
}

/// Module entry point — `import mediacast_netcatalog._native`.
#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCatalog>()?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
