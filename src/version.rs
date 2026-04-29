//! Firmware version parsing + range matching.
//!
//! Implements a small SemVer-flavored subset that handles the corners
//! found in real vendor firmware strings:
//!
//! - Standard SemVer: `17.6.4`, `4.32.2F`
//! - Aruba AOS-CX family-prefixed: `FL.10.13.1000`, `GL.10.16.0001`,
//!   `LL.10.13.0010` — prefix is stripped before SemVer compare.
//! - Cisco classic with parentheses: `9.3(5)`, `15.2(7)E3` — normalized.
//! - Junos service-release suffixes: `21.4R3-S2.4` — major/minor/patch
//!   extracted; suffix kept for equality checks.
//!
//! Range syntax (matches `catalog/SCHEMA.md`):
//!
//! - `">=16.6"`         — single bound
//! - `">=15.0,<17.0"`   — comma = AND
//! - `">=15.0 || >=17.0"` — `||` = OR (disjunction)
//! - `"*"`              — any version
//!
//! When multiple [`VersionRange`] entries match a firmware, the
//! **most-specific wins** — narrower upper/lower bounds beat wider ones.

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

/// Aruba AOS-CX family prefixes — stripped before SemVer comparison.
const AOSCX_PREFIXES: &[&str] = &["FL.", "GL.", "LL.", "ML.", "DL.", "PL.", "QL."];

/// A parsed firmware version. Compares lexicographically by
/// `(major, minor, patch, build)` after normalization.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FirmwareVersion {
    /// Optional family prefix (Aruba AOS-CX `FL`, `GL`, etc.).
    pub family_prefix: Option<String>,
    /// Major version.
    pub major: u32,
    /// Minor version.
    pub minor: u32,
    /// Patch version.
    pub patch: u32,
    /// Build / 4th component (defaults to 0).
    pub build: u32,
    /// Trailing suffix preserved verbatim (e.g., `R3-S2.4`, `F`).
    pub suffix: Option<String>,
}

impl FirmwareVersion {
    /// Parse a firmware string. Tolerant — strips family prefixes and
    /// Cisco-style parentheses before SemVer-parsing.
    pub fn parse(s: &str) -> Result<Self> {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return Err(Error::BadFirmwareVersion(s.to_owned()));
        }

        // Strip Aruba-style family prefix.
        let (family_prefix, body) = AOSCX_PREFIXES
            .iter()
            .find_map(|p| trimmed.strip_prefix(p).map(|rest| (Some(p.trim_end_matches('.').to_owned()), rest)))
            .unwrap_or((None, trimmed));

        // Normalize Cisco-style `9.3(5)` → `9.3.5`. Bracket suffix becomes patch.
        let normalized = body.replace('(', ".").replace(')', "");

        // Split off trailing suffix (any non-numeric tail after a -, R, F, etc.).
        let (numeric, suffix) = split_suffix(&normalized);

        let parts: Vec<&str> = numeric.split('.').collect();
        if parts.is_empty() || parts[0].is_empty() {
            return Err(Error::BadFirmwareVersion(s.to_owned()));
        }

        let parse_u32 = |idx: usize| -> Result<u32> {
            parts.get(idx)
                .copied()
                .unwrap_or("0")
                .parse::<u32>()
                .map_err(|_| Error::BadFirmwareVersion(s.to_owned()))
        };

        Ok(Self {
            family_prefix,
            major: parse_u32(0)?,
            minor: parse_u32(1)?,
            patch: parse_u32(2)?,
            build: parse_u32(3)?,
            suffix,
        })
    }
}

fn split_suffix(s: &str) -> (String, Option<String>) {
    // First non-numeric, non-dot char starts the suffix.
    for (i, ch) in s.char_indices() {
        if !ch.is_ascii_digit() && ch != '.' {
            let (head, tail) = s.split_at(i);
            return (head.trim_end_matches('.').to_owned(), Some(tail.to_owned()));
        }
    }
    (s.to_owned(), None)
}

/// A version range expression. Constructed via [`VersionRange::parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VersionRange {
    /// Disjunctive list of conjunctive clauses. `[ [a, b], [c] ]` =
    /// `(a AND b) OR (c)`.
    clauses: Vec<Vec<Bound>>,
    /// Whether this range is `*` (matches anything).
    wildcard: bool,
    /// Original expression string, preserved for diagnostics + ordering.
    raw: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Bound {
    Ge(FirmwareVersion),
    Gt(FirmwareVersion),
    Le(FirmwareVersion),
    Lt(FirmwareVersion),
    Eq(FirmwareVersion),
}

