use crate::vault::models::*;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct VaultManager {
    registry: Arc<RwLock<VaultRegistry>>,
    registry_path: PathBuf,
}

impl VaultManager {
    pub fn new() -> Self {
        let data_dir = dirs::data_dir().expect("Failed to get data directory");
        let app_data_dir = data_dir.join("inkrypt");
        std::fs::create_dir_all(&app_data_dir).expect("Failed to create app data directory");

        let registry_path = app_data_dir.join("vaults.json");
        let registry = if registry_path.exists() {
            let data =
                std::fs::read_to_string(&registry_path).expect("Failed to read registry file");
            serde_json::from_str(&data).expect("Failed to parse registry file")
        } else {
            VaultRegistry::new()
        };

        Self {
            registry: Arc::new(RwLock::new(registry)),
            registry_path,
        }
    }

    #[cfg(test)]
    pub fn new_for_testing(registry_path: PathBuf) -> Self {
        let registry = VaultRegistry::new();
        Self {
            registry: Arc::new(RwLock::new(registry)),
            registry_path,
        }
    }

    async fn save_registry(&self) -> Result<()> {
        let registry = self.registry.read().await;
        let data = serde_json::to_string_pretty(&*registry)?;
        fs::write(&self.registry_path, data)?;
        Ok(())
    }

    pub async fn create_vault(&self, root_directory: &Path, name: &str) -> Result<Vault> {
        let vault_path = root_directory.join(name);

        if vault_path.exists() {
            return Err(anyhow!("A directory with this name already exists"));
        }

        // Create vault directory structure
        fs::create_dir_all(&vault_path)?;
        let inkrypt_dir = vault_path.join(".inkrypt");
        fs::create_dir_all(&inkrypt_dir)?;

        // Make the .inkrypt directory hidden on Windows
        #[cfg(windows)]
        {
            Command::new("attrib")
                .args(["+h", inkrypt_dir.to_str().unwrap()])
                .status()
                .expect("failed to set hidden attribute");
        }

        // Create vault metadata
        let now = Utc::now();
        let vault_metadata = VaultMetadata {
            id: Uuid::now_v7(),
            version: 0,
            created_at: now,
        };

        // Save vault.json
        let vault_json_path = inkrypt_dir.join("vault.json");
        let metadata_json = serde_json::to_string_pretty(&vault_metadata)?;
        fs::write(&vault_json_path, metadata_json)?;

        // Create vault object
        let vault = Vault {
            id: vault_metadata.id,
            name: name.to_string(),
            path: vault_path.clone(),
            version: vault_metadata.version,
            created_at: vault_metadata.created_at,
            updated_at: now,
        };

        // Add to vault registry
        {
            let mut registry = self.registry.write().await;
            registry.insert_vault(vault_metadata.id, vault_path);
        }
        self.save_registry().await?;

        Ok(vault)
    }

    pub async fn open_vault(&self, vault_path: &Path) -> Result<Vault> {
        let vault = self.load_vault_from_path(vault_path).await?;

        // Add or update in vault registry
        {
            let mut registry = self.registry.write().await;
            registry.insert_vault(vault.id, vault_path.to_path_buf());
        }
        self.save_registry().await?;

        Ok(vault)
    }

    async fn load_vault_from_path(&self, vault_path: &Path) -> Result<Vault> {
        let inkrypt_dir = vault_path.join(".inkrypt");
        let vault_json_path = inkrypt_dir.join("vault.json");

        if !vault_json_path.exists() {
            return Err(anyhow!("Not a valid vault: vault.json not found"));
        }

        let metadata_json = fs::read_to_string(&vault_json_path)?;
        let metadata: VaultMetadata = serde_json::from_str(&metadata_json)?;

        let vault_name = vault_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unnamed Vault")
            .to_string();

        Ok(Vault {
            id: metadata.id,
            name: vault_name,
            path: vault_path.to_path_buf(),
            version: metadata.version,
            created_at: metadata.created_at,
            updated_at: Utc::now(),
        })
    }

    pub async fn list_vaults(&self) -> Result<Vec<Vault>> {
        let registry = self.registry.read().await;
        let mut vaults = Vec::new();

        for (id, path) in registry.get_vaults() {
            match self.load_vault_from_path(path).await {
                Ok(vault) => {
                    // Verify that the loaded vault has the expected ID
                    if vault.id == *id {
                        vaults.push(vault);
                    } else {
                        // ID mismatch, this vault has been replaced or corrupted
                        continue;
                    }
                }
                Err(_) => {
                    // Skip invalid vaults but don't fail the entire operation
                    continue;
                }
            }
        }

        Ok(vaults)
    }

