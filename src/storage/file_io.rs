//! File I/O utilities with atomic writes
//!
//! Provides safe file operations that won't corrupt data on failure.

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;

use serde::{de::DeserializeOwned, Serialize};

use crate::error::EnvelopeError;

/// Read JSON from a file, returning a default value if file doesn't exist
pub fn read_json<T, P>(path: P) -> Result<T, EnvelopeError>
where
    T: DeserializeOwned + Default,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        return Ok(T::default());
    }

    let file = File::open(path)
        .map_err(|e| EnvelopeError::Storage(format!("Failed to open {}: {}", path.display(), e)))?;

    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
        .map_err(|e| EnvelopeError::Storage(format!("Failed to parse {}: {}", path.display(), e)))
}

/// Read JSON from a file, returning an error if file doesn't exist
pub fn read_json_required<T, P>(path: P) -> Result<T, EnvelopeError>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        return Err(EnvelopeError::Storage(format!(
            "File not found: {}",
            path.display()
        )));
    }

    let file = File::open(path)
        .map_err(|e| EnvelopeError::Storage(format!("Failed to open {}: {}", path.display(), e)))?;

    let reader = BufReader::new(file);
    serde_json::from_reader(reader)
        .map_err(|e| EnvelopeError::Storage(format!("Failed to parse {}: {}", path.display(), e)))
}

/// Write JSON to a file atomically (write to temp, then rename)
///
/// This ensures that the file is either completely written or not modified at all,
/// preventing corruption on crashes or power failures.
pub fn write_json_atomic<T, P>(path: P, data: &T) -> Result<(), EnvelopeError>
where
    T: Serialize,
    P: AsRef<Path>,
{
    let path = path.as_ref();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            EnvelopeError::Storage(format!(
                "Failed to create directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }

    // Create temp file in same directory (important for atomic rename)
    let temp_path = path.with_extension("json.tmp");

    // Write to temp file
    let file = File::create(&temp_path)
        .map_err(|e| EnvelopeError::Storage(format!("Failed to create temp file: {}", e)))?;

    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data)
        .map_err(|e| EnvelopeError::Storage(format!("Failed to serialize data: {}", e)))?;

    writer
        .flush()
        .map_err(|e| EnvelopeError::Storage(format!("Failed to flush data: {}", e)))?;

    // Sync to disk before rename
    writer
        .get_ref()
        .sync_all()
        .map_err(|e| EnvelopeError::Storage(format!("Failed to sync data: {}", e)))?;

    // Atomic rename
    fs::rename(&temp_path, path).map_err(|e| {
        // Try to clean up temp file if rename fails
        let _ = fs::remove_file(&temp_path);
        EnvelopeError::Storage(format!("Failed to rename temp file: {}", e))
    })?;

    Ok(())
}

/// Check if a JSON file exists and is valid
pub fn json_file_valid<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();
    if !path.exists() {
        return false;
    }

    // Try to parse as JSON
    if let Ok(file) = File::open(path) {
        let reader = BufReader::new(file);
        serde_json::from_reader::<_, serde_json::Value>(reader).is_ok()
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::TempDir;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
    struct TestData {
        name: String,
        value: i32,
    }

    #[test]
    fn test_read_nonexistent_returns_default() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nonexistent.json");

        let data: TestData = read_json(&path).unwrap();
        assert_eq!(data, TestData::default());
    }

    #[test]
    fn test_write_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.json");

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        write_json_atomic(&path, &data).unwrap();
        assert!(path.exists());

        let loaded: TestData = read_json(&path).unwrap();
        assert_eq!(data, loaded);
    }

    #[test]
    fn test_atomic_write_no_temp_file_left() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.json");
        let temp_path = temp_dir.path().join("test.json.tmp");

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        write_json_atomic(&path, &data).unwrap();

        assert!(path.exists());
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_write_creates_parent_directories() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("nested").join("dir").join("test.json");

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        write_json_atomic(&path, &data).unwrap();
        assert!(path.exists());
    }

    #[test]
    fn test_json_file_valid() {
        let temp_dir = TempDir::new().unwrap();
        let valid_path = temp_dir.path().join("valid.json");
        let invalid_path = temp_dir.path().join("invalid.json");
        let nonexistent_path = temp_dir.path().join("nonexistent.json");

        // Create valid JSON
        fs::write(&valid_path, r#"{"name": "test"}"#).unwrap();
        assert!(json_file_valid(&valid_path));

        // Create invalid JSON
        fs::write(&invalid_path, "not json at all").unwrap();
        assert!(!json_file_valid(&invalid_path));

        // Nonexistent
        assert!(!json_file_valid(&nonexistent_path));
    }

    #[test]
    fn test_read_json_required() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("test.json");

        // Should fail for nonexistent
        assert!(read_json_required::<TestData, _>(&path).is_err());

        // Write and then read should work
        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };
        write_json_atomic(&path, &data).unwrap();

        let loaded: TestData = read_json_required(&path).unwrap();
        assert_eq!(data, loaded);
    }
}
