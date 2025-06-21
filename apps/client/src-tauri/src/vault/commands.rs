use crate::vault::{Entry, Vault, VaultManager, VaultWatcher};
use anyhow::Result;
use std::path::Path;
use tauri::{AppHandle, State};
use tracing::{error, info};
use uuid::Uuid;

#[tauri::command]
pub async fn create_vault(
    name: String,
    root_directory: String,
    manager_state: State<'_, VaultManager>,
) -> Result<Vault, String> {
    let location_path = Path::new(&root_directory);
    manager_state
        .create_vault(location_path, &name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_vaults(manager_state: State<'_, VaultManager>) -> Result<Vec<Vault>, String> {
    manager_state.list_vaults().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn open_vault(
    vault_path: String,
    _app_handle: AppHandle,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<Vault, String> {
    let path = Path::new(&vault_path);
    let vault = manager_state
        .open_vault(path)
        .await
        .map_err(|e| e.to_string())?;

    // Start watching the vault for changes
    if let Err(e) = watcher_state
        .watch_vault(vault.id, vault.path.clone())
        .await
    {
        error!("Failed to start watching vault {}: {}", vault.id, e);
        // Don't fail the entire operation if watcher fails
    } else {
        info!("Started watching vault: {}", vault.name);
    }

    Ok(vault)
}

#[tauri::command]
pub async fn close_vault(
    vault_id: Uuid,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Stop watching the vault
    if let Err(e) = watcher_state.unwatch_vault(&vault_id).await {
        error!("Failed to stop watching vault {}: {}", vault_id, e);
    } else {
        info!("Stopped watching vault: {}", vault_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn delete_vault(
    vault_id: Uuid,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Stop watching the vault first using the existing close_vault logic
    close_vault(vault_id, watcher_state).await?;

    // Delete the vault
    manager_state
        .delete_vault(&vault_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_vault(
    vault_id: Uuid,
    new_name: String,
    manager_state: State<'_, VaultManager>,
) -> Result<(), String> {
    manager_state
        .rename_vault(&vault_id, &new_name)
        .await
        .map(|_| ()) // Convert Result<Vault> to Result<()>
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_entries(
    vault_id: Uuid,
    directory_path: Option<String>,
    manager_state: State<'_, VaultManager>,
) -> Result<Vec<Entry>, String> {
    manager_state
        .list_entries(&vault_id, directory_path.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn read_note(
    vault_id: Uuid,
    note_path: String,
    manager_state: State<'_, VaultManager>,
) -> Result<String, String> {
    manager_state
        .read_note(&vault_id, &note_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn edit_note(
    vault_id: Uuid,
    note_path: String,
    content: String,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Get vault to construct full path
    let vault = manager_state
        .list_vaults()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|v| v.id == vault_id)
        .ok_or("Vault not found")?;

    let full_path = vault.path.join(&note_path);

    // Mark as pending operation to avoid watcher events
    watcher_state.mark_pending_operation(full_path).await;

    manager_state
        .edit_note(&vault_id, &note_path, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_note(
    vault_id: Uuid,
    note_path: String,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Get vault to construct full path
    let vault = manager_state
        .list_vaults()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|v| v.id == vault_id)
        .ok_or("Vault not found")?;

    let full_path = vault.path.join(&note_path);

    // Mark as pending operation to avoid watcher events
    watcher_state.mark_pending_operation(full_path).await;

    manager_state
        .create_note(&vault_id, &note_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn write_file(
    vault_id: Uuid,
    file_path: String,
    content: String,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Get vault to construct full path
    let vault = manager_state
        .list_vaults()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|v| v.id == vault_id)
        .ok_or("Vault not found")?;

    let full_path = vault.path.join(&file_path);

    // Mark as pending operation to avoid watcher events
    watcher_state.mark_pending_operation(full_path).await;

    manager_state
        .edit_note(&vault_id, &file_path, &content)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_directory(
    vault_id: Uuid,
    directory_path: String,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Get vault to construct full path
    let vault = manager_state
        .list_vaults()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|v| v.id == vault_id)
        .ok_or("Vault not found")?;

    let full_path = vault.path.join(&directory_path);

    // Mark as pending operation to avoid watcher events
    watcher_state.mark_pending_operation(full_path).await;

    manager_state
        .create_directory(&vault_id, &directory_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_entry(
    vault_id: Uuid,
    entry_path: String,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Get vault to construct full path
    let vault = manager_state
        .list_vaults()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|v| v.id == vault_id)
        .ok_or("Vault not found")?;

    let full_path = vault.path.join(&entry_path);

    // Mark as pending operation to avoid watcher events
    watcher_state.mark_pending_operation(full_path).await;

    manager_state
        .delete_entry(&vault_id, &entry_path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_entry(
    vault_id: Uuid,
    old_path: String,
    new_path: String,
    manager_state: State<'_, VaultManager>,
    watcher_state: State<'_, VaultWatcher>,
) -> Result<(), String> {
    // Get vault to construct full paths
    let vault = manager_state
        .list_vaults()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .find(|v| v.id == vault_id)
        .ok_or("Vault not found")?;

    let old_full_path = vault.path.join(&old_path);
    let new_full_path = vault.path.join(&new_path);

    // Mark both paths as pending operations to avoid watcher events
    watcher_state.mark_pending_operation(old_full_path).await;
    watcher_state.mark_pending_operation(new_full_path).await;

    manager_state
        .rename_entry(&vault_id, &old_path, &new_path)
        .await
        .map_err(|e| e.to_string())
}
