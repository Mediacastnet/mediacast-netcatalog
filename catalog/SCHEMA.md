# Vendor Command Catalog — YAML Schema

Each `<vendor>.yaml` file follows this shape. Every per-vendor research
file produced by the catalog effort conforms.

```yaml
# ── Vendor identity ────────────────────────────────────────────────
vendor: cisco_ios            # Stable id; matches the existing PLATFORM_MAP key when one exists.
display_name: "Cisco IOS / IOS-XE"
manufacturer: "Cisco Systems"
product_family: "Catalyst / IOS-XE"
notes: |                     # Free-form notes (multi-line ok).
  Single catalog covers IOS classic and IOS-XE — they share command
  surface for the abstract types in scope. Treat IOS-XE >= 16.6 as the
  baseline; IOS classic 12.x / 15.x have minor output drift noted per
  entry.

# ── Documentation sources ──────────────────────────────────────────
# Where the data in this file was extracted from. Reproducibility
# matters; future updates re-read the same docs.
sources:
  - title: "Cisco IOS-XE 17.x Command Reference"
    url: "https://www.cisco.com/.../ios-xe-17-cmd-reference.html"
    accessed: "2026-04-28"
  - title: "Catalyst 9000 Programmability Configuration Guide 17.x"
    url: "https://www.cisco.com/.../programmability-17.html"
    accessed: "2026-04-28"

# ── Protocol capabilities ──────────────────────────────────────────
# When each protocol became available; per-command applicability is
# captured under each `commands` entry's `protocol_alternatives`.
protocol_capabilities:
  netconf:
    introduced_in: "16.6"
    notes: "Requires `netconf-yang` to be enabled in running-config."
  restconf:
    introduced_in: "16.6"
    notes: "Requires `restconf` enable. HTTPS only on most platforms."
  gnmi:
    introduced_in: "16.10"
    notes: "Requires `gnmi-yang` enable; gRPC on TCP/9339."

# ── Commands ───────────────────────────────────────────────────────
# Each entry maps an abstract command type (from COMMAND_TYPES.md) to
# its concrete CLI implementation + protocol alternatives. Multiple
# `versions` blocks per type let a single file cover firmware drift.
commands:
  - type: ARP_TABLE
    description: "Display the ARP table"
    versions:
      - applies_to: ">=12.0"          # Version range expression
        cli: "show arp"
        sample_output: |
          Protocol  Address       Age (min)  Hardware Addr    Type   Interface
          Internet  10.0.0.1      120        001b.5377.fbe1   ARPA   Gi1/0/1
          Internet  10.0.0.42     0          aabb.cc00.0001   ARPA   Gi1/0/24
        parser_notes: |
          6-column space-separated. Hardware Addr is Cisco-dotted
          (xxxx.xxxx.xxxx). Age 0 = current; "incomplete" = no entry.
          Strip header line(s) before iterating rows.
        config_required: ""           # Empty if no opt-in needed.
        notes: |
          IOS classic 12.x has the same column set but 5 cols (no
          "Protocol"); guard the parser accordingly if 12.x support
          is needed.
    protocol_alternatives:
      netconf:
        yang_model: "Cisco-IOS-XE-arp-oper"
        data_path: "/arp-data"
        firmware_required: ">=16.6.1"
      restconf:
        url_path: "/restconf/data/Cisco-IOS-XE-arp-oper:arp-data"
        firmware_required: ">=16.6.1"
      gnmi:
        path: "/Cisco-IOS-XE-arp-oper:arp-data"
        firmware_required: ">=17.10"

  - type: MAC_TABLE
    description: "Display the MAC address table"
    versions:
      - applies_to: ">=15.0"
        cli: "show mac address-table"
        sample_output: |
          Vlan    Mac Address       Type        Ports
          ----    -----------       ----        -----
             1    aabb.cc00.0001    DYNAMIC     Gi1/0/24
        parser_notes: "..."
        config_required: ""
        notes: ""
    protocol_alternatives:
      netconf:
        yang_model: "Cisco-IOS-XE-mac-address-table-oper"
        data_path: "/mac-address-table"
        firmware_required: ">=16.6.1"
      restconf: null       # Use null when not supported.
      gnmi: null
```

## Field semantics

### `vendor` (string)
Stable identifier. For the v1 baseline, matches the existing
`PLATFORM_MAP` key in `netcaster_engine/platforms/__init__.py`
(`cisco_ios`, `cisco_nxos`, `aruba_aoscx`). For new vendors, pick
a snake_case identifier following the same convention
(`juniper_junos`, `arista_eos`, `hpe_procurve`, `meraki_ms`).

### `versions[].applies_to` (version range expression)
SemVer-flavored range. Recognized syntax:

- `">=16.6"` — version >= 16.6 (matches 16.6, 17.0, 17.10, etc.)
- `">=15.0,<17.0"` — comma-separated AND (range)
- `">=15.0 || >=17.0"` — `||` for disjunction (rarely needed)
- `"*"` — any version (use when version awareness genuinely doesn't
  matter for this command on this vendor)

When multiple `versions` blocks match, the **most specific** wins
(narrower range > wider range). The runtime catalog loader (future)
will implement this matching.

