//! Integrity verification implementation.

use nargo_types::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256, Sha512};

/// Supported hash algorithms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// SHA-256 hash.
    Sha256,
    /// SHA-512 hash.
    Sha512,
}

impl Default for HashAlgorithm {
    fn default() -> Self {
        HashAlgorithm::Sha512
    }
}

impl std::fmt::Display for HashAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashAlgorithm::Sha256 => write!(f, "sha256"),
            HashAlgorithm::Sha512 => write!(f, "sha512"),
        }
    }
}

/// Represents a content integrity hash.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Integrity {
    /// The hash algorithm used.
    pub algorithm: HashAlgorithm,
    /// The hex-encoded hash value.
    pub hash: String,
}

impl Default for Integrity {
    fn default() -> Self {
        Self { algorithm: HashAlgorithm::default(), hash: String::new() }
    }
}

impl Integrity {
    /// Creates a new integrity hash.
    pub fn new(algorithm: HashAlgorithm, hash: impl Into<String>) -> Self {
        Self { algorithm, hash: hash.into() }
    }

    /// Computes integrity from bytes using SHA-512.
    pub fn from_bytes(data: &[u8]) -> Self {
        let mut hasher = Sha512::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());
        Self::new(HashAlgorithm::Sha512, hash)
    }

    /// Computes integrity from bytes using SHA-256.
    pub fn sha256(data: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let hash = hex::encode(hasher.finalize());
        Self::new(HashAlgorithm::Sha256, hash)
    }

    /// Verifies data against this integrity hash.
    pub fn verify(&self, data: &[u8]) -> Result<bool> {
        let computed = match self.algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hex::encode(hasher.finalize())
            }
            HashAlgorithm::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(data);
                hex::encode(hasher.finalize())
            }
        };

        Ok(computed == self.hash)
    }

    /// Parses an SRI-style integrity string.
    pub fn parse_sri(sri: &str) -> Result<Self> {
        let parts: Vec<&str> = sri.splitn(2, '-').collect();
        if parts.len() != 2 {
            return Err(Error::external_error("integrity".to_string(), format!("Invalid SRI format: {}", sri), nargo_types::Span::unknown()));
        }

        let algorithm = match parts[0] {
            "sha256" => HashAlgorithm::Sha256,
            "sha512" => HashAlgorithm::Sha512,
            _ => return Err(Error::external_error("integrity".to_string(), format!("Unsupported hash algorithm: {}", parts[0]), nargo_types::Span::unknown())),
        };

        Ok(Self::new(algorithm, parts[1]))
    }

    /// Converts to SRI format string.
    pub fn to_sri(&self) -> String {
        format!("{}-{}", self.algorithm, self.hash)
    }
}

impl std::fmt::Display for Integrity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_sri())
    }
}
