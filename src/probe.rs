//! Protocol-capability probe — stdlib-only TCP fingerprinting.
//!
//! Fingerprints which programmatic interfaces a network device exposes:
//!
//! - **NETCONF** — TCP/830 + SSH banner read. NETCONF runs as an SSH
//!   subsystem; an `SSH-2.0-...` banner on this port is the strongest
//!   probe-time signal available without authenticating.
//! - **gNMI** — TCP/9339 connect. gRPC-over-TLS; the port is reserved
//!   for gNMI by IANA, so a successful TCP connect is sufficient signal
//!   for v0.2. v0.3 may add a proper TLS handshake check.
//! - **RESTCONF** — TCP/443 connect. **Necessary but not sufficient**:
//!   a switch's regular HTTPS management UI also lives on 443, so this
//!   probe means "RESTCONF is *possible* if also configured" rather
//!   than "RESTCONF is enabled." v0.3 adds a proper HTTPS GET to
//!   `/restconf` (with self-signed-cert tolerance) for authoritative
//!   detection.
//! - **SSH banner** — TCP/22, read first line up to `\r\n`. Captured
//!   raw for downstream parsing (vendor + sometimes firmware hints).
//!
//! All probes are **synchronous and stdlib-only** — no async runtime, no
//! TLS dependency, no HTTP client. PyO3 bindings release the GIL during
//! the blocking I/O. Total wheel-size impact is near-zero.
//!
//! Each probe is independent and best-effort. Per-protocol failures land
//! in [`ProbeReport::diagnostics`]; the report itself never errors at
//! the protocol level.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::{Duration, Instant};

/// Default per-probe connect + read timeout.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Standard NETCONF port.
pub const NETCONF_PORT: u16 = 830;
/// Standard gNMI port (IANA-reserved).
pub const GNMI_PORT: u16 = 9339;
/// Standard HTTPS port — RESTCONF lives here when enabled.
pub const RESTCONF_PORT: u16 = 443;
/// Standard SSH port.
pub const SSH_PORT: u16 = 22;

/// Per-probe configuration.
#[derive(Debug, Clone)]
pub struct ProbeConfig {
    /// Connect + read timeout per protocol.
    pub timeout: Duration,
    /// Skip individual probes by name (case-insensitive). Useful for
    /// limiting blast radius on production gear (e.g., skip RESTCONF
    /// if the operator knows it triggers an audit log entry on every
    /// request).
    ///
    /// Recognized names: `"ssh"`, `"netconf"`, `"gnmi"`, `"restconf"`.
    pub skip: Vec<String>,
}

impl Default for ProbeConfig {
    fn default() -> Self {
        Self {
            timeout: DEFAULT_TIMEOUT,
            skip: Vec::new(),
        }
    }
}

impl ProbeConfig {
    fn skipped(&self, name: &str) -> bool {
        self.skip.iter().any(|s| s.eq_ignore_ascii_case(name))
    }
}

/// One device's probe report. Each protocol field is `Some(true)` /
/// `Some(false)` / `None`:
/// - `Some(true)` — probe responded affirmatively.
/// - `Some(false)` — probe attempted but negative (port reachable but
///   protocol-specific check failed, or port unreachable).
/// - `None` — probe was skipped via [`ProbeConfig::skip`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProbeReport {
    /// Target host the probe was run against.
    pub host: String,
    /// Vendor identifier the probe was run with. Operator-supplied.
    /// v0.2 doesn't use this for routing; future versions will sequence
    /// per-vendor probes.
    pub vendor: String,
    /// NETCONF/830 detected (SSH banner present).
    pub netconf_available: Option<bool>,
    /// gNMI/9339 detected (TCP port open).
    pub gnmi_available: Option<bool>,
    /// HTTPS/443 reachable. **Caveat**: also true for vendor management
    /// web UIs that aren't RESTCONF. See module docs.
    pub restconf_available: Option<bool>,
    /// Raw SSH banner string from port 22, if reachable. May contain
    /// vendor + firmware hints; v0.2 leaves parsing to the consumer.
    pub ssh_banner: Option<String>,
    /// Best-effort firmware version. v0.2 always returns `None` —
    /// authoritative extraction needs an authenticated `show version`
    /// round-trip, which the probe deliberately does not do.
    pub firmware: Option<String>,
    /// Per-protocol diagnostics so operators can debug a "why didn't
    /// this detect" question without re-running with verbose flags.
    pub diagnostics: Vec<String>,
    /// Total wall-clock time for all probes, in milliseconds.
    pub elapsed_ms: u128,
}

