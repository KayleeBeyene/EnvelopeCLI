//! Encryption CLI commands
//!
//! Provides commands for enabling, disabling, and managing encryption.

use clap::Subcommand;

use crate::config::{paths::EnvelopePaths, settings::Settings};
use crate::crypto::{
    derive_key, encrypt_string, decrypt_string, EncryptedData, KeyDerivationParams,
};
use crate::error::{EnvelopeError, EnvelopeResult};
use crate::storage::Storage;

/// Encryption management commands
#[derive(Subcommand)]
pub enum EncryptCommands {
    /// Enable encryption for your budget data
    Enable,

    /// Disable encryption (requires current passphrase)
    Disable,

    /// Change your encryption passphrase
    #[command(alias = "change")]
    ChangePassphrase,

    /// Show encryption status
    Status,

    /// Verify your passphrase is correct
    Verify,
}

/// Handle encryption commands
pub fn handle_encrypt_command(
    paths: &EnvelopePaths,
    settings: &mut Settings,
    storage: &Storage,
    cmd: EncryptCommands,
) -> EnvelopeResult<()> {
    match cmd {
        EncryptCommands::Enable => enable_encryption(paths, settings, storage),
        EncryptCommands::Disable => disable_encryption(paths, settings, storage),
        EncryptCommands::ChangePassphrase => change_passphrase(paths, settings),
        EncryptCommands::Status => show_status(settings),
        EncryptCommands::Verify => verify_passphrase(settings),
    }
}

/// Enable encryption on budget data
fn enable_encryption(
    paths: &EnvelopePaths,
    settings: &mut Settings,
    _storage: &Storage,
) -> EnvelopeResult<()> {
    if settings.is_encryption_enabled() {
        println!("Encryption is already enabled.");
        println!("Use 'envelope encrypt change-passphrase' to change your passphrase.");
        return Ok(());
    }

    println!("Enable Encryption");
    println!("================");
    println!();
    println!("Encryption protects your budget data with AES-256-GCM encryption.");
    println!("You will need to enter your passphrase each time you use EnvelopeCLI.");
    println!();
    println!("IMPORTANT: If you forget your passphrase, your data cannot be recovered!");
    println!();

    // Get passphrase
    let passphrase = prompt_new_passphrase()?;

    // Generate key derivation params
    let key_params = KeyDerivationParams::new();

    // Derive key
    println!("Deriving encryption key...");
    let key = derive_key(&passphrase, &key_params)?;

    // Create verification hash
    let verification = encrypt_string("envelope_verify", &key)?;
    let verification_json = serde_json::to_string(&verification)
        .map_err(|e| EnvelopeError::Encryption(format!("Failed to serialize verification: {}", e)))?;

    // Update settings
    settings.encryption.enabled = true;
    settings.encryption.key_params = Some(key_params);
    settings.encryption.verification_hash = Some(verification_json);
    settings.encryption_enabled = true; // Legacy field

    // Save settings
    settings.save(paths)?;

    println!();
    println!("Encryption enabled successfully!");
    println!();
    println!("Your data will be encrypted on the next save operation.");
    println!("Remember to keep your passphrase safe - there is no recovery mechanism!");

    Ok(())
}

/// Disable encryption
fn disable_encryption(
    paths: &EnvelopePaths,
    settings: &mut Settings,
    _storage: &Storage,
) -> EnvelopeResult<()> {
    if !settings.is_encryption_enabled() {
        println!("Encryption is not enabled.");
        return Ok(());
    }

    println!("Disable Encryption");
    println!("==================");
    println!();

    // Verify current passphrase
    let passphrase = prompt_passphrase("Enter current passphrase: ")?;
    verify_passphrase_internal(settings, &passphrase)?;

    println!("Passphrase verified.");
    println!();

    // Confirm disable
    print!("Are you sure you want to disable encryption? (yes/no): ");
    std::io::Write::flush(&mut std::io::stdout())?;

    let mut confirm = String::new();
    std::io::stdin().read_line(&mut confirm)?;

    if confirm.trim().to_lowercase() != "yes" {
        println!("Aborted.");
        return Ok(());
    }

    // Update settings
    settings.encryption.enabled = false;
    settings.encryption.key_params = None;
    settings.encryption.verification_hash = None;
    settings.encryption_enabled = false;

    // Save settings
    settings.save(paths)?;

    println!();
    println!("Encryption disabled successfully!");
    println!("Your data is now stored unencrypted.");

    Ok(())
}

