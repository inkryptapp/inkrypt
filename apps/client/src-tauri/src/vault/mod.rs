pub mod commands;
pub mod manager;
pub mod models;
pub mod watcher;

pub use manager::VaultManager;
pub use models::*;
pub use watcher::VaultWatcher;

#[cfg(test)]
mod integration_tests {
    use super::*;
    use serial_test::serial;
    use tempfile::TempDir;
    use uuid::Uuid;

    fn setup_test_environment() -> (VaultManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        // Create a VaultManager with a custom registry path
        let registry_path = temp_dir.path().join("test_vaults.json");
        let manager = VaultManager::new_for_testing(registry_path);

        (manager, temp_dir)
    }

    #[tokio::test]
    #[serial]
    async fn test_vault_lifecycle() {
        let (manager, temp_dir) = setup_test_environment();

        // Test vault creation
        let vault = manager
            .create_vault(temp_dir.path(), "lifecycle_test")
            .await
            .unwrap();
        assert_eq!(vault.name, "lifecycle_test");
        assert!(vault.path.exists());

        // Test vault listing
        let vaults = manager.list_vaults().await.unwrap();
        assert_eq!(vaults.len(), 1);
        assert_eq!(vaults[0].id, vault.id);

        // Test vault opening
        let opened_vault = manager.open_vault(&vault.path).await.unwrap();
        assert_eq!(opened_vault.id, vault.id);

        // Test vault renaming
        let renamed_vault = manager
            .rename_vault(&vault.id, "renamed_vault")
            .await
            .unwrap();
        assert_eq!(renamed_vault.name, "renamed_vault");
        assert_ne!(renamed_vault.path, vault.path);

        // Test vault deletion
        manager.delete_vault(&vault.id).await.unwrap();
        let vaults_after_delete = manager.list_vaults().await.unwrap();
        assert!(vaults_after_delete.is_empty());
        assert!(!renamed_vault.path.exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_entry_operations() {
        let (manager, temp_dir) = setup_test_environment();

        // Create a vault
        let vault = manager
            .create_vault(temp_dir.path(), "entry_test")
            .await
            .unwrap();

        // Test directory creation
        manager
            .create_directory(&vault.id, "test_directory")
            .await
            .unwrap();

        // Test note creation
        manager
            .create_note(&vault.id, "test_note.md")
            .await
            .unwrap();

        // Test note creation in subdirectory
        manager
            .create_note(&vault.id, "test_directory/sub_note.md")
            .await
            .unwrap();

        // Test listing entries
        let entries = manager.list_entries(&vault.id, None).await.unwrap();
        assert_eq!(entries.len(), 2); // directory and top-level note

        let directory_entries = manager
            .list_entries(&vault.id, Some("test_directory"))
            .await
            .unwrap();
        assert_eq!(directory_entries.len(), 1); // sub_note.md

        // Test note editing and reading
        let content = "# Test Content\n\nThis is a test.";
        manager
            .edit_note(&vault.id, "test_note.md", content)
            .await
            .unwrap();
        let read_content = manager.read_note(&vault.id, "test_note.md").await.unwrap();
        assert_eq!(read_content, content);

        // Test entry renaming
        manager
            .rename_entry(&vault.id, "test_note.md", "renamed_note.md")
            .await
            .unwrap();
        assert!(vault.path.join("renamed_note.md").exists());
        assert!(!vault.path.join("test_note.md").exists());

        // Test entry deletion
        manager
            .delete_entry(&vault.id, "renamed_note.md")
            .await
            .unwrap();
        assert!(!vault.path.join("renamed_note.md").exists());

        manager
            .delete_entry(&vault.id, "test_directory")
            .await
            .unwrap();
        assert!(!vault.path.join("test_directory").exists());
    }

    #[tokio::test]
    #[serial]
    async fn test_vault_registry_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("test_vaults.json");

        // Create first manager instance
        {
            let manager = VaultManager::new_for_testing(registry_path.clone());

            let vault = manager
                .create_vault(temp_dir.path(), "persistence_test")
                .await
                .unwrap();
            assert!(registry_path.exists());

            // Registry should contain the vault
            let vaults = manager.list_vaults().await.unwrap();
            assert_eq!(vaults.len(), 1);
            assert_eq!(vaults[0].id, vault.id);
        }

        // Create second manager instance (simulating app restart)
        {
            let manager = VaultManager::new_for_testing(registry_path.clone());
            manager.load_registry_for_testing().await.unwrap();

            // Should still have the vault from previous session
            let vaults = manager.list_vaults().await.unwrap();
            assert_eq!(vaults.len(), 1);
            assert_eq!(vaults[0].name, "persistence_test");
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_error_handling() {
        let (manager, temp_dir) = setup_test_environment();

        // Test operations on non-existent vault
        let fake_id = Uuid::now_v7();

        let result = manager.create_directory(&fake_id, "test").await;
        assert!(result.is_err());

        let result = manager.create_note(&fake_id, "test.md").await;
        assert!(result.is_err());

        let result = manager.read_note(&fake_id, "test.md").await;
        assert!(result.is_err());

        let result = manager.delete_entry(&fake_id, "test").await;
        assert!(result.is_err());

        // Test opening invalid vault path
        let invalid_path = temp_dir.path().join("invalid");
        let result = manager.open_vault(&invalid_path).await;
        assert!(result.is_err());

        // Test reading non-existent note
        let vault = manager
            .create_vault(temp_dir.path(), "error_test")
            .await
            .unwrap();
        let result = manager.read_note(&vault.id, "nonexistent.md").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[serial]
    async fn test_nested_directories() {
        let (manager, temp_dir) = setup_test_environment();
        let vault = manager
            .create_vault(temp_dir.path(), "nested_test")
            .await
            .unwrap();

        // Create nested directory structure
        manager.create_directory(&vault.id, "level1").await.unwrap();
        manager
            .create_directory(&vault.id, "level1/level2")
            .await
            .unwrap();
        manager
            .create_directory(&vault.id, "level1/level2/level3")
            .await
            .unwrap();

        // Create notes at different levels
        manager.create_note(&vault.id, "root.md").await.unwrap();
        manager
            .create_note(&vault.id, "level1/note1.md")
            .await
            .unwrap();
        manager
            .create_note(&vault.id, "level1/level2/note2.md")
            .await
            .unwrap();
        manager
            .create_note(&vault.id, "level1/level2/level3/note3.md")
            .await
            .unwrap();

        // Test listing at different levels
        let root_entries = manager.list_entries(&vault.id, None).await.unwrap();
        assert_eq!(root_entries.len(), 2); // level1 directory and root.md

        let level1_entries = manager
            .list_entries(&vault.id, Some("level1"))
            .await
            .unwrap();
        assert_eq!(level1_entries.len(), 2); // level2 directory and note1.md

        let level2_entries = manager
            .list_entries(&vault.id, Some("level1/level2"))
            .await
            .unwrap();
        assert_eq!(level2_entries.len(), 2); // level3 directory and note2.md

        let level3_entries = manager
            .list_entries(&vault.id, Some("level1/level2/level3"))
            .await
            .unwrap();
        assert_eq!(level3_entries.len(), 1); // note3.md only

        // Test reading nested note
        let content = "Deep nested content";
        manager
            .edit_note(&vault.id, "level1/level2/level3/note3.md", content)
            .await
            .unwrap();
        let read_content = manager
            .read_note(&vault.id, "level1/level2/level3/note3.md")
            .await
            .unwrap();
        assert_eq!(read_content, content);
    }
}
