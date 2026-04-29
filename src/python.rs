//! PyO3 bindings — exposed as the `mediacast_netcatalog._native` extension.
//!
//! The Python-side `mediacast_netcatalog/` package re-exports these into a
//! more idiomatic surface; this file is the thin FFI seam.

use crate::catalog::Catalog as RustCatalog;
use crate::command_types::CommandType;
use crate::probe::{probe_device as probe_device_rs, ProbeConfig, ProbeReport};
use pyo3::exceptions::{PyKeyError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::time::Duration;

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

/// Run the protocol-capability probe against `host`. Returns a dict
/// mirroring [`crate::probe::ProbeReport`].
///
/// The probe is synchronous; PyO3 releases the GIL during the blocking
/// I/O via `py.allow_threads`, so this call doesn't stall other Python
/// threads.
#[pyfunction]
#[pyo3(signature = (host, vendor, timeout_seconds=5.0, skip=Vec::new()))]
fn probe_device(
    py: Python<'_>,
    host: &str,
    vendor: &str,
    timeout_seconds: f64,
    skip: Vec<String>,
) -> PyResult<PyObject> {
    let cfg = ProbeConfig {
        timeout: Duration::from_secs_f64(timeout_seconds.max(0.001)),
        skip,
    };
    let report = py
        .allow_threads(|| probe_device_rs(host, vendor, &cfg))
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
    report_to_pydict(py, &report)
}

fn report_to_pydict(py: Python<'_>, r: &ProbeReport) -> PyResult<PyObject> {
    let d = PyDict::new_bound(py);
    d.set_item("host", &r.host)?;
    d.set_item("vendor", &r.vendor)?;
    d.set_item("netconf_available", r.netconf_available)?;
    d.set_item("gnmi_available", r.gnmi_available)?;
    d.set_item("restconf_available", r.restconf_available)?;
    d.set_item("ssh_banner", r.ssh_banner.as_deref())?;
    d.set_item("firmware", r.firmware.as_deref())?;
    d.set_item("diagnostics", r.diagnostics.clone())?;
    d.set_item("elapsed_ms", r.elapsed_ms)?;
    Ok(d.into())
}

/// Module entry point — `import mediacast_netcatalog._native`.
#[pymodule]
fn _native(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyCatalog>()?;
    m.add_function(wrap_pyfunction!(probe_device, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
