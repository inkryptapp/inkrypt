use crate::vault::models::*;
use anyhow::Result;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{sleep, timeout};
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Debug)]
#[allow(dead_code)]
struct WatcherState {
    vault_id: Uuid,
    vault_path: PathBuf,
    watcher: RecommendedWatcher,
}

pub struct VaultWatcher {
    current_watcher: Arc<RwLock<Option<WatcherState>>>,
    app_handle: AppHandle,
    pending_operations: Arc<RwLock<HashSet<PathBuf>>>,
}

impl VaultWatcher {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            current_watcher: Arc::new(RwLock::new(None)),
            app_handle,
            pending_operations: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub async fn watch_vault(&self, vault_id: Uuid, vault_path: PathBuf) -> Result<()> {
        // Stop watching any existing vault first
        self.unwatch_current_vault().await?;

        let (tx, mut rx) = mpsc::channel(100);
        let debounce_duration = Duration::from_millis(200);

        // Create the watcher
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Err(e) = tx.blocking_send(res) {
                    error!("Failed to send event: {}", e);
                }
            },
            Config::default(),
        )?;

        // Watch the vault directory
        watcher.watch(&vault_path, RecursiveMode::Recursive)?;

        let watcher_state = WatcherState {
            vault_id,
            watcher,
            vault_path: vault_path.clone(),
        };

        // Store the watcher
        {
            let mut current = self.current_watcher.write().await;
            *current = Some(watcher_state);
        }

        // Process events in a separate task
        let app_handle = self.app_handle.clone();
        let pending_ops = self.pending_operations.clone();
        let vault_path_clone = vault_path.clone();

        tokio::spawn(async move {
            let mut event_buffer: Vec<FileSystemEvent> = Vec::new();
            let mut last_emit = tokio::time::Instant::now();

            loop {
                // Wait for events with timeout
                match timeout(debounce_duration, rx.recv()).await {
                    Ok(Some(Ok(event))) => {
                        if let Some(fs_event) =
                            process_notify_event(event, &vault_id, &vault_path_clone, &pending_ops)
                                .await
                        {
                            event_buffer.push(fs_event);
                        }
                    }
                    Ok(Some(Err(e))) => {
                        error!("Watch error: {}", e);
                    }
                    Ok(None) => {
                        info!("Watcher channel closed");
                        break;
                    }
                    Err(_) => {
                        // Timeout - check if we should emit buffered events
                        if !event_buffer.is_empty() && last_emit.elapsed() >= debounce_duration {
                            // Deduplicate events
                            let unique_events = deduplicate_events(event_buffer.clone());

                            // Emit events
                            if let Err(e) = app_handle.emit("vault-changes", &unique_events) {
                                error!("Failed to emit vault changes: {}", e);
                            }

                            event_buffer.clear();
                            last_emit = tokio::time::Instant::now();
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn unwatch_vault(&self, vault_id: &Uuid) -> Result<()> {
        let mut current = self.current_watcher.write().await;
        if let Some(ref mut watcher_state) = *current {
            if &watcher_state.vault_id == vault_id {
                // Properly unwatch the path
                if let Err(e) = watcher_state.watcher.unwatch(&watcher_state.vault_path) {
                    error!(
                        "Failed to unwatch path {:?}: {}",
                        watcher_state.vault_path, e
                    );
                }
                *current = None;
                info!("Stopped watching vault: {}", vault_id);
            }
        }
        Ok(())
    }

    async fn unwatch_current_vault(&self) -> Result<()> {
        let mut current = self.current_watcher.write().await;
        if let Some(mut watcher_state) = current.take() {
            // Properly unwatch the path
            if let Err(e) = watcher_state.watcher.unwatch(&watcher_state.vault_path) {
                error!(
                    "Failed to unwatch path {:?}: {}",
                    watcher_state.vault_path, e
                );
            }
            info!("Stopped watching vault: {}", watcher_state.vault_id);
        }
        Ok(())
    }

    pub async fn mark_pending_operation(&self, path: PathBuf) {
        let mut pending: tokio::sync::RwLockWriteGuard<'_, HashSet<PathBuf>> =
            self.pending_operations.write().await;
        pending.insert(path.clone());

        // Remove after a delay (preventing unnecessary UI re-renders)
        let pending_ops = self.pending_operations.clone();
        let path_clone = path;
        tokio::spawn(async move {
            sleep(Duration::from_millis(500)).await;
            let mut pending = pending_ops.write().await;
            pending.remove(&path_clone);
        });
    }
}

async fn process_notify_event(
    event: Event,
    vault_id: &Uuid,
    vault_path: &Path,
    pending_operations: &Arc<RwLock<HashSet<PathBuf>>>,
) -> Option<FileSystemEvent> {
    // Check if this is a pending operation
    {
        let pending = pending_operations.read().await;
        for path in &event.paths {
            if pending.contains(path) {
                debug!("Ignoring pending operation for: {:?}", path);
                return None;
            }
        }
    }

    // Filter out .inkrypt directory changes
    for path in &event.paths {
        if path.components().any(|c| c.as_os_str() == ".inkrypt") {
            return None;
        }
    }

    let event_type = match event.kind {
        EventKind::Create(_) => FileEventType::Create,
        EventKind::Modify(_) => FileEventType::Modify,
        EventKind::Remove(_) => FileEventType::Delete,
        EventKind::Any => return None,
        EventKind::Access(_) => return None,
        EventKind::Other => return None,
    };

    if let Some(path) = event.paths.first() {
        if let Ok(relative_path) = path.strip_prefix(vault_path) {
            let path_str = relative_path.to_string_lossy().replace('\\', "/");

            return Some(FileSystemEvent {
                event_type,
                path: path_str,
                vault_id: *vault_id,
            });
        }
    }

    None
}

fn deduplicate_events(events: Vec<FileSystemEvent>) -> Vec<FileSystemEvent> {
    let mut seen = HashSet::new();
    let mut unique_events = Vec::new();

    for event in events.into_iter().rev() {
        let key = (event.vault_id, event.path.clone());
        if seen.insert(key) {
            unique_events.push(event);
        }
    }

    unique_events.reverse();
    unique_events
}
