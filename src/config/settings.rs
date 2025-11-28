//! User settings for EnvelopeCLI
//!
//! Manages user preferences including budget period type, encryption settings,
//! and backup retention policies.

use serde::{Deserialize, Serialize};

use super::paths::EnvelopePaths;
use crate::crypto::key_derivation::KeyDerivationParams;
use crate::error::EnvelopeError;

/// Budget period type preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum BudgetPeriodType {
    /// Monthly budgets (default, e.g., "2025-01")
    #[default]
    Monthly,
    /// Weekly budgets (e.g., "2025-W03")
    Weekly,
    /// Bi-weekly budgets
    BiWeekly,
}

/// Backup retention settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupRetention {
    /// Number of daily backups to keep
    pub daily_count: u32,
    /// Number of monthly backups to keep
    pub monthly_count: u32,
}

impl Default for BackupRetention {
    fn default() -> Self {
        Self {
            daily_count: 30,
            monthly_count: 12,
        }
    }
}

/// Encryption settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EncryptionSettings {
    /// Whether encryption is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Key derivation parameters (salt, memory cost, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_params: Option<KeyDerivationParams>,

    /// Verification hash to check if passphrase is correct
    /// (This is a hash of "envelope_verify" encrypted with the key)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_hash: Option<String>,
}

/// User settings for EnvelopeCLI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Schema version for migration support
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,

    /// User's preferred budget period type
    #[serde(default)]
    pub budget_period_type: BudgetPeriodType,

    /// Whether encryption is enabled (legacy field for backwards compat)
    #[serde(default)]
    pub encryption_enabled: bool,

    /// Full encryption settings
    #[serde(default)]
    pub encryption: EncryptionSettings,

    /// Backup retention policy
    #[serde(default)]
    pub backup_retention: BackupRetention,

    /// Default currency symbol
    #[serde(default = "default_currency")]
    pub currency_symbol: String,

    /// Date format preference (strftime format)
    #[serde(default = "default_date_format")]
    pub date_format: String,

    /// First day of week (0 = Sunday, 1 = Monday)
    #[serde(default = "default_first_day_of_week")]
    pub first_day_of_week: u8,

    /// Whether initial setup has been completed
    #[serde(default)]
    pub setup_completed: bool,
}

fn default_schema_version() -> u32 {
    1
}

fn default_currency() -> String {
    "$".to_string()
}

fn default_date_format() -> String {
    "%Y-%m-%d".to_string()
}

fn default_first_day_of_week() -> u8 {
    0 // Sunday
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            budget_period_type: BudgetPeriodType::default(),
            encryption_enabled: false,
            encryption: EncryptionSettings::default(),
            backup_retention: BackupRetention::default(),
            currency_symbol: default_currency(),
            date_format: default_date_format(),
            first_day_of_week: default_first_day_of_week(),
            setup_completed: false,
        }
    }
}

impl Settings {
    /// Check if encryption is enabled (using new encryption field)
    pub fn is_encryption_enabled(&self) -> bool {
        self.encryption.enabled || self.encryption_enabled
    }

    /// Load settings from disk, or create default settings if file doesn't exist
    pub fn load_or_create(paths: &EnvelopePaths) -> Result<Self, EnvelopeError> {
        let settings_path = paths.settings_file();

        if settings_path.exists() {
            let contents = std::fs::read_to_string(&settings_path).map_err(|e| {
                EnvelopeError::Io(format!("Failed to read settings file: {}", e))
            })?;

            let settings: Settings = serde_json::from_str(&contents).map_err(|e| {
                EnvelopeError::Config(format!("Failed to parse settings file: {}", e))
            })?;

            Ok(settings)
        } else {
            // Create default settings
            let settings = Settings::default();
            // Don't save yet - let caller decide when to persist
            Ok(settings)
        }
    }

    /// Save settings to disk
    pub fn save(&self, paths: &EnvelopePaths) -> Result<(), EnvelopeError> {
        // Ensure the config directory exists
        paths.ensure_directories()?;

        let settings_path = paths.settings_file();
        let contents = serde_json::to_string_pretty(self).map_err(|e| {
            EnvelopeError::Config(format!("Failed to serialize settings: {}", e))
        })?;

        std::fs::write(&settings_path, contents).map_err(|e| {
            EnvelopeError::Io(format!("Failed to write settings file: {}", e))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_settings() {
        let settings = Settings::default();
        assert_eq!(settings.budget_period_type, BudgetPeriodType::Monthly);
        assert!(!settings.encryption_enabled);
        assert_eq!(settings.backup_retention.daily_count, 30);
        assert_eq!(settings.backup_retention.monthly_count, 12);
    }

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());

        let mut settings = Settings::default();
        settings.budget_period_type = BudgetPeriodType::Weekly;
        settings.encryption_enabled = true;

        settings.save(&paths).unwrap();

        let loaded = Settings::load_or_create(&paths).unwrap();
        assert_eq!(loaded.budget_period_type, BudgetPeriodType::Weekly);
        assert!(loaded.encryption_enabled);
    }

    #[test]
    fn test_serde_round_trip() {
        let settings = Settings::default();
        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(settings.budget_period_type, deserialized.budget_period_type);
    }
}
