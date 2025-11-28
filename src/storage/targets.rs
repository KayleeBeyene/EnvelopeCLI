//! Budget target repository for JSON storage

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

use crate::error::EnvelopeError;
use crate::models::{BudgetTarget, BudgetTargetId, CategoryId};

use super::file_io::{read_json, write_json_atomic};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct TargetData {
    #[serde(default)]
    targets: Vec<BudgetTarget>,
}

pub struct TargetRepository {
    path: PathBuf,
    targets: RwLock<HashMap<BudgetTargetId, BudgetTarget>>,
}

impl TargetRepository {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            targets: RwLock::new(HashMap::new()),
        }
    }

    pub fn load(&self) -> Result<(), EnvelopeError> {
        let file_data: TargetData = read_json(&self.path)?;

        let mut targets = self
            .targets
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        targets.clear();
        for target in file_data.targets {
            targets.insert(target.id, target);
        }

        Ok(())
    }

    pub fn save(&self) -> Result<(), EnvelopeError> {
        let targets = self
            .targets
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut target_list: Vec<_> = targets.values().cloned().collect();
        target_list.sort_by(|a, b| a.created_at.cmp(&b.created_at));

        let file_data = TargetData {
            targets: target_list,
        };

        write_json_atomic(&self.path, &file_data)
    }

    pub fn get(&self, id: BudgetTargetId) -> Result<Option<BudgetTarget>, EnvelopeError> {
        let targets = self
            .targets
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        Ok(targets.get(&id).cloned())
    }

    pub fn get_for_category(
        &self,
        category_id: CategoryId,
    ) -> Result<Option<BudgetTarget>, EnvelopeError> {
        let targets = self
            .targets
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        Ok(targets
            .values()
            .find(|t| t.category_id == category_id && t.active)
            .cloned())
    }

    pub fn get_all_active(&self) -> Result<Vec<BudgetTarget>, EnvelopeError> {
        let targets = self
            .targets
            .read()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire read lock: {}", e)))?;

        let mut list: Vec<_> = targets.values().filter(|t| t.active).cloned().collect();
        list.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        Ok(list)
    }

    pub fn upsert(&self, target: BudgetTarget) -> Result<(), EnvelopeError> {
        let mut targets = self
            .targets
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        targets.insert(target.id, target);
        Ok(())
    }

    pub fn delete(&self, id: BudgetTargetId) -> Result<bool, EnvelopeError> {
        let mut targets = self
            .targets
            .write()
            .map_err(|e| EnvelopeError::Storage(format!("Failed to acquire write lock: {}", e)))?;

        Ok(targets.remove(&id).is_some())
    }
}
