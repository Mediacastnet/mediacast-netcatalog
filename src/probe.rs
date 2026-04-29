//! Protocol-capability probe.
//!
//! Fingerprints which programmatic interfaces a real device exposes —
//! NETCONF (TCP/830), gNMI (TCP/9339), RESTCONF (HTTPS/443), SSH banner.
//!
//! Stdlib + `tokio` for async TCP. No Netmiko, no Paramiko.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;

/// One device's probe report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProbeReport {
    /// Target host (DNS or IP).
    pub host: String,
    /// Vendor identifier the probe was run with.
    pub vendor: String,
    /// True if NETCONF/830 accepted a TCP connection + sent a hello.
    pub netconf_available: bool,
    /// True if gNMI/9339 accepted a TCP connection.
    pub gnmi_available: bool,
    /// True if RESTCONF/443 returned a `/restconf` discovery response.
    pub restconf_available: bool,
    /// SSH banner string, if reachable.
    pub ssh_banner: Option<String>,
    /// Parsed firmware version, if extractable from any probe response.
    pub firmware: Option<String>,
}

/// Per-probe configuration.
#[derive(Debug, Clone)]
pub struct ProbeConfig {
    /// Connect timeout per protocol.
    pub timeout: Duration,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self { timeout: Duration::from_secs(5) }
    }
}

/// Run all enabled probes against a target. Implementation lands in v0.2 —
/// this signature is the stable contract.
#[cfg(feature = "bin")]
pub async fn probe_device(_host: &str, _vendor: &str, _cfg: &ProbeConfig) -> Result<ProbeReport> {
    todo!("probe implementation lands in v0.2")
}

/// Convenience helper.
#[allow(dead_code)]
fn parse_endpoint(host: &str, port: u16) -> Option<SocketAddr> {
    use std::net::ToSocketAddrs;
    (host, port).to_socket_addrs().ok()?.next()
}
