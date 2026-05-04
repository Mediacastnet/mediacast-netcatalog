# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0-dev] — 2026-05-04

### Added

- **RESTCONF content discrimination** — closes the documented v0.2
  limitation where `restconf_available: true` only meant "TCP/443 is
  open" (which also matched any vendor management UI). v0.3 issues an
  HTTPS GET to `/restconf` with self-signed-cert tolerance and parses
  the response status:
  - `200 OK` / `401 Unauthorized` / `403 Forbidden` / `5xx` → `Enabled`
  - `404 Not Found` / other 4xx → `NotEnabled`
  - TCP-open but TLS or HTTP fails → falls back to v0.2's port-open
    semantic (still positive, but flagged in diagnostics)
- Diagnostic messages now include the actual HTTP status code so
  operators can distinguish between "auth required" (401), "not
  configured" (404), and "vendor portal" (often 200 to a management
  page rather than a RESTCONF response).

### Changed

- Probe is no longer "stdlib-only" — `rustls` (with the ring crypto
  provider) is now a hard dependency for the RESTCONF HTTPS handshake.
  The other three probes (NETCONF, gNMI port, SSH banner) still use
  stdlib TCP only.
- Module docstring updated; the v0.2 "stdlib-only" claim removed.

### Known limitations carried into v0.4

- gNMI is still TCP-port-open. A proper TLS handshake check on 9339
  (without a full gRPC client) is small additional work and lands in
  the next release.
- The probe's HTTP parsing reads only the first 256 bytes and extracts
  the status line. Sufficient for classification; doesn't attempt to
  parse the body or the full headers. If a vendor's HTTPS server takes
  >256 bytes for the status line, the probe falls back to
  `PortOpenTlsFailed`.

### Security note

The HTTPS probe explicitly accepts any server certificate
(`make_insecure_client_config` + `AcceptAnyCert`) because network gear
universally uses self-signed certs. **This applies only to the probe
path**; NetCaster's authenticated device I/O is via Netmiko/SSH, which
uses its own credential and host-key verification. The lax verifier
does not leak into production credential paths.

## [0.2.0] — 2026-04-29

### Added

- **Protocol-capability probe — real implementation.** `probe_device()`
  now actually probes the target (v0.1 was a `todo!()` stub). Stdlib-only
  TCP fingerprinting:
  - **SSH/22** — connects, reads first line, captures `SSH-2.0-...`
    banner string raw (vendor + sometimes firmware hints).
  - **NETCONF/830** — same banner read; presence of an `SSH-` prefix on
    this port is the strongest probe-time signal without authenticating.
  - **gNMI/9339** — TCP connect-only (IANA-reserved port).
  - **HTTPS/443** — TCP connect-only. **Reaches RESTCONF when present
    but also reaches the regular vendor management UI**; v0.3 plans a
    proper HTTPS GET to `/restconf` with self-signed-cert tolerance for
    authoritative discrimination.
  - All probes synchronous; PyO3 releases the GIL during blocking I/O.
  - No async runtime, no TLS dep, no HTTP-client dep — wheel-size impact
    is near-zero.
- **`mediacast_netcatalog.probe_device(...)` Python binding.** Returns
  a dict with `netconf_available` / `gnmi_available` / `restconf_available`
  / `ssh_banner` / `diagnostics` / `elapsed_ms`.
- **`ProbeConfig::skip` list** to skip individual probes by name (case-
  insensitive). Limits probe blast-radius on production gear that audits
  every connection attempt.
- **CLI: `mediacast-netcatalog probe --skip ssh,netconf` flag.**
- **EapiMapping flexibility**: now accepts both `cli: "show ip arp"`
  (single string, what the catalog YAML actually uses) and
  `commands: [...]` (array, for multi-command eAPI batches). New
  `EapiMapping::command_list()` helper normalizes both forms.

### Fixed

- `catalog/cisco-ios-xe.yaml`: MAC_TABLE sample_output had inconsistent
  indentation that broke the YAML literal block — re-indented to match
  surrounding entries. All seven bundled catalog files now parse cleanly
  via `serde_yaml`.
- Clippy lint cleanup (`io_other_error`, `manual_contains`,
  `unnecessary_map_or` — the catalog version-matcher's `map_or(true,
  ...)` is now `is_none_or(...)`).
- **`.github/workflows/ci.yml`**: yamllint step's `run:` value was a
  plain YAML scalar containing `{`, which the GitHub Actions validator
  rejected at dispatch time (every push completed in 0s with 0 jobs and
  no logs). Converted to a `|` block scalar.
- **`.github/workflows/release.yml`**: the `cargo-publish` step's
  `if-then-else` swallowed real publish failures as a misleading
  `cargo publish failed — likely version already published` warning.
  Now distinguishes "already uploaded" (idempotent re-run, exits 0) from
  any other failure (hard-fails the pipeline with the full cargo error).
- Dropped `rust-version = "1.78"` MSRV anchor from `Cargo.toml` and the
  matching `1.78` matrix entry from `ci.yml`. Transitive deps (`toml_datetime`
  needs edition2024 ≥ 1.85, `icu_*` 2.2 needs ≥ 1.86) churn faster than
  is worth chasing for a research scaffold. Re-introduce a stable MSRV
  anchor at v1.0 when the API stabilizes and downstream consumers
  actually need a floor.

### Changed

- **Probe module is no longer behind the `bin` feature**. It's always-on
  in the lib so the Python bindings can expose it without requiring the
  CLI dependency tree.
- **Dropped `tokio` and `reqwest` deps** — the probe is sync stdlib-only.
  `bin` feature now pulls only `clap` + `anyhow` + `serde_json`.
- **Probe never errors at the report level**: per-protocol failures land
  in `ProbeReport.diagnostics`; the function only returns `Err` for
  reserved future error variants.

### Known limitations

- RESTCONF detection is currently TCP-port-open only; it cannot
  distinguish "RESTCONF enabled" from "vendor web UI on 443." v0.3 wires
  in proper HTTPS content discrimination.
- Firmware-version extraction always returns `None`. Authoritative
  extraction needs an authenticated `show version` round-trip, which the
  probe deliberately doesn't do.
- Catalog-driven per-vendor probe sequencing is not yet wired —
  `vendor` parameter is captured in the report but doesn't yet route
  the probe sequence.

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
