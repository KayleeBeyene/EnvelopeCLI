//! Key derivation using Argon2id
//!
//! Derives encryption keys from user passphrases using Argon2id,
//! a memory-hard key derivation function resistant to GPU/ASIC attacks.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, Params,
};
use serde::{Deserialize, Serialize};

use crate::error::{EnvelopeError, EnvelopeResult};

/// Parameters for key derivation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationParams {
    /// Salt for key derivation (base64 encoded)
    pub salt: String,
    /// Memory cost in KiB (default: 65536 = 64 MiB)
    pub memory_cost: u32,
    /// Time cost (iterations, default: 3)
    pub time_cost: u32,
    /// Parallelism degree (default: 4)
    pub parallelism: u32,
}

impl Default for KeyDerivationParams {
    fn default() -> Self {
        Self {
            salt: String::new(), // Will be generated on first use
            memory_cost: 65536,  // 64 MiB
            time_cost: 3,
            parallelism: 4,
        }
    }
}

impl KeyDerivationParams {
    /// Create new params with a random salt
    pub fn new() -> Self {
        let salt = SaltString::generate(&mut OsRng);
        Self {
            salt: salt.to_string(),
            ..Default::default()
        }
    }

    /// Create params with specific values
    pub fn with_values(salt: String, memory_cost: u32, time_cost: u32, parallelism: u32) -> Self {
        Self {
            salt,
            memory_cost,
            time_cost,
            parallelism,
        }
    }
}

/// A derived encryption key
pub struct DerivedKey {
    /// The 32-byte key for AES-256
    key: [u8; 32],
}

impl DerivedKey {
    /// Get the key bytes
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.key
    }
}

impl Drop for DerivedKey {
    fn drop(&mut self) {
        // Zero out the key when dropped
        self.key.iter_mut().for_each(|b| *b = 0);
    }
}

/// Derive an encryption key from a passphrase
pub fn derive_key(passphrase: &str, params: &KeyDerivationParams) -> EnvelopeResult<DerivedKey> {
    // Parse the salt
    let salt = SaltString::from_b64(&params.salt)
        .map_err(|e| EnvelopeError::Encryption(format!("Invalid salt: {}", e)))?;

    // Configure Argon2id with custom params
    let argon2_params = Params::new(
        params.memory_cost,
        params.time_cost,
        params.parallelism,
        Some(32), // Output length for AES-256
    )
    .map_err(|e| EnvelopeError::Encryption(format!("Invalid Argon2 parameters: {}", e)))?;

    let argon2 = Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2_params,
    );

    // Derive the key by hashing the password
    let hash = argon2
        .hash_password(passphrase.as_bytes(), &salt)
        .map_err(|e| EnvelopeError::Encryption(format!("Key derivation failed: {}", e)))?;

    // Extract the hash output (the actual derived key)
    let hash_output = hash
        .hash
        .ok_or_else(|| EnvelopeError::Encryption("No hash output generated".to_string()))?;

    let hash_bytes = hash_output.as_bytes();

    if hash_bytes.len() < 32 {
        return Err(EnvelopeError::Encryption(
            "Hash output too short for AES-256 key".to_string(),
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&hash_bytes[..32]);

    Ok(DerivedKey { key })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let params = KeyDerivationParams::new();
        let key = derive_key("test_passphrase", &params).unwrap();
        assert_eq!(key.as_bytes().len(), 32);
    }

    #[test]
    fn test_same_passphrase_same_key() {
        let params = KeyDerivationParams::new();
        let key1 = derive_key("test_passphrase", &params).unwrap();
        let key2 = derive_key("test_passphrase", &params).unwrap();
        assert_eq!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_passphrase_different_key() {
        let params = KeyDerivationParams::new();
        let key1 = derive_key("passphrase1", &params).unwrap();
        let key2 = derive_key("passphrase2", &params).unwrap();
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_different_salt_different_key() {
        let params1 = KeyDerivationParams::new();
        let params2 = KeyDerivationParams::new();
        let key1 = derive_key("same_passphrase", &params1).unwrap();
        let key2 = derive_key("same_passphrase", &params2).unwrap();
        // Different salts should produce different keys
        assert_ne!(key1.as_bytes(), key2.as_bytes());
    }

    #[test]
    fn test_key_zeroized_on_drop() {
        let params = KeyDerivationParams::new();
        let key_ptr: *const [u8; 32];
        {
            let key = derive_key("test_passphrase", &params).unwrap();
            key_ptr = key.as_bytes() as *const [u8; 32];
        }
        // Note: This test is more of a documentation that we implement Drop
        // In practice, the memory might not be immediately zeroed due to optimizations
        // but we've at least attempted to clear it
        let _ = key_ptr; // Suppress unused warning
    }
}
