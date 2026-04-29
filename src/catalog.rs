//! Catalog: the typed in-memory representation of one or more vendor YAML files.
//!
//! Schema mirrors `catalog/SCHEMA.md`. See that doc for field semantics.

use crate::command_types::CommandType;
use crate::error::{Error, Result};
use crate::version::{FirmwareVersion, VersionRange};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Bundled catalog files — embedded at compile time so `Catalog::load_bundled()`
/// works without any filesystem access. Order is alphabetical; the loader
/// is order-independent.
const BUNDLED: &[(&str, &str)] = &[
    ("arista-eos.yaml", include_str!("../catalog/arista-eos.yaml")),
    ("aruba-aoscx.yaml", include_str!("../catalog/aruba-aoscx.yaml")),
    ("cisco-ios-xe.yaml", include_str!("../catalog/cisco-ios-xe.yaml")),
    ("cisco-nxos.yaml", include_str!("../catalog/cisco-nxos.yaml")),
    ("hpe-procurve.yaml", include_str!("../catalog/hpe-procurve.yaml")),
    ("juniper-junos.yaml", include_str!("../catalog/juniper-junos.yaml")),
    ("meraki-mx-ms.yaml", include_str!("../catalog/meraki-mx-ms.yaml")),
];

/// In-memory catalog. Indexed by vendor identifier (`cisco_ios`, `aruba_aoscx`,
/// etc.). Build via [`Catalog::load_bundled`] or [`Catalog::load_dir`].
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    vendors: IndexMap<String, VendorFile>,
}

impl Catalog {
    /// Load the catalog files bundled with this crate (no filesystem access).
    pub fn load_bundled() -> Result<Self> {
        let mut cat = Catalog::default();
        for (name, body) in BUNDLED {
            let parsed: VendorFile = serde_yaml::from_str(body)
                .map_err(|source| Error::CatalogParse { file: (*name).to_owned(), source })?;
            cat.vendors.insert(parsed.vendor.clone(), parsed);
        }
        Ok(cat)
    }

    /// Load every `*.yaml` file in a directory. Useful for consumer-supplied
    /// overrides on top of (or instead of) the bundled set.
    pub fn load_dir(dir: impl AsRef<Path>) -> Result<Self> {
        let mut cat = Catalog::default();
        for entry in std::fs::read_dir(dir.as_ref())? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
                continue;
            }
            let body = std::fs::read_to_string(&path)?;
            let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("?").to_owned();
            let parsed: VendorFile = serde_yaml::from_str(&body)
                .map_err(|source| Error::CatalogParse { file: name, source })?;
            cat.vendors.insert(parsed.vendor.clone(), parsed);
        }
        Ok(cat)
    }

    /// All vendor identifiers in this catalog.
    pub fn vendors(&self) -> impl Iterator<Item = &str> {
        self.vendors.keys().map(String::as_str)
    }

    /// Get the full vendor file by id.
    pub fn vendor(&self, id: &str) -> Option<&VendorFile> {
        self.vendors.get(id)
    }

    /// Look up the most-specific catalog entry for `(vendor, firmware, command)`.
    /// Returns `None` if the vendor doesn't have an entry for this command type.
    pub fn lookup(
        &self,
        vendor: &str,
        firmware: &str,
        command: CommandType,
    ) -> Result<Option<&CommandEntry>> {
        let vf = self.vendors.get(vendor).ok_or_else(|| Error::UnknownVendor(vendor.to_owned()))?;
        let fw = FirmwareVersion::parse(firmware)?;

        let Some(cmd) = vf.commands.iter().find(|c| c.command_type == command) else {
            return Ok(None);
        };

        // Pick the most-specific matching `versions` block.
        let mut best: Option<(&CommandEntry, usize)> = None;
        for entry in &cmd.versions {
            let range = VersionRange::parse(&entry.applies_to)?;
            if !range.matches(&fw) {
                continue;
            }
            let score = range.specificity();
            if best.map_or(true, |(_, s)| score > s) {
                best = Some((entry, score));
            }
        }

        match best {
            Some((entry, _)) => {
                if entry.cli == "NOT_SUPPORTED" {
                    return Err(Error::NotSupported {
                        vendor: vendor.to_owned(),
                        command,
                        reason: entry.notes.clone().unwrap_or_default(),
                    });
                }
                Ok(Some(entry))
            }
            None => Err(Error::NoMatchingEntry {
                vendor: vendor.to_owned(),
                firmware: Some(firmware.to_owned()),
                command,
            }),
        }
    }
}

// ── YAML schema types ───────────────────────────────────────────────

/// One vendor catalog file (`<vendor>.yaml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorFile {
    /// Stable vendor identifier (`cisco_ios`, `aruba_aoscx`, etc.).
    pub vendor: String,
    /// Human-readable display name.
    pub display_name: String,
    /// Manufacturer (Cisco Systems, HPE, etc.).
    pub manufacturer: String,
    /// Product family within the manufacturer's lineup.
    pub product_family: String,
    /// Free-form notes (multi-line OK).
    #[serde(default)]
    pub notes: Option<String>,
    /// Citations for the data in this file.
    #[serde(default)]
    pub sources: Vec<Source>,
    /// Per-protocol availability metadata.
    #[serde(default)]
    pub protocol_capabilities: ProtocolCapabilities,
    /// Per-command-type entries.
    pub commands: Vec<CommandBlock>,
}

/// A documentation citation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Document title.
    pub title: String,
    /// URL.
    pub url: String,
    /// ISO date (YYYY-MM-DD).
    pub accessed: String,
}