    pub async fn delete_vault(&self, vault_id: &Uuid) -> Result<()> {
        // Get the vault path from registry
        let vault_path = {
            let registry = self.registry.read().await;
            registry.get_vault_path(vault_id).cloned()
        };

        if let Some(path) = vault_path {
            // Remove from file system
            if path.exists() {
                fs::remove_dir_all(&path)?;
            }

            // Remove from registry
            {
                let mut registry = self.registry.write().await;
                registry.remove_vault(vault_id);
            }
            self.save_registry().await?;
        }

        Ok(())
    }

    pub async fn rename_vault(&self, vault_id: &Uuid, new_name: &str) -> Result<Vault> {
        // Find the vault by ID
        let vault = self.find_vault_by_id(vault_id).await?;

        let parent = vault
            .path
            .parent()
            .ok_or_else(|| anyhow!("Cannot get parent directory"))?;
        let new_path = parent.join(new_name);

        if new_path.exists() {
            return Err(anyhow!("A directory with this name already exists"));
        }

        // Rename the directory in the file system
        fs::rename(&vault.path, &new_path)?;

        // Update vault registry with new path
        {
            let mut registry = self.registry.write().await;
            registry.insert_vault(vault.id, new_path.clone());
        }
        self.save_registry().await?;

        // Return updated vault
        let updated_vault = Vault {
            id: vault.id,
            name: new_name.to_string(),
            path: new_path,
            version: vault.version,
            created_at: vault.created_at,
            updated_at: Utc::now(),
        };

        Ok(updated_vault)
    }

    async fn find_vault_by_id(&self, vault_id: &Uuid) -> Result<Vault> {
        let registry = self.registry.read().await;

        if let Some(path) = registry.get_vault_path(vault_id) {
            self.load_vault_from_path(path).await
        } else {
            Err(anyhow!("Vault not found"))
        }
    }

    pub async fn create_directory(&self, vault_id: &Uuid, directory_path: &str) -> Result<()> {
        let vault = self.find_vault_by_id(vault_id).await?;
        let full_path = vault.path.join(directory_path);
        fs::create_dir_all(full_path)?;
        Ok(())
    }

    pub async fn create_note(&self, vault_id: &Uuid, note_path: &str) -> Result<()> {
        let vault = self.find_vault_by_id(vault_id).await?;
        let full_path = vault.path.join(note_path);

        // Ensure parent directory exists
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create empty note file
        fs::write(&full_path, "")?;
        Ok(())
    }

    pub async fn edit_note(&self, vault_id: &Uuid, note_path: &str, content: &str) -> Result<()> {
        let vault = self.find_vault_by_id(vault_id).await?;
        let full_path = vault.path.join(note_path);
        fs::write(&full_path, content)?;
        Ok(())
    }

    pub async fn read_note(&self, vault_id: &Uuid, note_path: &str) -> Result<String> {
        let vault = self.find_vault_by_id(vault_id).await?;
        let full_path = vault.path.join(note_path);
        let content = fs::read_to_string(&full_path)?;
        Ok(content)
    }

    pub async fn delete_entry(&self, vault_id: &Uuid, entry_path: &str) -> Result<()> {
        let vault = self.find_vault_by_id(vault_id).await?;
        let full_path = vault.path.join(entry_path);

        if full_path.is_dir() {
            fs::remove_dir_all(&full_path)?;
        } else if full_path.is_file() {
            fs::remove_file(&full_path)?;
        }

        Ok(())
    }

    pub async fn rename_entry(
        &self,
        vault_id: &Uuid,
        old_path: &str,
        new_path: &str,
    ) -> Result<()> {
        let vault = self.find_vault_by_id(vault_id).await?;
        let old_full_path = vault.path.join(old_path);
        let new_full_path = vault.path.join(new_path);

        // Ensure parent directory exists for new path
        if let Some(parent) = new_full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::rename(&old_full_path, &new_full_path)?;
        Ok(())
    }

