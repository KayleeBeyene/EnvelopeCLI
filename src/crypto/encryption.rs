//! AES-256-GCM encryption/decryption
//!
//! Provides authenticated encryption for data at rest using AES-256-GCM.
//! Each encryption operation generates a unique nonce for security.

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::rand_core::RngCore;
use serde::{Deserialize, Serialize};

use crate::error::{EnvelopeError, EnvelopeResult};

use super::DerivedKey;

/// Size of the AES-GCM nonce in bytes (96 bits)
const NONCE_SIZE: usize = 12;

/// Encrypted data with associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The nonce used for this encryption (base64 encoded)
    pub nonce: String,
    /// The encrypted ciphertext with authentication tag (base64 encoded)
    pub ciphertext: String,
    /// Version for future algorithm upgrades
    #[serde(default = "default_version")]
    pub version: u8,
}

fn default_version() -> u8 {
    1
}

impl EncryptedData {
    /// Create a new EncryptedData from raw bytes
    fn new(nonce: &[u8], ciphertext: &[u8]) -> Self {
        use base64::{engine::general_purpose::STANDARD, Engine};
        Self {
            nonce: STANDARD.encode(nonce),
            ciphertext: STANDARD.encode(ciphertext),
            version: 1,
        }
    }

    /// Decode the nonce from base64
    fn decode_nonce(&self) -> EnvelopeResult<Vec<u8>> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD.decode(&self.nonce).map_err(|e| {
            EnvelopeError::Encryption(format!("Invalid nonce encoding: {}", e))
        })
    }

    /// Decode the ciphertext from base64
    fn decode_ciphertext(&self) -> EnvelopeResult<Vec<u8>> {
        use base64::{engine::general_purpose::STANDARD, Engine};
        STANDARD.decode(&self.ciphertext).map_err(|e| {
            EnvelopeError::Encryption(format!("Invalid ciphertext encoding: {}", e))
        })
    }
}

/// Encrypt plaintext data using AES-256-GCM
///
/// Generates a random nonce for each encryption operation.
pub fn encrypt(plaintext: &[u8], key: &DerivedKey) -> EnvelopeResult<EncryptedData> {
    // Create cipher from key
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| EnvelopeError::Encryption(format!("Failed to create cipher: {}", e)))?;

    // Generate random nonce
    let mut nonce_bytes = [0u8; NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    // Encrypt
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| EnvelopeError::Encryption(format!("Encryption failed: {}", e)))?;

    Ok(EncryptedData::new(&nonce_bytes, &ciphertext))
}

/// Decrypt ciphertext using AES-256-GCM
pub fn decrypt(encrypted: &EncryptedData, key: &DerivedKey) -> EnvelopeResult<Vec<u8>> {
    // Verify version
    if encrypted.version != 1 {
        return Err(EnvelopeError::Encryption(format!(
            "Unsupported encryption version: {}",
            encrypted.version
        )));
    }

    // Create cipher from key
    let cipher = Aes256Gcm::new_from_slice(key.as_bytes())
        .map_err(|e| EnvelopeError::Encryption(format!("Failed to create cipher: {}", e)))?;

    // Decode nonce and ciphertext
    let nonce_bytes = encrypted.decode_nonce()?;
    if nonce_bytes.len() != NONCE_SIZE {
        return Err(EnvelopeError::Encryption(format!(
            "Invalid nonce size: expected {}, got {}",
            NONCE_SIZE,
            nonce_bytes.len()
        )));
    }
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = encrypted.decode_ciphertext()?;

    // Decrypt
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| EnvelopeError::Encryption("Decryption failed: invalid key or corrupted data".to_string()))?;

    Ok(plaintext)
}

/// Encrypt a string
pub fn encrypt_string(plaintext: &str, key: &DerivedKey) -> EnvelopeResult<EncryptedData> {
    encrypt(plaintext.as_bytes(), key)
}

/// Decrypt to a string
pub fn decrypt_string(encrypted: &EncryptedData, key: &DerivedKey) -> EnvelopeResult<String> {
    let plaintext = decrypt(encrypted, key)?;
    String::from_utf8(plaintext)
        .map_err(|e| EnvelopeError::Encryption(format!("Invalid UTF-8 in decrypted data: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::key_derivation::{derive_key, KeyDerivationParams};

    fn test_key() -> DerivedKey {
        let params = KeyDerivationParams::new();
        derive_key("test_passphrase", &params).unwrap()
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = test_key();
        let plaintext = "Hello, World!";

        let encrypted = encrypt_string(plaintext, &key).unwrap();
        let decrypted = decrypt_string(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let encrypted1 = encrypt(plaintext, &key).unwrap();
        let encrypted2 = encrypt(plaintext, &key).unwrap();

        // Same plaintext should produce different ciphertext (different nonces)
        assert_ne!(encrypted1.nonce, encrypted2.nonce);
        assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = test_key();
        let params2 = KeyDerivationParams::new();
        let key2 = derive_key("different_passphrase", &params2).unwrap();

        let plaintext = b"Hello, World!";
        let encrypted = encrypt(plaintext, &key1).unwrap();

        // Decryption with wrong key should fail
        let result = decrypt(&encrypted, &key2);
        assert!(result.is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"Hello, World!";

        let mut encrypted = encrypt(plaintext, &key).unwrap();

        // Tamper with ciphertext
        use base64::{engine::general_purpose::STANDARD, Engine};
        let mut ciphertext = STANDARD.decode(&encrypted.ciphertext).unwrap();
        if !ciphertext.is_empty() {
            ciphertext[0] ^= 0xFF;
        }
        encrypted.ciphertext = STANDARD.encode(&ciphertext);

        // Decryption should fail due to authentication
        let result = decrypt(&encrypted, &key);
        assert!(result.is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let key = test_key();
        let plaintext = b"";

        let encrypted = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted.as_slice());
    }

    #[test]
    fn test_large_plaintext() {
        let key = test_key();
        let plaintext: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();

        let encrypted = encrypt(&plaintext, &key).unwrap();
        let decrypted = decrypt(&encrypted, &key).unwrap();

        assert_eq!(plaintext, decrypted);
    }
}
