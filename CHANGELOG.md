# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] — 2026-04-29

### Added

- Initial release. Repository scaffold + research-grade catalog data layer.
- Catalog YAML for seven vendors:
  - Cisco IOS / IOS-XE (`catalog/cisco-ios-xe.yaml`)
  - Cisco NX-OS (`catalog/cisco-nxos.yaml`)
  - Aruba AOS-CX (`catalog/aruba-aoscx.yaml`)
  - Juniper Junos (`catalog/juniper-junos.yaml`)
  - Arista EOS (`catalog/arista-eos.yaml`)
  - HPE / Aruba ProCurve (`catalog/hpe-procurve.yaml`)
  - Cisco Meraki MS (`catalog/meraki-mx-ms.yaml`)
- 25 abstract command types covered per vendor (see `catalog/COMMAND_TYPES.md`).
- Schema documented in `catalog/SCHEMA.md` with extensions for eAPI,
  vendor-proprietary REST, SNMP, and cloud Dashboard API.
- Cross-vendor synthesis in `catalog/FINDINGS.md`.
- Rust core scaffold:
  - `Catalog::load_bundled` + `Catalog::load_dir`
  - `FirmwareVersion` parser handling Aruba family prefixes (`FL.`, `GL.`,
    etc.), Cisco-style parens (`9.3(5)`), Junos suffixes (`R3-S2.4`)
  - `VersionRange` with `>=`, `<`, `,` (AND), `||` (OR), `*` (wildcard)
    and most-specific-wins matching
- PyO3 bindings under `mediacast_netcatalog._native` (feature `python`).
- Stub `mediacast-netcatalog probe` CLI binary (feature `bin`).
- CI: cargo fmt + clippy + test on stable/MSRV across Linux/macOS/Windows;
  maturin wheel build + smoke test on Python 3.9 + 3.12; yamllint on catalog.

### Provenance

- Catalog data produced by [Mediacast NetCaster](https://github.com/Mediacastnet)'s
  2026-04 vendor doc-crawl effort: seven parallel research agents over vendor
  command references, YANG model repos, and community sources. Every entry
  cites its source in the `sources:` block.

### Known limitations

- Probe implementation is a stub. v0.2 wires up NETCONF/gNMI/RESTCONF/SSH
  fingerprints.
- API is unstable. Expect breaking changes before v0.2.
- Catalog entries marked `unverified: true` are heuristic and want validation
  against real gear.

[Unreleased]: https://github.com/Mediacastnet/mediacast-netcatalog/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Mediacastnet/mediacast-netcatalog/releases/tag/v0.1.0
