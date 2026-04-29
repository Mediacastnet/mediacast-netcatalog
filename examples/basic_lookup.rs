//! Minimal usage example. Runs against the bundled catalog.
//!
//! ```bash
//! cargo run --example basic_lookup
//! ```

use mediacast_netcatalog::{Catalog, CommandType};

fn main() -> anyhow::Result<()> {
    let catalog = Catalog::load_bundled()?;

    println!("Vendors loaded:");
    for v in catalog.vendors() {
        println!("  - {}", v);
    }
    println!();

    let cases = [
        ("cisco_ios", "17.6.4", CommandType::ArpTable),
        ("cisco_nxos", "9.3.10", CommandType::MacTable),
        ("aruba_aoscx", "FL.10.13.1000", CommandType::VlanList),
        ("juniper_junos", "21.4R3", CommandType::SaveConfig),
        ("arista_eos", "4.30.0F", CommandType::PortShutdown),
    ];

    for (vendor, fw, cmd) in cases {
        match catalog.lookup(vendor, fw, cmd) {
            Ok(Some(entry)) => println!("{:>14} {:>16} {:?} → {}", vendor, fw, cmd, entry.cli),
            Ok(None) => println!("{:>14} {:>16} {:?} → (no entry)", vendor, fw, cmd),
            Err(e) => println!("{:>14} {:>16} {:?} → ERR: {}", vendor, fw, cmd, e),
        }
    }

    Ok(())
}
