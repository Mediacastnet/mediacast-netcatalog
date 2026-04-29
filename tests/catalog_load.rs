//! Smoke tests: every bundled vendor file loads, every vendor covers
//! every abstract command type.

use mediacast_netcatalog::{Catalog, CommandType};

#[test]
fn bundled_catalog_loads() {
    let catalog = Catalog::load_bundled().expect("bundled catalog must load");
    let vendors: Vec<&str> = catalog.vendors().collect();
    assert!(!vendors.is_empty(), "bundled catalog has at least one vendor");

    let expected = [
        "cisco_ios", "cisco_nxos", "aruba_aoscx", "juniper_junos",
        "arista_eos", "hpe_procurve", "meraki_ms",
    ];
    for v in expected {
        assert!(
            vendors.contains(&v),
            "expected vendor '{}' present (have {:?})", v, vendors,
        );
    }
}

#[test]
fn every_vendor_covers_every_command_type() {
    let catalog = Catalog::load_bundled().expect("bundled catalog must load");
    for vendor_id in catalog.vendors() {
        let vf = catalog.vendor(vendor_id).unwrap();
        for &cmd in CommandType::all() {
            assert!(
                vf.commands.iter().any(|c| c.command_type == cmd),
                "vendor '{}' is missing command type {:?}", vendor_id, cmd,
            );
        }
    }
}
