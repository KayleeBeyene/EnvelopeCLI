//! Cryptographic functions for EnvelopeCLI
//!
//! Provides AES-256-GCM encryption with Argon2id key derivation
//! for optional at-rest encryption of budget data.

pub mod encryption;
pub mod key_derivation;
pub mod secure_memory;

pub use encryption::{decrypt, decrypt_string, encrypt, encrypt_string, EncryptedData};
pub use key_derivation::{derive_key, DerivedKey, KeyDerivationParams};
pub use secure_memory::SecureString;
