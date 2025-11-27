//! Payee repository for JSON storage
//!
//! Manages loading and saving payees to payees.json

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::EnvelopeError;
use crate::models::{Payee, PayeeId};

use super::file_io::{read_json, write_json_atomic};

/// Serializable payee data structure
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct PayeeData {
    payees: Vec<Payee>,
}

/// Repository for payee persistence
pub struct PayeeRepository {
    path: PathBuf,
    data: RwLock<HashMap<PayeeId, Payee>>,
    /// Index: normalized name -> payee_id
    by_name: RwLock<HashMap<String, PayeeId>>,
}

impl PayeeRepository {
    /// Create a new payee repository
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            data: RwLock::new(HashMap::new()),
            by_name: RwLock::new(HashMap::new()),
        }
    }

    /// Load payees from disk
    pub fn load(&self) -> Result<(), EnvelopeError> {
        let file_data: PayeeData = read_json(&self.path)?;

        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_name = self.by_name.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        data.clear();
        by_name.clear();

        for payee in file_data.payees {
            let normalized = Payee::normalize_name(&payee.name);
            by_name.insert(normalized, payee.id);
            data.insert(payee.id, payee);
        }

        Ok(())
    }

    /// Save payees to disk
    pub fn save(&self) -> Result<(), EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let mut payees: Vec<_> = data.values().cloned().collect();
        payees.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

        let file_data = PayeeData { payees };
        write_json_atomic(&self.path, &file_data)
    }

    /// Get a payee by ID
    pub fn get(&self, id: PayeeId) -> Result<Option<Payee>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(data.get(&id).cloned())
    }

    /// Get all payees
    pub fn get_all(&self) -> Result<Vec<Payee>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let mut payees: Vec<_> = data.values().cloned().collect();
        payees.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(payees)
    }

    /// Get a payee by exact name (case-insensitive)
    pub fn get_by_name(&self, name: &str) -> Result<Option<Payee>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;
        let by_name = self.by_name.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let normalized = Payee::normalize_name(name);
        if let Some(&id) = by_name.get(&normalized) {
            Ok(data.get(&id).cloned())
        } else {
            Ok(None)
        }
    }

    /// Find payees matching a query (fuzzy search)
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Payee>, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;

        let mut scored: Vec<_> = data
            .values()
            .map(|p| (p.clone(), p.similarity_score(query)))
            .filter(|(_, score)| *score > 0.3)
            .collect();

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(scored.into_iter().take(limit).map(|(p, _)| p).collect())
    }

    /// Get or create a payee by name
    pub fn get_or_create(&self, name: &str) -> Result<Payee, EnvelopeError> {
        if let Some(payee) = self.get_by_name(name)? {
            return Ok(payee);
        }

        let payee = Payee::new(name);
        self.upsert(payee.clone())?;
        Ok(payee)
    }

    /// Insert or update a payee
    pub fn upsert(&self, payee: Payee) -> Result<(), EnvelopeError> {
        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_name = self.by_name.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        // Remove old name index if updating
        if let Some(old) = data.get(&payee.id) {
            let old_normalized = Payee::normalize_name(&old.name);
            by_name.remove(&old_normalized);
        }

        // Add new name index
        let normalized = Payee::normalize_name(&payee.name);
        by_name.insert(normalized, payee.id);

        data.insert(payee.id, payee);
        Ok(())
    }

    /// Delete a payee
    pub fn delete(&self, id: PayeeId) -> Result<bool, EnvelopeError> {
        let mut data = self.data.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;
        let mut by_name = self.by_name.write().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e))
        })?;

        if let Some(payee) = data.remove(&id) {
            let normalized = Payee::normalize_name(&payee.name);
            by_name.remove(&normalized);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Count payees
    pub fn count(&self) -> Result<usize, EnvelopeError> {
        let data = self.data.read().map_err(|e| {
            EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e))
        })?;
        Ok(data.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_repo() -> (TempDir, PayeeRepository) {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("payees.json");
        let repo = PayeeRepository::new(path);
        (temp_dir, repo)
    }

    #[test]
    fn test_empty_load() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();
        assert_eq!(repo.count().unwrap(), 0);
    }

    #[test]
    fn test_upsert_and_get() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let payee = Payee::new("Test Store");
        let id = payee.id;

        repo.upsert(payee).unwrap();

        let retrieved = repo.get(id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Store");
    }

    #[test]
    fn test_get_by_name() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        repo.upsert(Payee::new("Grocery Store")).unwrap();

        // Case insensitive
        let found = repo.get_by_name("grocery store").unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Grocery Store");

        let not_found = repo.get_by_name("other store").unwrap();
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_or_create() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        // Should create
        let p1 = repo.get_or_create("New Store").unwrap();
        assert_eq!(p1.name, "New Store");
        assert_eq!(repo.count().unwrap(), 1);

        // Should return existing
        let p2 = repo.get_or_create("new store").unwrap();
        assert_eq!(p1.id, p2.id);
        assert_eq!(repo.count().unwrap(), 1);
    }

    #[test]
    fn test_search() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        repo.upsert(Payee::new("Grocery Store")).unwrap();
        repo.upsert(Payee::new("Gas Station")).unwrap();
        repo.upsert(Payee::new("Restaurant")).unwrap();

        let results = repo.search("groc", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "Grocery Store");

        let results2 = repo.search("st", 10).unwrap();
        // Should match "Store" and "Station"
        assert!(results2.len() >= 2);
    }

    #[test]
    fn test_save_and_reload() {
        let (temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let payee = Payee::new("Test Store");
        let id = payee.id;

        repo.upsert(payee).unwrap();
        repo.save().unwrap();

        // Create new repo and load
        let path = temp_dir.path().join("payees.json");
        let repo2 = PayeeRepository::new(path);
        repo2.load().unwrap();

        let retrieved = repo2.get(id).unwrap().unwrap();
        assert_eq!(retrieved.name, "Test Store");
    }

    #[test]
    fn test_delete() {
        let (_temp_dir, repo) = create_test_repo();
        repo.load().unwrap();

        let payee = Payee::new("Test Store");
        let id = payee.id;

        repo.upsert(payee).unwrap();
        assert_eq!(repo.count().unwrap(), 1);

        repo.delete(id).unwrap();
        assert_eq!(repo.count().unwrap(), 0);

        // Name index should also be cleared
        let not_found = repo.get_by_name("Test Store").unwrap();
        assert!(not_found.is_none());
    }
}
