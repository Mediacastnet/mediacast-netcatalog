//! `mediacast-netcatalog probe` — protocol-capability fingerprint CLI.
//!
//! Synchronous; no async runtime.

use clap::{Parser, Subcommand};
use mediacast_netcatalog::probe::{probe_device, ProbeConfig};
use std::time::Duration;

#[derive(Parser)]
#[command(name = "mediacast-netcatalog", version, about = "Mediacast NetCatalog CLI")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Fingerprint a device's protocol capabilities.
    Probe {
        /// Target host (DNS or IP).
        #[arg(long)]
        host: String,
        /// Vendor identifier (cisco_ios, aruba_aoscx, ...).
        #[arg(long)]
        vendor: String,
        /// Per-protocol connect + read timeout in seconds.
        #[arg(long, default_value_t = 5)]
        timeout: u64,
        /// Skip individual probes (comma-separated; recognized: ssh,
        /// netconf, gnmi, restconf).
        #[arg(long, value_delimiter = ',')]
        skip: Vec<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Probe { host, vendor, timeout, skip } => {
            let cfg = ProbeConfig {
                timeout: Duration::from_secs(timeout),
                skip,
            };
            let report = probe_device(&host, &vendor, &cfg)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
    Ok(())
}
