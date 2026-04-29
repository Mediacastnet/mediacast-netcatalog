# Vendor Command Catalog — Status

Roll-up of research progress per vendor. Updated as each `<vendor>.yaml`
file lands.

## Coverage matrix

| Vendor | File | Status | Coverage | Notes |
|---|---|---|---|---|
| Cisco IOS / IOS-XE | [`cisco-ios-xe.yaml`](cisco-ios-xe.yaml) | 🟢 Complete | 25/25 (0 NOT_SUPPORTED) | Mirrors existing handler; adds firmware-version awareness (12.x vs 15.x vs 17.x output drift). Real YANG models cited from YangModels/yang. |
| Cisco NX-OS | [`cisco-nxos.yaml`](cisco-nxos.yaml) | 🟢 Complete | 25/25 (0 NOT_SUPPORTED) | Documents 7+ parser-load-bearing divergences from IOS that would silently break the existing handler if used unmodified on Nexus (DHCP relay syntax, portfast naming, LLDP labels, IP/CIDR format, vPC awareness, modular-chassis serial model). |
| Aruba AOS-CX | [`aruba-aoscx.yaml`](aruba-aoscx.yaml) | 🟢 Complete | 25/25 (0 NOT_SUPPORTED) | Confirms 2026-04 tech-stack-review's "weakest leg" verdict on protocol-based I/O: NETCONF effectively absent, gNMI brand-new (10.17), REST is non-standard (auto-generated from data model, not RESTCONF). Family-prefixed firmware strings (FL.10.13.x) require version-matcher special-casing. |
| Juniper Junos OS | [`juniper-junos.yaml`](juniper-junos.yaml) | 🟢 Complete | 25/25 (0 NOT_SUPPORTED) | Transactional config model is fundamentally different from Cisco/Aruba: every write is `configure → set/delete → commit`, single-line forms don't exist. `commit confirmed <minutes>` auto-rollback flagged as audit-grade safety net worth adopting platform-wide. |
| Arista EOS | [`arista-eos.yaml`](arista-eos.yaml) | 🟢 Complete | 25/25 (1 NOT_SUPPORTED — CIVIC_LOCATION) | Mostly Cisco-IOS-compatible at user level — but real divergences (interface naming `Et24`, CIDR not dotted-mask, `spanning-tree vlan-id`, `poe disabled`, default MSTP not PVST+, default L2 MTU 9214). eAPI added as schema extension under `protocol_alternatives`. |
| HPE / Aruba ProCurve | [`hpe-procurve.yaml`](hpe-procurve.yaml) | 🟢 Complete | 25/25 (1 NOT_SUPPORTED — CIVIC_LOCATION) | Severe protocol gap by design: NETCONF/RESTCONF/gNMI never delivered on this line. Only programmatic surface is proprietary REST API + SNMP. Schema extended with `rest_api` and `snmp` under `protocol_capabilities`. CLI footguns documented (e.g., `disable`/`enable` not `shutdown`, MAC format `xxxxxx-xxxxxx`, "Trunk" means LAG not 802.1Q tagged). |
| Cisco Meraki MS | [`meraki-mx-ms.yaml`](meraki-mx-ms.yaml) | 🟢 Complete (with strategic flag) | 23/25 populated as Dashboard API endpoints + 2 NOT_SUPPORTED | **Architectural divergence flagged.** Meraki is cloud-managed; primary interface is Dashboard API (REST + JSON), not CLI. Six load-bearing reasons NOT to shoehorn into `SwitchPlatform` ABC: org-scoped Bearer token (not per-device), aggregate org rate limit (not per-device), network/org-scoped operations (not switch-scoped), typed JSON not text-parsed CLI, atomic writes with no save step, no firmware-version drift. Recommended new `CloudManagedPlatform` abstraction. |

## Status legend

- ⚪ Not started
- 🟡 In progress
- 🟢 Complete
- ⚠️ Partial (some types missing or unverified)
- 🔴 Blocked

## Coverage breakdown

All 7 vendors completed 25/25 abstract command type coverage on
2026-04-28. Two vendors flagged CIVIC_LOCATION as `NOT_SUPPORTED`
(Arista EOS, HPE ProCurve — neither exposes LLDP-MED civic-address
config trees), and Meraki MS marked CIVIC_LOCATION + SAVE_CONFIG
`NOT_SUPPORTED` (no LLDP-MED civic; cloud-managed has no
running/startup distinction).

Aggregate: **175 abstract command type entries** (7 × 25), of which
**171 are populated** with concrete CLI / API endpoints and **4 are
explicitly NOT_SUPPORTED** with reasoning.

## Schema extensions discovered during research

The original [SCHEMA.md](SCHEMA.md) anticipated three protocol
alternatives (NETCONF / RESTCONF / gNMI). Three vendors required
additions, each documented in their respective YAML:

| Extension | Vendor(s) | Slot |
|---|---|---|
| `eapi` (JSON-RPC over HTTPS) | Arista EOS | Per-command in `protocol_alternatives` |
| `rest_api` (proprietary HTTPS REST) | HPE ProCurve, Aruba AOS-CX (its REST is non-RESTCONF) | Top-level under `protocol_capabilities` |
| `snmp` | HPE ProCurve | Top-level under `protocol_capabilities` |
| `dashboard_api` (cloud REST) | Cisco Meraki MS | Top-level under `protocol_capabilities` |

These are **additive extensions** — they don't conflict with the
core schema. Captured as a finding in [`FINDINGS.md`](FINDINGS.md);
the schema doc is updated to bless the pattern formally.

## Last updated

2026-04-28 — all 7 vendor catalogs landed via parallel research
agents. See [`FINDINGS.md`](FINDINGS.md) for the cross-vendor
synthesis.