/// Change the encryption passphrase
fn change_passphrase(paths: &EnvelopePaths, settings: &mut Settings) -> EnvelopeResult<()> {
    if !settings.is_encryption_enabled() {
        println!("Encryption is not enabled.");
        println!("Use 'envelope encrypt enable' to enable encryption first.");
        return Ok(());
    }

    println!("Change Passphrase");
    println!("=================");
    println!();

    // Verify current passphrase
    let current = prompt_passphrase("Enter current passphrase: ")?;
    verify_passphrase_internal(settings, &current)?;

    println!("Current passphrase verified.");
    println!();

    // Get new passphrase
    let new_passphrase = prompt_new_passphrase()?;

    // Generate new key derivation params
    let new_key_params = KeyDerivationParams::new();

    // Derive new key
    println!("Deriving new encryption key...");
    let new_key = derive_key(&new_passphrase, &new_key_params)?;

    // Create new verification hash
    let verification = encrypt_string("envelope_verify", &new_key)?;
    let verification_json = serde_json::to_string(&verification)
        .map_err(|e| EnvelopeError::Encryption(format!("Failed to serialize verification: {}", e)))?;

    // Update settings
    settings.encryption.key_params = Some(new_key_params);
    settings.encryption.verification_hash = Some(verification_json);

    // Save settings
    settings.save(paths)?;

    println!();
    println!("Passphrase changed successfully!");
    println!("Your data will be re-encrypted with the new key on the next save.");

    Ok(())
}

/// Show encryption status
fn show_status(settings: &Settings) -> EnvelopeResult<()> {
    println!("Encryption Status");
    println!("=================");
    println!();

    if settings.is_encryption_enabled() {
        println!("Status: ENABLED");
        println!();
        if let Some(ref params) = settings.encryption.key_params {
            println!("Key Derivation Parameters:");
            println!("  Algorithm: Argon2id");
            println!("  Memory Cost: {} KiB", params.memory_cost);
            println!("  Time Cost: {} iterations", params.time_cost);
            println!("  Parallelism: {} threads", params.parallelism);
        }
    } else {
        println!("Status: DISABLED");
        println!();
        println!("Your data is stored unencrypted.");
        println!("Run 'envelope encrypt enable' to enable encryption.");
    }

    Ok(())
}

/// Verify the current passphrase
fn verify_passphrase(settings: &Settings) -> EnvelopeResult<()> {
    if !settings.is_encryption_enabled() {
        println!("Encryption is not enabled.");
        return Ok(());
    }

    let passphrase = prompt_passphrase("Enter passphrase: ")?;

    match verify_passphrase_internal(settings, &passphrase) {
        Ok(()) => {
            println!("Passphrase is correct!");
            Ok(())
        }
        Err(_) => {
            println!("Passphrase is incorrect.");
            Err(EnvelopeError::Encryption("Invalid passphrase".to_string()))
        }
    }
}

/// Internal passphrase verification
fn verify_passphrase_internal(settings: &Settings, passphrase: &str) -> EnvelopeResult<()> {
    let key_params = settings.encryption.key_params.as_ref()
        .ok_or_else(|| EnvelopeError::Encryption("No key parameters found".to_string()))?;

    let verification_json = settings.encryption.verification_hash.as_ref()
        .ok_or_else(|| EnvelopeError::Encryption("No verification hash found".to_string()))?;

    let encrypted: EncryptedData = serde_json::from_str(verification_json)
        .map_err(|e| EnvelopeError::Encryption(format!("Invalid verification data: {}", e)))?;

    let key = derive_key(passphrase, key_params)?;

    let decrypted = decrypt_string(&encrypted, &key)?;

    if decrypted != "envelope_verify" {
        return Err(EnvelopeError::Encryption("Invalid passphrase".to_string()));
    }

    Ok(())
}

/// Prompt for a new passphrase with confirmation
fn prompt_new_passphrase() -> EnvelopeResult<String> {
    loop {
        let pass1 = prompt_passphrase("Enter new passphrase: ")?;

        if pass1.len() < 8 {
            println!("Passphrase must be at least 8 characters. Please try again.");
            continue;
        }

        let pass2 = prompt_passphrase("Confirm passphrase: ")?;

        if pass1 != pass2 {
            println!("Passphrases do not match. Please try again.");
            continue;
        }

        return Ok(pass1);
    }
}

/// Prompt for a passphrase (hidden input)
fn prompt_passphrase(prompt: &str) -> EnvelopeResult<String> {
    rpassword::prompt_password(prompt)
        .map_err(|e| EnvelopeError::Encryption(format!("Failed to read passphrase: {}", e)))
}

/// Get the derived key from a passphrase and settings
pub fn get_encryption_key(settings: &Settings, passphrase: &str) -> EnvelopeResult<crate::crypto::DerivedKey> {
    let key_params = settings.encryption.key_params.as_ref()
        .ok_or_else(|| EnvelopeError::Encryption("No key parameters found".to_string()))?;

    derive_key(passphrase, key_params)
}
