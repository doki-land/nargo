//! Integration tests for integrity module.

use nargo_cache::{HashAlgorithm, Integrity};

#[test]
fn test_integrity_from_bytes() {
    let data = b"hello world";
    let integrity = Integrity::from_bytes(data);

    assert_eq!(integrity.algorithm, HashAlgorithm::Sha512);
    assert!(!integrity.hash.is_empty());
}

#[test]
fn test_integrity_verify() {
    let data = b"hello world";
    let integrity = Integrity::from_bytes(data);

    assert!(integrity.verify(data).unwrap());
    assert!(!integrity.verify(b"different data").unwrap());
}

#[test]
fn test_sri_format() {
    let data = b"test data";
    let integrity = Integrity::from_bytes(data);
    let sri = integrity.to_sri();

    assert!(sri.starts_with("sha512-"));

    let parsed = Integrity::parse_sri(&sri).unwrap();
    assert_eq!(parsed, integrity);
}

#[test]
fn test_sha256() {
    let data = b"test";
    let integrity = Integrity::sha256(data);

    assert_eq!(integrity.algorithm, HashAlgorithm::Sha256);
    assert!(integrity.verify(data).unwrap());
}