    pub async fn list_entries(
        &self,
        vault_id: &Uuid,
        directory_path: Option<&str>,
    ) -> Result<Vec<Entry>> {
        let vault: Vault = self.find_vault_by_id(vault_id).await?;
        let base_path = if let Some(directory) = directory_path {
            vault.path.join(directory)
        } else {
            vault.path.clone()
        };

        let mut entries = Vec::new();

        for entry in fs::read_dir(&base_path)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files and .inkrypt directory
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if file_name.starts_with('.') {
                continue;
            }

            let metadata = entry.metadata()?;
            let relative_path = path
                .strip_prefix(&vault.path)?
                .to_string_lossy()
                .replace('\\', "/");

            let entry_type = if metadata.is_dir() {
                EntryType::Directory
            } else {
                EntryType::Note
            };

            let created_at = metadata
                .created()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
                .flatten()
                .map(|dt| dt.with_timezone(&Utc));

            let updated_at = metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| DateTime::from_timestamp(d.as_secs() as i64, 0))
                .flatten()
                .map(|dt| dt.with_timezone(&Utc));

            entries.push(Entry {
                name: file_name.to_string(),
                path: relative_path,
                entry_type,
                created_at,
                updated_at,
            });
        }

        // Sort: directories first, then alphabetically
        entries.sort_by(|a, b| match (&a.entry_type, &b.entry_type) {
            (EntryType::Directory, EntryType::Note) => std::cmp::Ordering::Less,
            (EntryType::Note, EntryType::Directory) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        });

        Ok(entries)
    }

    #[cfg(test)]
    pub async fn load_registry_for_testing(&self) -> Result<()> {
        if self.registry_path.exists() {
            let data = std::fs::read_to_string(&self.registry_path)?;
            let loaded_registry: VaultRegistry = serde_json::from_str(&data)?;
            let mut registry = self.registry.write().await;
            *registry = loaded_registry;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    use serial_test::serial;

    async fn setup_test_manager() -> (VaultManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        // Create a mock VaultManager with a custom registry path
        let registry_path = temp_dir.path().join("test_vaults.json");
        let manager = VaultManager::new_for_testing(registry_path);

        (manager, temp_dir)
    }

    #[tokio::test]
    #[serial]
    async fn test_create_vault() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault_name = "test_vault";

        let result = manager.create_vault(temp_dir.path(), vault_name).await;
        assert!(result.is_ok());

        let vault = result.unwrap();
        assert_eq!(vault.name, vault_name);
        assert!(vault.path.exists());
        assert!(vault.path.join(".inkrypt").exists());
        assert!(vault.path.join(".inkrypt/vault.json").exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_create_vault_duplicate_name() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault_name = "test_vault";

        // Create first vault
        let result1 = manager.create_vault(temp_dir.path(), vault_name).await;
        assert!(result1.is_ok());

        // Try to create second vault with same name
        let result2 = manager.create_vault(temp_dir.path(), vault_name).await;
        assert!(result2.is_err());
        assert!(result2
            .unwrap_err()
            .to_string()
            .contains("directory with this name already exists"));
    }

    #[tokio::test]
    #[serial]
    async fn test_open_vault() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault_name = "test_vault";

        // First create a vault
        let created_vault = manager
            .create_vault(temp_dir.path(), vault_name)
            .await
            .unwrap();

        // Then open it
        let result = manager.open_vault(&created_vault.path).await;
        assert!(result.is_ok());

        let opened_vault = result.unwrap();
        assert_eq!(opened_vault.id, created_vault.id);
        assert_eq!(opened_vault.name, created_vault.name);
        assert_eq!(opened_vault.path, created_vault.path);
    }

    #[tokio::test]
    #[serial]
    async fn test_open_invalid_vault() {
        let (manager, temp_dir) = setup_test_manager().await;
        let invalid_path = temp_dir.path().join("nonexistent");

        let result = manager.open_vault(&invalid_path).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("vault.json not found"));
    }

    #[tokio::test]
    #[serial]
    async fn test_list_vaults() {
        let (manager, temp_dir) = setup_test_manager().await;

        // Initially empty
        let vaults = manager.list_vaults().await.unwrap();
        assert!(vaults.is_empty());

        // Create some vaults
        let vault1 = manager
            .create_vault(temp_dir.path(), "vault1")
            .await
            .unwrap();
        let vault2 = manager
            .create_vault(temp_dir.path(), "vault2")
            .await
            .unwrap();

        // List should contain both
        let vaults = manager.list_vaults().await.unwrap();
        assert_eq!(vaults.len(), 2);

        let vault_ids: Vec<Uuid> = vaults.iter().map(|v| v.id).collect();
        assert!(vault_ids.contains(&vault1.id));
        assert!(vault_ids.contains(&vault2.id));
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_vault() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "test_vault")
            .await
            .unwrap();

        assert!(vault.path.exists());

        let result = manager.delete_vault(&vault.id).await;
        assert!(result.is_ok());
        assert!(!vault.path.exists());

        // Should not be in registry anymore
        let vaults = manager.list_vaults().await.unwrap();
        assert!(vaults.is_empty());
    }

    #[tokio::test]
    #[serial]
    async fn test_rename_vault() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "old_name")
            .await
            .unwrap();
        let old_path = vault.path.clone();

        let result = manager.rename_vault(&vault.id, "new_name").await;
        assert!(result.is_ok());

        let renamed_vault = result.unwrap();
        assert_eq!(renamed_vault.name, "new_name");
        assert_eq!(renamed_vault.id, vault.id);
        assert!(!old_path.exists());
        assert!(renamed_vault.path.exists());
        assert!(renamed_vault.path.file_name().unwrap() == "new_name");
    }

    #[tokio::test]
    #[serial]
    async fn test_create_directory() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "test_vault")
            .await
            .unwrap();

        let result = manager.create_directory(&vault.id, "test_directory").await;
        assert!(result.is_ok());

        let directory_path = vault.path.join("test_directory");
        assert!(directory_path.exists());
        assert!(directory_path.is_dir());
    }

    #[tokio::test]
    #[serial]
    async fn test_create_note() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "test_vault")
            .await
            .unwrap();

        let result = manager.create_note(&vault.id, "test_note.md").await;
        assert!(result.is_ok());

        let note_path = vault.path.join("test_note.md");
        assert!(note_path.exists());
        assert!(note_path.is_file());
    }

    #[tokio::test]
    #[serial]
    async fn test_edit_and_read_note() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "test_vault")
            .await
            .unwrap();

        let note_content = "# Test Note\n\nThis is a test note.";

        // Create and edit note
        manager
            .create_note(&vault.id, "test_note.md")
            .await
            .unwrap();
        let edit_result = manager
            .edit_note(&vault.id, "test_note.md", note_content)
            .await;
        assert!(edit_result.is_ok());

        // Read note
        let read_result = manager.read_note(&vault.id, "test_note.md").await;
        assert!(read_result.is_ok());
        assert_eq!(read_result.unwrap(), note_content);
    }

    #[tokio::test]
    #[serial]
    async fn test_delete_entry() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "test_vault")
            .await
            .unwrap();

        // Create a note
        manager
            .create_note(&vault.id, "test_note.md")
            .await
            .unwrap();
        let note_path = vault.path.join("test_note.md");
        assert!(note_path.exists());

        // Delete it
        let result = manager.delete_entry(&vault.id, "test_note.md").await;
        assert!(result.is_ok());
        assert!(!note_path.exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_rename_entry() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "test_vault")
            .await
            .unwrap();

        // Create a note
        manager.create_note(&vault.id, "old_name.md").await.unwrap();
        let old_path = vault.path.join("old_name.md");
        let new_path = vault.path.join("new_name.md");

        assert!(old_path.exists());
        assert!(!new_path.exists());

        // Rename it
        let result = manager
            .rename_entry(&vault.id, "old_name.md", "new_name.md")
            .await;
        assert!(result.is_ok());

        assert!(!old_path.exists());
        assert!(new_path.exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_list_entries() {
        let (manager, temp_dir) = setup_test_manager().await;
        let vault = manager
            .create_vault(temp_dir.path(), "test_vault")
            .await
            .unwrap();

        // Create some entries
        manager
            .create_directory(&vault.id, "test_directory")
            .await
            .unwrap();
        manager
            .create_note(&vault.id, "test_note.md")
            .await
            .unwrap();

        let result = manager.list_entries(&vault.id, None).await;
        assert!(result.is_ok());

        let entries = result.unwrap();
        assert_eq!(entries.len(), 2);

        let entry_names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(entry_names.contains(&"test_directory"));
        assert!(entry_names.contains(&"test_note.md"));

        // Check types
        let directory_entry = entries.iter().find(|e| e.name == "test_directory").unwrap();
        let note_entry = entries.iter().find(|e| e.name == "test_note.md").unwrap();

        assert_eq!(directory_entry.entry_type, EntryType::Directory);
        assert_eq!(note_entry.entry_type, EntryType::Note);
    }

    #[tokio::test]
    #[serial]
    async fn test_find_vault_by_id_not_found() {
        let (manager, _temp_dir) = setup_test_manager().await;
        let nonexistent_id = Uuid::now_v7();

        let result = manager.find_vault_by_id(&nonexistent_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Vault not found"));
    }
}
