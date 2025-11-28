//! YAML Export functionality
//!
//! Exports the complete database to YAML format for human-readable backup.

use crate::error::EnvelopeResult;
use crate::export::json::FullExport;
use crate::storage::Storage;
use std::io::Write;

/// Export the full database to YAML format
pub fn export_full_yaml<W: Write>(storage: &Storage, writer: &mut W) -> EnvelopeResult<()> {
    let export = FullExport::from_storage(storage)?;

    // Add a header comment
    writeln!(writer, "# EnvelopeCLI Full Database Export")
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
    writeln!(writer, "# Generated: {}", export.exported_at)
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
    writeln!(writer, "# App Version: {}", export.app_version)
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
    writeln!(writer, "#").map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
    writeln!(
        writer,
        "# This file can be used to restore your budget data."
    )
    .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
    writeln!(
        writer,
        "# Keep it secure - it contains all your financial data."
    )
    .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;
    writeln!(writer).map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

    // Serialize to YAML
    serde_yaml::to_writer(writer, &export)
        .map_err(|e| crate::error::EnvelopeError::Export(e.to_string()))?;

    Ok(())
}

/// Import from a YAML export
pub fn import_from_yaml(yaml_str: &str) -> EnvelopeResult<FullExport> {
    let export: FullExport = serde_yaml::from_str(yaml_str)
        .map_err(|e| crate::error::EnvelopeError::Import(e.to_string()))?;

    // Validate the import
    export
        .validate()
        .map_err(crate::error::EnvelopeError::Import)?;

    Ok(export)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::paths::EnvelopePaths;
    use crate::models::{Account, AccountType, Category, CategoryGroup};
    use tempfile::TempDir;

    fn create_test_storage() -> (TempDir, Storage) {
        let temp_dir = TempDir::new().unwrap();
        let paths = EnvelopePaths::with_base_dir(temp_dir.path().to_path_buf());
        let mut storage = Storage::new(paths).unwrap();
        storage.load_all().unwrap();
        (temp_dir, storage)
    }

    #[test]
    fn test_yaml_export() {
        let (_temp_dir, storage) = create_test_storage();

        // Create test data
        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account).unwrap();
        storage.accounts.save().unwrap();

        let group = CategoryGroup::new("Test");
        storage.categories.upsert_group(group.clone()).unwrap();
        let cat = Category::new("Groceries", group.id);
        storage.categories.upsert_category(cat).unwrap();
        storage.categories.save().unwrap();

        // Export to YAML
        let mut yaml_output = Vec::new();
        export_full_yaml(&storage, &mut yaml_output).unwrap();

        let yaml_string = String::from_utf8(yaml_output).unwrap();

        // Verify header comments
        assert!(yaml_string.contains("# EnvelopeCLI Full Database Export"));

        // Verify data
        assert!(yaml_string.contains("Checking"));
        assert!(yaml_string.contains("Groceries"));
    }

    #[test]
    fn test_yaml_roundtrip() {
        let (_temp_dir, storage) = create_test_storage();

        // Create test data
        let account = Account::new("Checking", AccountType::Checking);
        storage.accounts.upsert(account).unwrap();
        storage.accounts.save().unwrap();

        // Export to YAML
        let mut yaml_output = Vec::new();
        export_full_yaml(&storage, &mut yaml_output).unwrap();

        let yaml_string = String::from_utf8(yaml_output).unwrap();

        // Skip the comment lines for parsing
        let yaml_content: String = yaml_string
            .lines()
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>()
            .join("\n");

        // Import back
        let imported = import_from_yaml(&yaml_content).unwrap();

        assert_eq!(imported.accounts.len(), 1);
        assert_eq!(imported.accounts[0].name, "Checking");
    }
}
