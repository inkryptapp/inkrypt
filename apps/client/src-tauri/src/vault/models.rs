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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use uuid::Uuid;

    #[test]
    fn test_vault_registry_new() {
        let registry = VaultRegistry::new();
        assert!(registry.vaults.is_empty());
    }

    #[test]
    fn test_vault_registry_insert_and_get() {
        let mut registry = VaultRegistry::new();
        let vault_id = Uuid::now_v7();
        let path = PathBuf::from("/test/path");

        registry.insert_vault(vault_id, path.clone());

        assert_eq!(registry.get_vault_path(&vault_id), Some(&path));
        assert_eq!(registry.vaults.len(), 1);
    }

    #[test]
    fn test_vault_registry_insert_overwrites() {
        let mut registry = VaultRegistry::new();
        let vault_id = Uuid::now_v7();
        let path1 = PathBuf::from("/test/path1");
        let path2 = PathBuf::from("/test/path2");

        registry.insert_vault(vault_id, path1);
        registry.insert_vault(vault_id, path2.clone());

        assert_eq!(registry.get_vault_path(&vault_id), Some(&path2));
        assert_eq!(registry.vaults.len(), 1);
    }

    #[test]
    fn test_vault_registry_remove() {
        let mut registry = VaultRegistry::new();
        let vault_id = Uuid::now_v7();
        let path = PathBuf::from("/test/path");

        registry.insert_vault(vault_id, path);
        registry.remove_vault(&vault_id);

        assert_eq!(registry.get_vault_path(&vault_id), None);
        assert!(registry.vaults.is_empty());
    }

    #[test]
    fn test_vault_registry_multiple_vaults() {
        let mut registry = VaultRegistry::new();
        let vault_id1 = Uuid::now_v7();
        let vault_id2 = Uuid::now_v7();
        let path1 = PathBuf::from("/test/path1");
        let path2 = PathBuf::from("/test/path2");

        registry.insert_vault(vault_id1, path1.clone());
        registry.insert_vault(vault_id2, path2.clone());

        assert_eq!(registry.get_vault_path(&vault_id1), Some(&path1));
        assert_eq!(registry.get_vault_path(&vault_id2), Some(&path2));
        assert_eq!(registry.vaults.len(), 2);
    }

    #[test]
    fn test_vault_serialization() {
        use chrono::Utc;

        let vault = Vault {
            id: Uuid::now_v7(),
            name: "Test Vault".to_string(),
            path: PathBuf::from("/test/vault"),
            version: 1,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&vault).unwrap();
        let deserialized: Vault = serde_json::from_str(&json).unwrap();

        assert_eq!(vault.id, deserialized.id);
        assert_eq!(vault.name, deserialized.name);
        assert_eq!(vault.path, deserialized.path);
        assert_eq!(vault.version, deserialized.version);
    }

    #[test]
    fn test_entry_serialization() {
        use chrono::Utc;

        let entry = Entry {
            name: "test.md".to_string(),
            path: "notes/test.md".to_string(),
            entry_type: EntryType::Note,
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        };

        let json = serde_json::to_string(&entry).unwrap();
        let deserialized: Entry = serde_json::from_str(&json).unwrap();

        assert_eq!(entry.name, deserialized.name);
        assert_eq!(entry.path, deserialized.path);
        assert_eq!(entry.entry_type, deserialized.entry_type);
    }

    #[test]
    fn test_vault_registry_serialization() {
        let mut registry = VaultRegistry::new();
        let vault_id = Uuid::now_v7();
        let path = PathBuf::from("/test/path");

        registry.insert_vault(vault_id, path.clone());

        let json = serde_json::to_string(&registry).unwrap();
        let deserialized: VaultRegistry = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.get_vault_path(&vault_id), Some(&path));
    }
}
