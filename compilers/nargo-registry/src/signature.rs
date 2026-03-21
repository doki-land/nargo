//! Package signature generation and verification.

use nargo_types::{Error, Result, Span};
use ring::{rand, signature, signature::KeyPair as RingKeyPair};
use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

/// Key pair for package signing.
#[derive(Debug, Clone)]
pub struct KeyPair {
    /// Private key in PKCS#8 format.
    private_key: Vec<u8>,
    /// Public key in SPKI format.
    public_key: Vec<u8>,
}

/// Public key for signature verification.
#[derive(Debug, Clone)]
pub struct PublicKey {
    /// Public key in SPKI format.
    public_key: Vec<u8>,
}

impl KeyPair {
    /// Generates a new Ed25519 key pair.
    pub fn generate() -> Result<Self> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to generate key pair: {:?}", e), Span::unknown()))?;

        let key_pair = signature::Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref()).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to parse key pair: {:?}", e), Span::unknown()))?;

        let public_key = key_pair.public_key().as_ref().to_vec();

        Ok(Self { private_key: pkcs8_bytes.as_ref().to_vec(), public_key })
    }

    /// Loads a key pair from a file.
    pub fn load(private_key_path: &PathBuf) -> Result<Self> {
        let mut file = File::open(private_key_path).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to open private key file: {:?}", e), Span::unknown()))?;

        let mut private_key = Vec::new();
        file.read_to_end(&mut private_key).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to read private key: {:?}", e), Span::unknown()))?;

        let key_pair = signature::Ed25519KeyPair::from_pkcs8(private_key.as_ref()).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to parse key pair: {:?}", e), Span::unknown()))?;

        let public_key = key_pair.public_key().as_ref().to_vec();

        Ok(Self { private_key, public_key })
    }

    /// Saves the private key to a file.
    pub fn save_private_key(&self, path: &PathBuf) -> Result<()> {
        let mut file = File::create(path).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to create private key file: {:?}", e), Span::unknown()))?;

        file.write_all(&self.private_key).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to write private key: {:?}", e), Span::unknown()))?;

        // Set file permissions to read-only for owner
        #[cfg(unix)]
        {
            {
                use std::os::unix::fs::PermissionsExt;
                let mut permissions = file.metadata().unwrap().permissions();
                permissions.set_mode(0o600);
                fs::set_permissions(path, permissions).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to set permissions: {:?}", e), Span::unknown()))?;
            }
        }

        Ok(())
    }

    /// Saves the public key to a file.
    pub fn save_public_key(&self, path: &PathBuf) -> Result<()> {
        let mut file = File::create(path).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to create public key file: {:?}", e), Span::unknown()))?;

        file.write_all(&self.public_key).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to write public key: {:?}", e), Span::unknown()))?;

        Ok(())
    }

    /// Signs a message.
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let key_pair = signature::Ed25519KeyPair::from_pkcs8(self.private_key.as_ref()).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to parse key pair: {:?}", e), Span::unknown()))?;

        let signature = key_pair.sign(message);
        Ok(signature.as_ref().to_vec())
    }

    /// Gets the public key.
    pub fn public_key(&self) -> PublicKey {
        PublicKey { public_key: self.public_key.clone() }
    }
}

impl PublicKey {
    /// Loads a public key from a file.
    pub fn load(path: &PathBuf) -> Result<Self> {
        let mut file = File::open(path).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to open public key file: {:?}", e), Span::unknown()))?;

        let mut public_key = Vec::new();
        file.read_to_end(&mut public_key).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to read public key: {:?}", e), Span::unknown()))?;

        Ok(Self { public_key })
    }

    /// Verifies a signature.
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<bool> {
        let public_key = signature::UnparsedPublicKey::new(&signature::ED25519, &self.public_key);

        match public_key.verify(message, signature) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Gets the public key as bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.public_key
    }
}

/// Signs a package tarball.
pub fn sign_package(tarball_path: &PathBuf, key_pair: &KeyPair) -> Result<Vec<u8>> {
    let mut file = File::open(tarball_path).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to open tarball: {:?}", e), Span::unknown()))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to read tarball: {:?}", e), Span::unknown()))?;

    key_pair.sign(&content)
}

/// Verifies a package tarball signature.
pub fn verify_package(tarball_path: &PathBuf, signature: &[u8], public_key: &PublicKey) -> Result<bool> {
    let mut file = File::open(tarball_path).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to open tarball: {:?}", e), Span::unknown()))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content).map_err(|e| Error::external_error("signature".to_string(), format!("Failed to read tarball: {:?}", e), Span::unknown()))?;

    public_key.verify(&content, signature)
}

/// Default key directory.
pub fn default_key_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".nargo").join("keys")
}

/// Default private key path.
pub fn default_private_key_path() -> PathBuf {
    default_key_dir().join("default.key")
}

/// Default public key path.
pub fn default_public_key_path() -> PathBuf {
    default_key_dir().join("default.pub")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_key_pair_generation() {
        let key_pair = KeyPair::generate().unwrap();
        assert!(!key_pair.private_key.is_empty());
        assert!(!key_pair.public_key.is_empty());
    }

    #[test]
    fn test_signature_verification() {
        let key_pair = KeyPair::generate().unwrap();
        let public_key = key_pair.public_key();
        let message = b"test message";
        let signature = key_pair.sign(message).unwrap();
        let verified = public_key.verify(message, &signature).unwrap();
        assert!(verified);
    }

    #[test]
    fn test_signature_verification_fails() {
        let key_pair = KeyPair::generate().unwrap();
        let public_key = key_pair.public_key();
        let message = b"test message";
        let signature = key_pair.sign(message).unwrap();
        let wrong_message = b"wrong message";
        let verified = public_key.verify(wrong_message, &signature).unwrap();
        assert!(!verified);
    }

    #[test]
    fn test_key_saving_and_loading() {
        let temp_dir = tempdir().unwrap();
        let private_key_path = temp_dir.path().join("private.key");
        let public_key_path = temp_dir.path().join("public.pub");

        let key_pair = KeyPair::generate().unwrap();
        key_pair.save_private_key(&private_key_path).unwrap();
        key_pair.save_public_key(&public_key_path).unwrap();

        let loaded_key_pair = KeyPair::load(&private_key_path).unwrap();
        let loaded_public_key = PublicKey::load(&public_key_path).unwrap();

        let message = b"test message";
        let signature = loaded_key_pair.sign(message).unwrap();
        let verified = loaded_public_key.verify(message, &signature).unwrap();
        assert!(verified);
    }
}