/// Run all enabled probes against `host`. Returns a [`ProbeReport`]
/// summarizing what was found.
///
/// This function is infallible at the protocol level — per-probe
/// failures land in [`ProbeReport::diagnostics`]. The `Result` is
/// reserved for future error variants (e.g., catalog-driven probe
/// sequencing failures); v0.2 always returns `Ok`.
pub fn probe_device(host: &str, vendor: &str, cfg: &ProbeConfig) -> Result<ProbeReport> {
    let start = Instant::now();
    let mut report = ProbeReport {
        host: host.to_owned(),
        vendor: vendor.to_owned(),
        ..Default::default()
    };

    if !cfg.skipped("ssh") {
        match probe_ssh_banner(host, SSH_PORT, cfg.timeout) {
            Ok(Some(banner)) => report.ssh_banner = Some(banner),
            Ok(None) => report.diagnostics.push(format!("ssh:{SSH_PORT} reachable but no banner read")),
            Err(e) => report.diagnostics.push(format!("ssh:{SSH_PORT} {e}")),
        }
    }

    if !cfg.skipped("netconf") {
        report.netconf_available = Some(match probe_ssh_banner(host, NETCONF_PORT, cfg.timeout) {
            Ok(Some(_)) => true,
            Ok(None) => {
                report.diagnostics.push(format!("netconf:{NETCONF_PORT} reachable but no SSH banner"));
                false
            }
            Err(e) => {
                report.diagnostics.push(format!("netconf:{NETCONF_PORT} {e}"));
                false
            }
        });
    }

    if !cfg.skipped("gnmi") {
        report.gnmi_available = Some(match probe_tcp_open(host, GNMI_PORT, cfg.timeout) {
            Ok(()) => true,
            Err(e) => {
                report.diagnostics.push(format!("gnmi:{GNMI_PORT} {e}"));
                false
            }
        });
    }

    if !cfg.skipped("restconf") {
        report.restconf_available = Some(match probe_tcp_open(host, RESTCONF_PORT, cfg.timeout) {
            Ok(()) => true,
            Err(e) => {
                report.diagnostics.push(format!("restconf:{RESTCONF_PORT} {e}"));
                false
            }
        });
    }

    report.elapsed_ms = start.elapsed().as_millis();
    Ok(report)
}

// ── Individual probes ────────────────────────────────────────────────

/// Connect to `host:port` with a deadline. Returns the established
/// stream with read/write timeouts already applied.
fn connect(host: &str, port: u16, timeout: Duration) -> std::io::Result<TcpStream> {
    let addrs: Vec<_> = (host, port)
        .to_socket_addrs()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::AddrNotAvailable, format!("DNS: {e}")))?
        .collect();
    if addrs.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::AddrNotAvailable,
            "no addresses resolved",
        ));
    }
    let mut last_err = std::io::Error::other("no addresses tried");
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(stream) => {
                stream.set_read_timeout(Some(timeout))?;
                stream.set_write_timeout(Some(timeout))?;
                return Ok(stream);
            }
            Err(e) => last_err = e,
        }
    }
    Err(last_err)
}

/// Returns `Ok(())` if the port accepts a TCP connection.
fn probe_tcp_open(host: &str, port: u16, timeout: Duration) -> std::io::Result<()> {
    let _stream = connect(host, port, timeout)?;
    Ok(())
}

/// Read the first line of a banner from `host:port` (up to `\r\n` or
/// 255 bytes). Returns `Some(line)` if it starts with `SSH-`, `None` if
/// the port was reachable but didn't emit an SSH-shaped banner.
fn probe_ssh_banner(host: &str, port: u16, timeout: Duration) -> std::io::Result<Option<String>> {
    let mut stream = connect(host, port, timeout)?;
    let mut buf = [0u8; 256];
    let mut total = 0;
    while total < buf.len() {
        let n = match stream.read(&mut buf[total..]) {
            Ok(0) => break,
            Ok(n) => n,
            Err(ref e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        total += n;
        if buf[..total].contains(&b'\n') {
            break;
        }
    }
    if total == 0 {
        return Ok(None);
    }
    let line: String = buf[..total]
        .iter()
        .take_while(|&&b| b != b'\n' && b != b'\r')
        .map(|&b| b as char)
        .collect();
    if line.starts_with("SSH-") {
        Ok(Some(line))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_skip_is_case_insensitive() {
        let cfg = ProbeConfig { skip: vec!["SSH".into(), "Netconf".into()], ..Default::default() };
        assert!(cfg.skipped("ssh"));
        assert!(cfg.skipped("netconf"));
        assert!(!cfg.skipped("gnmi"));
    }

    #[test]
    fn report_default_has_no_results() {
        let r = ProbeReport::default();
        assert!(r.netconf_available.is_none());
        assert!(r.gnmi_available.is_none());
        assert!(r.restconf_available.is_none());
        assert!(r.ssh_banner.is_none());
        assert_eq!(r.elapsed_ms, 0);
    }

    #[test]
    fn probe_unreachable_host_gracefully_reports() {
        // 198.51.100.x is RFC 5737 TEST-NET-2; guaranteed-unreachable.
        let cfg = ProbeConfig {
            timeout: Duration::from_millis(250),
            ..Default::default()
        };
        let report = probe_device("198.51.100.1", "cisco_ios", &cfg).expect("probe never errors at lib level");
        assert_eq!(report.host, "198.51.100.1");
        assert_eq!(report.vendor, "cisco_ios");
        // Every probe should resolve to Some(false) or None — never Some(true) for an unreachable host.
        assert!(report.netconf_available != Some(true));
        assert!(report.gnmi_available != Some(true));
        assert!(report.restconf_available != Some(true));
        assert!(report.ssh_banner.is_none());
        // Diagnostics should be populated for the unreachable probes.
        assert!(!report.diagnostics.is_empty(), "expected at least one diagnostic for unreachable host");
    }

    #[test]
    fn probe_respects_skip_list() {
        let cfg = ProbeConfig {
            timeout: Duration::from_millis(250),
            skip: vec!["ssh".into(), "netconf".into(), "gnmi".into(), "restconf".into()],
        };
        let report = probe_device("198.51.100.1", "cisco_ios", &cfg).unwrap();
        // All probes skipped → all None.
        assert_eq!(report.netconf_available, None);
        assert_eq!(report.gnmi_available, None);
        assert_eq!(report.restconf_available, None);
        assert_eq!(report.ssh_banner, None);
        assert!(report.diagnostics.is_empty(), "no probes ran, no diagnostics expected");
    }
}
