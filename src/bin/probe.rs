//! `mediacast-netcatalog probe` — protocol-capability fingerprint CLI.
//!
//! v0.1: stub. v0.2 wires up the actual probes from `crate::probe`.

use clap::{Parser, Subcommand};
use mediacast_netcatalog::probe::{probe_device, ProbeConfig};

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
        /// Per-protocol connect timeout in seconds.
        #[arg(long, default_value_t = 5)]
        timeout: u64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Probe { host, vendor, timeout } => {
            let cfg = ProbeConfig { timeout: std::time::Duration::from_secs(timeout) };
            let report = probe_device(&host, &vendor, &cfg).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
    }
    Ok(())
}
