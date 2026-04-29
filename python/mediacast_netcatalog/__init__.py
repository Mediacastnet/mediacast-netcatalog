"""mediacast-netcatalog: vendor command catalog + version matcher + protocol probe.

This is a thin Python re-export of the Rust core. The native extension lives
in ``mediacast_netcatalog._native``; the public API is here.
"""

from __future__ import annotations

from enum import Enum

from ._native import Catalog, __version__, probe_device

__all__ = ["Catalog", "CommandType", "probe_device", "__version__"]


class CommandType(str, Enum):
    """Abstract command vocabulary. Mirrors the Rust ``CommandType`` enum."""

    ARP_TABLE = "ARP_TABLE"
    MAC_TABLE = "MAC_TABLE"
    NEIGHBOR_DETAIL = "NEIGHBOR_DETAIL"
    LLDP_DETAIL = "LLDP_DETAIL"
    PORT_CHANNEL_MEMBERS = "PORT_CHANNEL_MEMBERS"
    INTERFACE_CONFIG = "INTERFACE_CONFIG"
    INTERFACE_CONFIG_BULK = "INTERFACE_CONFIG_BULK"
    INTERFACE_STATS = "INTERFACE_STATS"
    CIVIC_LOCATION = "CIVIC_LOCATION"
    VLAN_LIST = "VLAN_LIST"
    SVI_CONFIG = "SVI_CONFIG"
    STP_VLAN_LIST = "STP_VLAN_LIST"
    HARDWARE_IDENTITY = "HARDWARE_IDENTITY"
    HOSTNAME = "HOSTNAME"
    VERSION = "VERSION"
    PORT_SHUTDOWN = "PORT_SHUTDOWN"
    PORT_NO_SHUTDOWN = "PORT_NO_SHUTDOWN"
    PORT_VLAN_ASSIGN = "PORT_VLAN_ASSIGN"
    PORT_POE_OFF = "PORT_POE_OFF"
    PORT_POE_ON = "PORT_POE_ON"
    PORT_POLICY_PUSH = "PORT_POLICY_PUSH"
    VLAN_CREATE = "VLAN_CREATE"
    SVI_CREATE = "SVI_CREATE"
    STP_VLAN_ENABLE = "STP_VLAN_ENABLE"
    SAVE_CONFIG = "SAVE_CONFIG"
