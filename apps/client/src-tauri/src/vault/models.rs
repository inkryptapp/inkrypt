use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vault {
    pub id: Uuid,
    pub name: String,
    pub path: PathBuf,
    pub version: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMetadata {
    pub id: Uuid,
    pub version: u32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EntryType {
    Directory,
    Note,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileSystemEvent {
    pub event_type: FileEventType,
    pub path: String,
    pub vault_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FileEventType {
    Create,
    Modify,
    Delete,
    Rename,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultRegistry {
    pub vaults: HashMap<Uuid, PathBuf>,
}

impl VaultRegistry {
    pub fn new() -> Self {
        Self {
            vaults: HashMap::new(),
        }
    }

    pub fn insert_vault(&mut self, id: Uuid, path: PathBuf) {
        self.vaults.insert(id, path);
    }

    pub fn remove_vault(&mut self, id: &Uuid) {
        self.vaults.remove(id);
    }

    pub fn get_vault_path(&self, id: &Uuid) -> Option<&PathBuf> {
        self.vaults.get(id)
    }

    pub fn get_vaults(&self) -> &HashMap<Uuid, PathBuf> {
        &self.vaults
    }
}

impl Default for VaultRegistry {
    fn default() -> Self {
        Self::new()
    }
}