/// Top-level protocol-availability metadata for a vendor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolCapabilities {
    /// NETCONF availability.
    #[serde(default)]
    pub netconf: Option<ProtocolCapability>,
    /// RESTCONF availability.
    #[serde(default)]
    pub restconf: Option<ProtocolCapability>,
    /// gNMI availability.
    #[serde(default)]
    pub gnmi: Option<ProtocolCapability>,
    /// Vendor proprietary REST (Aruba AOS-CX, HPE ProCurve).
    #[serde(default)]
    pub rest_api: Option<ProtocolCapability>,
    /// SNMP (HPE ProCurve and similar legacy gear).
    #[serde(default)]
    pub snmp: Option<ProtocolCapability>,
    /// Cloud Dashboard API (Meraki).
    #[serde(default)]
    pub dashboard_api: Option<ProtocolCapability>,
}

/// Per-protocol availability info — when introduced + freeform notes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCapability {
    /// First firmware version that introduced this protocol.
    #[serde(default)]
    pub introduced_in: Option<String>,
    /// Vendor notes / config requirements.
    #[serde(default)]
    pub notes: Option<String>,
    /// Catch-all for vendor-specific extras (auth scheme, base URL, rate limits).
    #[serde(flatten)]
    pub extras: IndexMap<String, serde_yaml::Value>,
}

/// A single command-type block in a vendor file. May contain multiple
/// `versions` entries for firmware-version-aware selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandBlock {
    /// Abstract command type.
    #[serde(rename = "type")]
    pub command_type: CommandType,
    /// Human description.
    #[serde(default)]
    pub description: Option<String>,
    /// One entry per firmware-version range.
    pub versions: Vec<CommandEntry>,
    /// Protocol alternatives (NETCONF, gNMI, eAPI, etc.).
    #[serde(default)]
    pub protocol_alternatives: ProtocolAlternatives,
}

/// One concrete command entry — applies to a specific firmware-version range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEntry {
    /// Version range this entry applies to (e.g., `">=16.6,<17.0"`).
    pub applies_to: String,
    /// The CLI string. Use `NOT_SUPPORTED` to mark vendor-absent commands.
    pub cli: String,
    /// Real-world output sample.
    #[serde(default)]
    pub sample_output: Option<String>,
    /// Parser hints.
    #[serde(default)]
    pub parser_notes: Option<String>,
    /// Required device configuration for this command to work.
    #[serde(default)]
    pub config_required: Option<String>,
    /// Vendor-specific quirks / deprecation notes.
    #[serde(default)]
    pub notes: Option<String>,
    /// True if extracted from a heuristic source rather than vendor docs.
    #[serde(default)]
    pub unverified: bool,
}

/// Per-command protocol alternatives. All slots are optional / nullable in YAML.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolAlternatives {
    /// NETCONF/YANG mapping.
    #[serde(default)]
    pub netconf: Option<NetconfMapping>,
    /// RESTCONF mapping.
    #[serde(default)]
    pub restconf: Option<RestconfMapping>,
    /// gNMI mapping.
    #[serde(default)]
    pub gnmi: Option<GnmiMapping>,
    /// Arista eAPI (JSON-RPC over HTTPS).
    #[serde(default)]
    pub eapi: Option<EapiMapping>,
    /// Vendor proprietary REST (Aruba AOS-CX, HPE ProCurve).
    #[serde(default)]
    pub rest_api: Option<RestApiMapping>,
    /// SNMP (legacy gear).
    #[serde(default)]
    pub snmp: Option<SnmpMapping>,
    /// Cloud Dashboard API (Meraki).
    #[serde(default)]
    pub dashboard_api: Option<DashboardApiMapping>,
}

/// NETCONF/YANG mapping for a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetconfMapping {
    /// YANG model name.
    pub yang_model: String,
    /// Path within the model.
    pub data_path: String,
    /// Minimum firmware version.
    #[serde(default)]
    pub firmware_required: Option<String>,
}

/// RESTCONF mapping for a command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestconfMapping {
    /// Full path after the host.
    pub url_path: String,
    /// Minimum firmware version.
    #[serde(default)]
    pub firmware_required: Option<String>,
}

/// gNMI path mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GnmiMapping {
    /// gNMI path.
    pub path: String,
    /// Minimum firmware version.
    #[serde(default)]
    pub firmware_required: Option<String>,
}

/// Arista eAPI mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EapiMapping {
    /// JSON-RPC method (typically `"runCmds"`).
    pub method: String,
    /// Commands to execute.
    pub commands: Vec<String>,
    /// Output format (`"json"` or `"text"`).
    #[serde(default)]
    pub format: Option<String>,
    /// Minimum firmware version.
    #[serde(default)]
    pub firmware_required: Option<String>,
}

/// Vendor proprietary REST mapping (AOS-CX, ProCurve).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestApiMapping {
    /// HTTP method.
    pub method: String,
    /// URL path (relative to vendor base URL).
    pub path: String,
    /// Minimum firmware version.
    #[serde(default)]
    pub firmware_required: Option<String>,
    /// Catch-all (auth scheme, response shape, etc.).
    #[serde(flatten)]
    pub extras: IndexMap<String, serde_yaml::Value>,
}

/// SNMP OID mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnmpMapping {
    /// MIB / OID path.
    pub oid: String,
    /// SNMP version (`v2c`, `v3`).
    #[serde(default)]
    pub version: Option<String>,
}

/// Cloud Dashboard API mapping (Meraki).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardApiMapping {
    /// Endpoint shape (`GET /networks/{networkId}/clients`).
    pub endpoint: String,
    /// Catch-all (rate-limit notes, scope, etc.).
    #[serde(flatten)]
    pub extras: IndexMap<String, serde_yaml::Value>,
}
