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
}
