# mediacast-netcatalog

**Vendor command catalog + version matcher + protocol probe for multi-vendor
network automation.** Rust core; Python bindings via PyO3.

[![Crates.io](https://img.shields.io/crates/v/mediacast-netcatalog.svg)](https://crates.io/crates/mediacast-netcatalog)
[![PyPI](https://img.shields.io/pypi/v/mediacast-netcatalog.svg)](https://pypi.org/project/mediacast-netcatalog/)
[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#license)

## What this is

A YAML-backed library that maps **abstract command types** (e.g.
`ARP_TABLE`, `MAC_TABLE`, `PORT_VLAN_ASSIGN`) to **concrete CLI strings,
protocol alternatives (NETCONF / RESTCONF / gNMI / vendor-specific), and
parser hints** — selectable by `(vendor, firmware version)`.

It comes seeded with research-grade catalogs for seven switch platforms:

| Vendor             | Coverage              | Notes                                                              |
|--------------------|-----------------------|--------------------------------------------------------------------|
| Cisco IOS / IOS-XE | 25/25                 | 12.x / 15.x / 17.x output drift captured                           |
| Cisco NX-OS        | 25/25                 | 7+ load-bearing divergences from IOS documented                    |
| Aruba AOS-CX       | 25/25                 | Family-prefixed firmware (FL./GL./LL.) version matcher needed      |
| Juniper Junos      | 25/25                 | Transactional `configure → set → commit` model                     |
| Arista EOS         | 24/25 (CIVIC absent)  | eAPI extension; mostly IOS-compatible *but* — see FINDINGS         |
| HPE ProCurve       | 24/25 (CIVIC absent)  | No NETCONF/RESTCONF/gNMI ever; SNMP + proprietary REST only        |
| Cisco Meraki MS    | 23/25 + 2 NOT_SUPP    | Cloud-managed; Dashboard API, not CLI — see FINDINGS strategic flag |

See [`catalog/FINDINGS.md`](catalog/FINDINGS.md) for cross-vendor synthesis,
[`catalog/SCHEMA.md`](catalog/SCHEMA.md) for the YAML format, and
[`catalog/COMMAND_TYPES.md`](catalog/COMMAND_TYPES.md) for the abstract command
vocabulary.

## Why

Most multi-vendor tooling hard-codes per-vendor command strings inside
per-vendor handler classes, with no awareness of firmware-version drift.
That works until:

- Cisco IOS-XE 12.x emits a 5-column ARP table and 17.x emits 6 — your parser
  silently truncates.
- NX-OS uses `ip dhcp relay address` not `ip helper-address` — your IOS
  parser ignores DHCP servers entirely on Nexus.
- Aruba AOS-CX firmware strings look like `FL.10.13.1000` — your SemVer
  comparator throws.
- Arista EOS defaults STP to MSTP and L2 MTU to 9214, not PVST+/1500 — your
  drift detector flags every port as misconfigured.

This library is the **data layer** that lets a runtime pick the right
command for the right `(vendor, firmware)` instead of guessing. It also
ships a **protocol-capability probe** (CLI tool + library) that fingerprints
which programmatic interfaces a real device actually exposes.

## Status

**v0.1 — research / scaffold.** The catalog YAML is research-grade and
already drives planning inside [Mediacast NetCaster](https://github.com/Mediacastnet).
The Rust loader, version matcher, and PyO3 bindings are under active
development. **API is unstable until v0.2.**

## Quick start (Rust)

```toml
[dependencies]
mediacast-netcatalog = "0.1"
```

```rust
use mediacast_netcatalog::{Catalog, CommandType};

fn main() -> anyhow::Result<()> {
    let catalog = Catalog::load_bundled()?;            // ships embedded YAML
    let entry = catalog
        .lookup("cisco_ios", "17.6.4", CommandType::ArpTable)?
        .expect("arp table is universal on cisco_ios");

    println!("CLI: {}", entry.cli);
    if let Some(gnmi) = &entry.protocol_alternatives.gnmi {
        println!("gNMI path: {}", gnmi.path);
    }
    Ok(())
}
```

## Quick start (Python)

```bash
pip install mediacast-netcatalog
```

```python
from mediacast_netcatalog import Catalog, CommandType

catalog = Catalog.load_bundled()
entry = catalog.lookup("aruba_aoscx", "FL.10.13.1000", CommandType.ARP_TABLE)
print(entry.cli)
print(entry.protocol_alternatives.rest_api)   # AOS-CX has proprietary REST
```

The Python bindings are zero-copy where possible and re-export the same
type vocabulary as the Rust crate.

## Protocol probe

```bash
cargo install mediacast-netcatalog --features bin
mediacast-netcatalog probe --host 10.0.0.1 --vendor cisco_ios
```

Or from Python:

```python
from mediacast_netcatalog.probe import probe_device
report = probe_device(host="10.0.0.1", vendor="cisco_ios")
print(report.netconf_available, report.gnmi_available, report.firmware)
```

The probe uses **stdlib-only Rust** (no Netmiko, no Paramiko) — it issues
TCP connects + minimal protocol handshakes for NETCONF (830), gNMI (9339),
RESTCONF (443/HTTPS), and the vendor's text CLI banner.

## Catalog as data

If you don't want a Rust dependency, just consume the YAML directly:

```bash
git clone https://github.com/Mediacastnet/mediacast-netcatalog
cd mediacast-netcatalog/catalog
ls *.yaml
```

The schema is documented in [`catalog/SCHEMA.md`](catalog/SCHEMA.md). Files
are pure YAML — load them with whatever tool you prefer.

## Project layout

```
mediacast-netcatalog/
├── catalog/                   # Canonical YAML data + research docs
│   ├── cisco-ios-xe.yaml
│   ├── cisco-nxos.yaml
│   ├── aruba-aoscx.yaml
│   ├── juniper-junos.yaml
│   ├── arista-eos.yaml
│   ├── hpe-procurve.yaml
│   ├── meraki-mx-ms.yaml
│   ├── SCHEMA.md
│   ├── COMMAND_TYPES.md
│   ├── STATUS.md
│   └── FINDINGS.md
├── src/                       # Rust core
│   ├── lib.rs
│   ├── catalog.rs             # YAML → typed Catalog
│   ├── version.rs             # Version range matcher (handles AOS-CX FL./GL.)
│   ├── command_types.rs       # CommandType enum
│   ├── error.rs
│   ├── probe.rs               # Protocol-capability probe
│   └── python.rs              # PyO3 bindings (feature = "python")
├── examples/
│   └── basic_lookup.rs
├── tests/
│   ├── catalog_load.rs
│   └── version_matcher.rs
├── pyproject.toml             # maturin build config
├── Cargo.toml
├── CHANGELOG.md
├── CONTRIBUTING.md
├── LICENSE-MIT
└── LICENSE-APACHE
```

## Contributing

Contributions welcome — see [`CONTRIBUTING.md`](CONTRIBUTING.md). High-value
contributions:

- **New vendor catalogs** (Extreme, FortiSwitch, MikroTik, Brocade/RUCKUS).
  See `catalog/SCHEMA.md`; aim for ≥80% coverage of the abstract command
  set with citation links.
- **Firmware-version drift entries** for vendors already covered. If you
  hit an output-format change between firmware revisions, file a PR with
  a `versions:` block + sample output.
- **Probe protocol additions** — currently NETCONF/gNMI/RESTCONF/SSH-banner;
  IPMI, ONIE, and SONiC discovery would round it out.

## Related projects

- **[NetCaster](https://github.com/Mediacastnet/netcaster)** — venue-centric
  network management product (Mediacast Network Solutions). First production
  consumer of this crate.
- **[Mediacast Platform](https://github.com/Mediacastnet)** — broader
  Rust-first platform from the same org.

## License

Dual-licensed under either of:

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE))
- MIT license ([`LICENSE-MIT`](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual-licensed as above, without any additional terms or
conditions.

## Provenance

Catalog data was produced by [Mediacast NetCaster](https://github.com/Mediacastnet)'s
2026-04 vendor doc-crawl effort: seven parallel research agents working
from vendor command references, YANG model repos, and community sources.
Every entry in `catalog/<vendor>.yaml` cites its source; entries marked
`unverified: true` are heuristic and want validation against real gear.
See `catalog/FINDINGS.md` §7 for the doc-access friction encountered and
fallback sources used.
