# Vendor Command Catalog — Abstract Command Types

The abstract types each catalog file must cover. Extracted from
`netcaster_engine/platforms/__init__.py:SwitchPlatform` (the existing
ABC) plus a few additions (HOSTNAME, VERSION) that the engine uses
implicitly today.

Use these exact identifiers (uppercase + underscores) as the `type`
field in `<vendor>.yaml::commands`.

## Read operations

| Type | Description | Implements (current `SwitchPlatform` method) |
|---|---|---|
| `ARP_TABLE` | Display the ARP table | `get_arp_command` |
| `MAC_TABLE` | Display the MAC address table | `get_mac_table_command` |
| `NEIGHBOR_DETAIL` | CDP/LLDP neighbor detail (the vendor's preferred protocol — typically CDP on Cisco, LLDP on Aruba) | `get_neighbor_command` |
| `LLDP_DETAIL` | LLDP neighbor detail specifically (used on Cisco when CDP is the primary; some neighbors only run LLDP) | `get_lldp_command` |
| `PORT_CHANNEL_MEMBERS` | Map port-channel logical interfaces to physical member interfaces | `get_port_channel_command` |
| `INTERFACE_CONFIG` | `show running-config interface <X>` for a single interface | `get_interface_config_command` |
| `INTERFACE_CONFIG_BULK` | All-at-once interface running-config (faster than per-interface for large switches) | `get_all_interface_configs_command` |
| `INTERFACE_STATS` | `show interface <X>` detailed counters / state | `get_interface_stats_command` |
| `CIVIC_LOCATION` | Display LLDP-MED civic location definitions (Aruba-specific today; capture if the vendor exposes it) | `get_civic_location_command` |
| `VLAN_LIST` | List all VLANs on the switch | `get_vlan_brief_command` |
| `SVI_CONFIG` | SVI / Switch Virtual Interface running-config sections (per-VLAN L3 settings) | `get_svi_config_command` |
| `STP_VLAN_LIST` | List of VLANs with spanning-tree enabled / participating | `get_spanning_tree_vlan_command` |
| `HARDWARE_IDENTITY` | Stable hardware identity: chassis serial, base MAC, stack member serials. Composite — typically a few commands combined | `get_hardware_identity` (and helper commands it invokes) |
| `HOSTNAME` | The switch's configured hostname (often read from prompt; some vendors require an explicit show) | (read from netmiko prompt today) |
| `VERSION` | Firmware version string. **Critical for the catalog itself** — used for version-aware selection | (read from `show version` today) |

## Write operations — port-level

| Type | Description | Implements |
|---|---|---|
| `PORT_SHUTDOWN` | Administratively disable an interface | `get_shutdown_commands` |
| `PORT_NO_SHUTDOWN` | Administratively enable an interface | `get_no_shutdown_commands` |
| `PORT_VLAN_ASSIGN` | Set the access VLAN of an interface | `get_vlan_assign_commands` |
| `PORT_POE_OFF` | Disable PoE on an interface | `get_poe_off_command` |
| `PORT_POE_ON` | Enable PoE on an interface | `get_poe_on_command` |
| `PORT_POLICY_PUSH` | Composite: portfast, BPDU guard, storm control, description. Vendor-specific composition | `get_port_config_commands` |

## Write operations — provisioning

| Type | Description | Implements |
|---|---|---|
| `VLAN_CREATE` | Create a VLAN definition (vid + optional name) | `get_vlan_create_commands` |
| `SVI_CREATE` | Create a Switch Virtual Interface for a VLAN, with optional IP / gateway / DHCP relay / IGMP / PIM | `get_svi_create_commands` |
| `STP_VLAN_ENABLE` | Enable spanning-tree for a VLAN | `get_spanning_tree_vlan_commands` |

## Persistence

| Type | Description | Implements |
|---|---|---|
| `SAVE_CONFIG` | Copy running-config to startup-config (or vendor equivalent — `commit` on Junos, etc.) | `get_save_config_command` |

## Coverage checklist

A complete `<vendor>.yaml` has an entry for **every** type above (24
total). When the vendor genuinely doesn't expose an operation (e.g.,
Meraki MS doesn't have a traditional `show running-config interface`
because it's cloud-managed), use `cli: "NOT_SUPPORTED"` and explain
in `notes`.

## Categorization rationale

Three groups (read / port-write / provisioning-write) match how the
engine currently invokes them:

- **Reads** are the bulk of inventory / discovery work; protocol-based
  alternatives (NETCONF, gNMI) are most commonly available here per
  ADR-0018's research.
- **Port-level writes** are the load-bearing risk surface (per the
  network-touching code review policy); CLI-only is the conservative
  default.
- **Provisioning writes** are higher-impact (creating a VLAN affects
  more than one port) but lower-frequency — used by NetCaster's
  smart-VLAN-suggestion remediation flow.

## Variables in `cli` strings

Recognized placeholders (when populating `cli` for command types that
take parameters):

| Placeholder | Meaning |
|---|---|
| `{interface}` | Interface identifier in the vendor's preferred form (e.g., `GigabitEthernet1/0/24` for Cisco, `1/1/24` for AOS-CX) |
| `{vlan_id}` | VLAN ID (integer 1–4094) |
| `{name}` | VLAN name |
| `{mac}` | MAC address in the vendor's preferred format |
| `{ip_address}` | IP address (no CIDR unless the command needs it) |
| `{cidr}` | CIDR notation (e.g., `10.1.1.0/24`) |
| `{description}` | Free-text description (operator-supplied) |

For commands that take no parameters (like `ARP_TABLE`'s `show arp`),
no placeholders. Keep the string literal.
