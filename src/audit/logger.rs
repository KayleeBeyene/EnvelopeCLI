//! Audit logger for append-only audit log
//!
//! Provides the AuditLogger struct that writes audit entries to a log file.
//! Each entry is written as a single JSON line and flushed immediately.

use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

use crate::error::{EnvelopeError, EnvelopeResult};

use super::entry::AuditEntry;

/// Handles writing audit entries to the audit log file
///
/// The log file uses a line-delimited JSON format (JSONL) where each line
/// is a complete JSON object representing one audit entry.
pub struct AuditLogger {
    /// Path to the audit log file
    log_path: PathBuf,
}

impl AuditLogger {
    /// Create a new AuditLogger that writes to the specified path
    pub fn new(log_path: PathBuf) -> Self {
        Self { log_path }
    }

    /// Log an audit entry
    ///
    /// Appends the entry as a JSON line to the audit log file.
    /// Each write is flushed immediately to ensure durability.
    pub fn log(&self, entry: &AuditEntry) -> EnvelopeResult<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| EnvelopeError::Io(format!("Failed to open audit log: {}", e)))?;

        let json = serde_json::to_string(entry)
            .map_err(|e| EnvelopeError::Json(format!("Failed to serialize audit entry: {}", e)))?;

        writeln!(file, "{}", json)
            .map_err(|e| EnvelopeError::Io(format!("Failed to write audit entry: {}", e)))?;

        file.flush()
            .map_err(|e| EnvelopeError::Io(format!("Failed to flush audit log: {}", e)))?;

        Ok(())
    }

    /// Log multiple audit entries atomically
    ///
    /// Writes all entries and flushes once at the end.
    pub fn log_batch(&self, entries: &[AuditEntry]) -> EnvelopeResult<()> {
        if entries.is_empty() {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|e| EnvelopeError::Io(format!("Failed to open audit log: {}", e)))?;

        for entry in entries {
            let json = serde_json::to_string(entry)
                .map_err(|e| EnvelopeError::Json(format!("Failed to serialize audit entry: {}", e)))?;

            writeln!(file, "{}", json)
                .map_err(|e| EnvelopeError::Io(format!("Failed to write audit entry: {}", e)))?;
        }

        file.flush()
            .map_err(|e| EnvelopeError::Io(format!("Failed to flush audit log: {}", e)))?;

        Ok(())
    }

    /// Read all audit entries from the log file
    ///
    /// Returns entries in chronological order (oldest first).
    pub fn read_all(&self) -> EnvelopeResult<Vec<AuditEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let file = File::open(&self.log_path)
            .map_err(|e| EnvelopeError::Io(format!("Failed to open audit log: {}", e)))?;

        let reader = BufReader::new(file);
        let mut entries = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| {
                EnvelopeError::Io(format!("Failed to read audit log line {}: {}", line_num + 1, e))
            })?;

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            let entry: AuditEntry = serde_json::from_str(&line).map_err(|e| {
                EnvelopeError::Json(format!(
                    "Failed to parse audit entry at line {}: {}",
                    line_num + 1,
                    e
                ))
            })?;

            entries.push(entry);
        }

        Ok(entries)
    }

    /// Read the most recent N entries from the log
    pub fn read_recent(&self, count: usize) -> EnvelopeResult<Vec<AuditEntry>> {
        let all_entries = self.read_all()?;
        let start = all_entries.len().saturating_sub(count);
        Ok(all_entries[start..].to_vec())
    }

    /// Get the number of entries in the audit log
    pub fn entry_count(&self) -> EnvelopeResult<usize> {
        if !self.log_path.exists() {
            return Ok(0);
        }

        let file = File::open(&self.log_path)
            .map_err(|e| EnvelopeError::Io(format!("Failed to open audit log: {}", e)))?;

        let reader = BufReader::new(file);
        let count = reader.lines().filter(|l| l.is_ok()).count();

        Ok(count)
    }

    /// Check if the audit log file exists
    pub fn exists(&self) -> bool {
        self.log_path.exists()
    }

    /// Get the path to the audit log file
    pub fn path(&self) -> &PathBuf {
        &self.log_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audit::entry::{EntityType, Operation};
    use serde_json::json;
    use tempfile::TempDir;

    fn create_test_logger() -> (AuditLogger, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let logger = AuditLogger::new(log_path);
        (logger, temp_dir)
    }

    fn create_test_entry() -> AuditEntry {
        AuditEntry::create(
            EntityType::Account,
            "acc-12345678",
            Some("Test Account".to_string()),
            &json!({"name": "Test Account", "balance": 1000}),
        )
    }

    #[test]
    fn test_log_and_read() {
        let (logger, _temp) = create_test_logger();
        let entry = create_test_entry();

        // Log the entry
        logger.log(&entry).unwrap();

        // Read it back
        let entries = logger.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, Operation::Create);
        assert_eq!(entries[0].entity_type, EntityType::Account);
    }

    #[test]
    fn test_multiple_entries() {
        let (logger, _temp) = create_test_logger();

        // Log multiple entries
        for i in 0..5 {
            let entry = AuditEntry::create(
                EntityType::Account,
                format!("acc-{}", i),
                Some(format!("Account {}", i)),
                &json!({"name": format!("Account {}", i)}),
            );
            logger.log(&entry).unwrap();
        }

        // Verify count
        assert_eq!(logger.entry_count().unwrap(), 5);

        // Verify all entries readable
        let entries = logger.read_all().unwrap();
        assert_eq!(entries.len(), 5);
    }

    #[test]
    fn test_log_batch() {
        let (logger, _temp) = create_test_logger();

        let entries: Vec<AuditEntry> = (0..3)
            .map(|i| {
                AuditEntry::create(
                    EntityType::Account,
                    format!("acc-{}", i),
                    None,
                    &json!({"id": i}),
                )
            })
            .collect();

        logger.log_batch(&entries).unwrap();

        let read_entries = logger.read_all().unwrap();
        assert_eq!(read_entries.len(), 3);
    }

    #[test]
    fn test_read_recent() {
        let (logger, _temp) = create_test_logger();

        // Log 10 entries
        for i in 0..10 {
            let entry = AuditEntry::create(
                EntityType::Account,
                format!("acc-{}", i),
                None,
                &json!({"index": i}),
            );
            logger.log(&entry).unwrap();
        }

        // Read last 3
        let recent = logger.read_recent(3).unwrap();
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].entity_id, "acc-7");
        assert_eq!(recent[1].entity_id, "acc-8");
        assert_eq!(recent[2].entity_id, "acc-9");
    }

    #[test]
    fn test_empty_log() {
        let (logger, _temp) = create_test_logger();

        assert!(!logger.exists());
        assert_eq!(logger.entry_count().unwrap(), 0);
        assert!(logger.read_all().unwrap().is_empty());
    }

    #[test]
    fn test_update_entry_logged() {
        let (logger, _temp) = create_test_logger();

        let before = json!({"name": "Old Name", "balance": 100});
        let after = json!({"name": "New Name", "balance": 100});

        let entry = AuditEntry::update(
            EntityType::Account,
            "acc-12345678",
            Some("Account".to_string()),
            &before,
            &after,
            Some("name: \"Old Name\" -> \"New Name\"".to_string()),
        );

        logger.log(&entry).unwrap();

        let entries = logger.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, Operation::Update);
        assert!(entries[0].before.is_some());
        assert!(entries[0].after.is_some());
    }

    #[test]
    fn test_delete_entry_logged() {
        let (logger, _temp) = create_test_logger();

        let entity = json!({"name": "Deleted Account"});
        let entry = AuditEntry::delete(
            EntityType::Account,
            "acc-12345678",
            Some("Deleted Account".to_string()),
            &entity,
        );

        logger.log(&entry).unwrap();

        let entries = logger.read_all().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].operation, Operation::Delete);
        assert!(entries[0].before.is_some());
        assert!(entries[0].after.is_none());
    }

    #[test]
    fn test_survives_crash_simulation() {
        let (logger, temp) = create_test_logger();

        // Log entry
        let entry = create_test_entry();
        logger.log(&entry).unwrap();

        // Create a new logger pointing to the same file (simulating restart)
        let logger2 = AuditLogger::new(temp.path().join("audit.log"));

        // Should still be readable
        let entries = logger2.read_all().unwrap();
        assert_eq!(entries.len(), 1);
    }
}
