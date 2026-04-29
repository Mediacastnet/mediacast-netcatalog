//! Abstract command types — vendor-neutral vocabulary for switch operations.
//!
//! See [`catalog/COMMAND_TYPES.md`](https://github.com/Mediacastnet/mediacast-netcatalog/blob/main/catalog/COMMAND_TYPES.md)
//! for prose descriptions.

use serde::{Deserialize, Serialize};

/// Abstract command vocabulary. Each variant maps to a concrete CLI string
/// per `(vendor, firmware version)` in the catalog YAML.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum CommandType {
    // ── Read operations ──────────────────────────────────────────────
    /// Display the ARP table.
    ArpTable,
    /// Display the MAC address table.
    MacTable,
    /// CDP/LLDP neighbor detail (vendor's preferred protocol).
    NeighborDetail,
    /// LLDP-specific neighbor detail.
    LldpDetail,
    /// Map port-channel logical interfaces to physical members.
    PortChannelMembers,
    /// `show running-config interface <X>` for a single interface.
    InterfaceConfig,
    /// All-at-once interface running-config.
    InterfaceConfigBulk,
    /// `show interface <X>` detailed counters / state.
    InterfaceStats,
    /// LLDP-MED civic location definitions.
    CivicLocation,
    /// List all VLANs on the switch.
    VlanList,
    /// SVI / Switch Virtual Interface running-config sections.
    SviConfig,
    /// List of VLANs with spanning-tree enabled / participating.
    StpVlanList,
    /// Stable hardware identity (chassis serial, base MAC, stack member serials).
    HardwareIdentity,
    /// The switch's configured hostname.
    Hostname,
    /// Firmware version string. Critical for the catalog's own selection logic.
    Version,

    // ── Write operations — port-level ────────────────────────────────
    /// Administratively disable an interface.
    PortShutdown,
    /// Administratively enable an interface.
    PortNoShutdown,
    /// Set the access VLAN of an interface.
    PortVlanAssign,
    /// Disable PoE on an interface.
    PortPoeOff,
    /// Enable PoE on an interface.
    PortPoeOn,
    /// Composite: portfast, BPDU guard, storm control, description.
    PortPolicyPush,

    // ── Write operations — provisioning ──────────────────────────────
    /// Create a VLAN definition.
    VlanCreate,
    /// Create a Switch Virtual Interface.
    SviCreate,
    /// Enable spanning-tree for a VLAN.
    StpVlanEnable,

    // ── Persistence ──────────────────────────────────────────────────
    /// Copy running-config to startup-config (or vendor equivalent).
    SaveConfig,
}

impl CommandType {
    /// Iterate over every defined command type. Useful for catalog
    /// completeness checks.
    pub fn all() -> &'static [CommandType] {
        &[
            CommandType::ArpTable,
            CommandType::MacTable,
            CommandType::NeighborDetail,
            CommandType::LldpDetail,
            CommandType::PortChannelMembers,
            CommandType::InterfaceConfig,
            CommandType::InterfaceConfigBulk,
            CommandType::InterfaceStats,
            CommandType::CivicLocation,
            CommandType::VlanList,
            CommandType::SviConfig,
            CommandType::StpVlanList,
            CommandType::HardwareIdentity,
            CommandType::Hostname,
            CommandType::Version,
            CommandType::PortShutdown,
            CommandType::PortNoShutdown,
            CommandType::PortVlanAssign,
            CommandType::PortPoeOff,
            CommandType::PortPoeOn,
            CommandType::PortPolicyPush,
            CommandType::VlanCreate,
            CommandType::SviCreate,
            CommandType::StpVlanEnable,
            CommandType::SaveConfig,
        ]
    }
}