impl VersionRange {
    /// Parse a range expression like `">=16.6,<17.0"` or `"*"`.
    pub fn parse(expr: &str) -> Result<Self> {
        let trimmed = expr.trim();
        if trimmed == "*" {
            return Ok(Self { clauses: vec![], wildcard: true, raw: expr.to_owned() });
        }

        let mut clauses = Vec::new();
        for disj in trimmed.split("||") {
            let mut conj = Vec::new();
            for piece in disj.split(',') {
                conj.push(parse_bound(piece.trim()).map_err(|reason| Error::BadVersionRange {
                    expr: expr.to_owned(),
                    reason,
                })?);
            }
            if !conj.is_empty() {
                clauses.push(conj);
            }
        }

        if clauses.is_empty() {
            return Err(Error::BadVersionRange {
                expr: expr.to_owned(),
                reason: "empty range".to_owned(),
            });
        }

        Ok(Self { clauses, wildcard: false, raw: expr.to_owned() })
    }

    /// Test whether a firmware version satisfies this range.
    pub fn matches(&self, v: &FirmwareVersion) -> bool {
        if self.wildcard {
            return true;
        }
        self.clauses.iter().any(|conj| conj.iter().all(|b| b.matches(v)))
    }

    /// A heuristic specificity score — higher = more specific.
    /// Wildcards score 0; bounded ranges score by clause count.
    pub fn specificity(&self) -> usize {
        if self.wildcard {
            return 0;
        }
        self.clauses.iter().map(|c| c.len()).sum()
    }

    /// Original range expression (for diagnostics).
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

fn parse_bound(s: &str) -> std::result::Result<Bound, String> {
    let (op, rest) = if let Some(r) = s.strip_prefix(">=") {
        (">=", r)
    } else if let Some(r) = s.strip_prefix("<=") {
        ("<=", r)
    } else if let Some(r) = s.strip_prefix(">") {
        (">", r)
    } else if let Some(r) = s.strip_prefix("<") {
        ("<", r)
    } else if let Some(r) = s.strip_prefix("=") {
        ("=", r)
    } else {
        ("=", s)
    };
    let v = FirmwareVersion::parse(rest.trim())
        .map_err(|e| format!("can't parse '{}': {}", rest.trim(), e))?;
    Ok(match op {
        ">=" => Bound::Ge(v),
        ">" => Bound::Gt(v),
        "<=" => Bound::Le(v),
        "<" => Bound::Lt(v),
        _ => Bound::Eq(v),
    })
}

impl Bound {
    fn matches(&self, v: &FirmwareVersion) -> bool {
        // Family prefix must match if both present.
        let other = match self {
            Bound::Ge(b) | Bound::Gt(b) | Bound::Le(b) | Bound::Lt(b) | Bound::Eq(b) => b,
        };
        if let (Some(a), Some(b)) = (&v.family_prefix, &other.family_prefix) {
            if a != b {
                return false;
            }
        }
        let cmp = (v.major, v.minor, v.patch, v.build).cmp(&(other.major, other.minor, other.patch, other.build));
        match self {
            Bound::Ge(_) => cmp.is_ge(),
            Bound::Gt(_) => cmp.is_gt(),
            Bound::Le(_) => cmp.is_le(),
            Bound::Lt(_) => cmp.is_lt(),
            Bound::Eq(_) => cmp.is_eq(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_semver() {
        let v = FirmwareVersion::parse("17.6.4").unwrap();
        assert_eq!(v.major, 17);
        assert_eq!(v.minor, 6);
        assert_eq!(v.patch, 4);
        assert_eq!(v.family_prefix, None);
    }

    #[test]
    fn parses_aoscx_family_prefix() {
        let v = FirmwareVersion::parse("FL.10.13.1000").unwrap();
        assert_eq!(v.family_prefix.as_deref(), Some("FL"));
        assert_eq!(v.major, 10);
        assert_eq!(v.minor, 13);
        assert_eq!(v.patch, 1000);
    }

    #[test]
    fn parses_cisco_parens() {
        let v = FirmwareVersion::parse("9.3(5)").unwrap();
        assert_eq!((v.major, v.minor, v.patch), (9, 3, 5));
    }

    #[test]
    fn wildcard_matches_anything() {
        let r = VersionRange::parse("*").unwrap();
        assert!(r.matches(&FirmwareVersion::parse("1.0.0").unwrap()));
        assert!(r.matches(&FirmwareVersion::parse("FL.10.13.1000").unwrap()));
    }

    #[test]
    fn ge_lt_range_matches_inside() {
        let r = VersionRange::parse(">=15.0,<17.0").unwrap();
        assert!(r.matches(&FirmwareVersion::parse("15.0.0").unwrap()));
        assert!(r.matches(&FirmwareVersion::parse("16.6.4").unwrap()));
        assert!(!r.matches(&FirmwareVersion::parse("17.0.0").unwrap()));
        assert!(!r.matches(&FirmwareVersion::parse("14.9.99").unwrap()));
    }
}
