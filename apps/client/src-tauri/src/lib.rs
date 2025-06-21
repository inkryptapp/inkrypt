// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use tauri::Manager;

pub mod vault;

pub use vault::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let vault_manager = VaultManager::new();
            let vault_watcher = VaultWatcher::new(app.handle().clone());

            app.manage(vault_manager);
            app.manage(vault_watcher);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            vault::commands::create_vault,
            vault::commands::list_vaults,
            vault::commands::open_vault,
            vault::commands::close_vault,
            vault::commands::delete_vault,
            vault::commands::rename_vault,
            vault::commands::list_entries,
            vault::commands::read_note,
            vault::commands::edit_note,
            vault::commands::create_note,
            vault::commands::write_file,
            vault::commands::create_directory,
            vault::commands::delete_entry,
            vault::commands::rename_entry
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