### `versions[].cli` (string)
The concrete CLI command. Single line. Variables in curly braces:

- `{interface}` — interface identifier
- `{vlan_id}` — VLAN ID (integer)
- `{mac}` — MAC address (vendor's preferred format)

For composite commands (multiple lines sent in config mode), use a
multi-line string with one command per line.

### `versions[].sample_output` (string, multi-line)
Real-world output snippet. Used as a parser-test fixture and a
sanity check that the catalog matches what real switches emit.
**Not invented** — pulled from vendor docs or real captures. Mark
as `unverified: true` if the source is heuristic.

### `versions[].parser_notes` (string)
What a parser needs to know to extract data from the output.
Column layout, separators, header lines to skip, sentinel values,
etc.

### `versions[].config_required` (string, multi-line)
Configuration the device must have for this command to work. Empty
string if no special config is needed. Examples:
- `cdp run` for CDP-based neighbors
- `lldp run` for LLDP-based neighbors
- `netconf-yang` for NETCONF
- `restconf` for RESTCONF

### `versions[].notes` (string)
Vendor-specific quirks, deprecation notes, gotchas. Anything a
reader needs to avoid surprises.

### `protocol_alternatives` (object)
Per-protocol mapping when the same data is available via NETCONF /
RESTCONF / gNMI. Use `null` for protocols where this command type
isn't exposed via that protocol on the vendor.

- `yang_model`: the YANG module name (e.g., `Cisco-IOS-XE-arp-oper`)
- `data_path`: the path within the model (e.g., `/arp-data`)
- `url_path` (RESTCONF only): full path after the host
- `path` (gNMI only): full gNMI path
- `firmware_required`: minimum firmware version

## Validation expectations

A complete catalog file:

- Has all command types from [`COMMAND_TYPES.md`](COMMAND_TYPES.md).
  Use `"NOT_SUPPORTED"` as the `cli` value when the vendor genuinely
  doesn't expose the operation (rare; document why in `notes`).
- Has at least one `versions` block per command type. Multiple blocks
  for firmware-version-sensitive commands.
- Has citations in `sources` for the firmware versions covered.
- Has `sample_output` populated where output-parsing is involved
  (read commands). Skip for write-only commands (shutdown, etc.).

## Schema extensions (added 2026-04-28 from research findings)

The 2026-04-28 research surfaced four protocol slots beyond the
original NETCONF / RESTCONF / gNMI trio. These are **additive**:
add to a catalog file as needed; don't remove or rename core slots.

### Per-command extension: `eapi` (Arista)

Arista's eAPI is JSON-RPC 2.0 over HTTPS. Treated as a 4th
protocol alternative parallel to NETCONF/RESTCONF/gNMI:

```yaml
commands:
  - type: ARP_TABLE
    versions: [...]
    protocol_alternatives:
      netconf: null
      restconf: null
      gnmi:
        path: "/network-instances/network-instance/protocols/protocol/aft/state"
        firmware_required: ">=4.20"
      eapi:                                    # ← extension
        method: "runCmds"
        commands: ["show ip arp"]
        format: "json"
        firmware_required: ">=4.13"
```

### Top-level extensions: `rest_api`, `snmp`, `dashboard_api`

When a vendor's primary programmatic interface isn't a standard
protocol, document it under `protocol_capabilities` alongside
`netconf` / `restconf` / `gnmi`:

```yaml
protocol_capabilities:
  netconf: null         # genuinely absent on this vendor
  restconf: null
  gnmi:
    introduced_in: "10.17"
    notes: "..."
  rest_api:                                    # ← extension (Aruba AOS-CX, HPE ProCurve)
    introduced_in: "10.04"
    base_url_pattern: "https://<host>/rest/v10.13"
    auth: "Cookie session via /rest/v10.x/login"
    notes: "Vendor-proprietary; not IETF RESTCONF."
  snmp:                                        # ← extension (HPE ProCurve)
    versions_supported: ["v2c", "v3"]
    notes: "Practical fallback for read state on legacy gear."
  dashboard_api:                               # ← extension (Cisco Meraki MS)
    base_url: "https://api.meraki.com/api/v1"
    auth: "Bearer token (org-scoped)"
    rate_limit: "10 rps per organization"
    notes: |
      Cloud-managed — Dashboard API is the primary management
      interface, not CLI. See FINDINGS.md for the architectural
      flag this raises.
```

Per-command references to these top-level extensions can use a
matching `protocol_alternatives` slot (e.g., `dashboard_api:` with
`endpoint: "GET /networks/{networkId}/clients"`) — same shape as
the core slots.

## Out-of-scope schema decisions (for now)

These are deliberately left for the future runtime ADR:

- How the catalog file is loaded at runtime (file-based? bundled?
  fetched from an updates feed?).
- How firmware-version detection feeds into command selection
  (engine reads `show version` first, caches the result, looks up).
- How to handle catalog updates without a NetCaster restart.
- Whether `dashboard_api` / cloud-managed vendors should live in
  this catalog at all, or be a parallel `cloud-managed-platform-
  catalog` (per Meraki research's strategic flag).

Keeping the schema simple now keeps these options open.
