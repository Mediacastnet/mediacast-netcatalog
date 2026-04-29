//! Version-matcher tests targeting the corners that real vendor firmware
//! strings hit (Aruba family prefixes, Cisco parens, IOS-XE drift).

use mediacast_netcatalog::{FirmwareVersion, VersionRange};

#[test]
fn aoscx_family_prefix_required_to_match() {
    let fw = FirmwareVersion::parse("FL.10.13.1000").unwrap();
    assert_eq!(fw.family_prefix.as_deref(), Some("FL"));
    assert_eq!(fw.major, 10);
    assert_eq!(fw.patch, 1000);
}

#[test]
fn cisco_paren_normalized() {
    let fw = FirmwareVersion::parse("9.3(5)").unwrap();
    assert_eq!((fw.major, fw.minor, fw.patch), (9, 3, 5));
}

#[test]
fn range_specificity_ordering() {
    let wide = VersionRange::parse(">=15.0").unwrap();
    let narrow = VersionRange::parse(">=15.0,<17.0").unwrap();
    assert!(narrow.specificity() > wide.specificity());
}

#[test]
fn wildcard_specificity_is_zero() {
    assert_eq!(VersionRange::parse("*").unwrap().specificity(), 0);
}

#[test]
fn ios_xe_16_6_satisfies_lower_bound() {
    let r = VersionRange::parse(">=16.6").unwrap();
    let fw = FirmwareVersion::parse("16.6.1").unwrap();
    assert!(r.matches(&fw));
}

#[test]
fn ios_xe_15_disqualified_from_17_range() {
    let r = VersionRange::parse(">=17.0").unwrap();
    let fw = FirmwareVersion::parse("15.2.7").unwrap();
    assert!(!r.matches(&fw));
}
