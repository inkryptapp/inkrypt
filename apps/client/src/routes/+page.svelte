<script lang="ts">
  import { Button, Input } from "@inkrypt/ui";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onDestroy, onMount } from "svelte";
  import type { Entry, FileSystemEvent, Vault } from "../types/vault";

  let vaults: Vault[] = $state([]);
  let selectedVault: Vault | null = $state(null);
  let entries: Entry[] = $state([]);
  let currentPath = $state("");
  let newVaultName = $state("");
  let newDirectoryName = $state("");
  let newNoteName = $state("");
  let noteContent = $state("");
  let selectedNote: string | null = $state(null);
  let unlisten: (() => void) | null = null;

  // Editing states
  let editingVault: string | null = $state(null);
  let editingEntry: string | null = $state(null);
  let editingValue = $state("");
  let showImportDialog = $state(false);

  onMount(async () => {
    await loadVaults();

    // Listen for file system changes
    unlisten = await listen<FileSystemEvent[]>("vault-changes", async event => {
      console.log("Vault changes:", event.payload);
      if (!selectedVault) return;

      const relevantEvents = event.payload.filter(e => e.vaultId === selectedVault.id);
      if (relevantEvents.length === 0) return;

      await loadEntries();

      for (const fsEvent of relevantEvents) {
        await handleFileSystemEvent(fsEvent);
      }
    });
  });

  onDestroy(() => {
    if (unlisten) {
      unlisten();
    }
  });

  async function loadVaults() {
    try {
      vaults = await invoke<Vault[]>("list_vaults");
    } catch (error) {
      console.error("Error loading vaults:", error);
    }
  }

  async function createVault() {
    if (!newVaultName) return;

    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const rootDirectory = await open({
        directory: true,
        multiple: false,
      });

      if (rootDirectory && typeof rootDirectory === "string") {
        const vault = await invoke<Vault>("create_vault", {
          rootDirectory,
          name: newVaultName,
        });
        vaults = [...vaults, vault];
        newVaultName = "";
      }
    } catch (error) {
      console.error("Error creating vault:", error);
    }
  }

  async function openVault(vault: Vault) {
    try {
      selectedVault = await invoke<Vault>("open_vault", {
        vaultPath: vault.path,
      });
      currentPath = "";
      await loadEntries();
    } catch (error) {
      console.error("Error opening vault:", error);
    }
  }

  async function deleteVault(vault: Vault) {
    try {
      await invoke("delete_vault", { vaultId: vault.id });
      vaults = vaults.filter(v => v.id !== vault.id);
      if (selectedVault?.id === vault.id) {
        selectedVault = null;
        entries = [];
        selectedNote = null;
        noteContent = "";
      }
    } catch (error) {
      console.error("Error deleting vault:", error);
    }
  }

  async function loadEntries() {
    if (!selectedVault) return;

    try {
      entries = await invoke<Entry[]>("list_entries", {
        vaultId: selectedVault.id,
        directoryPath: currentPath || undefined,
      });
    } catch (error) {
      console.error("Error loading entries:", error);
    }
  }

  async function createDirectory() {
    if (!selectedVault || !newDirectoryName) return;

    try {
      const directoryPath = currentPath ? `${currentPath}/${newDirectoryName}` : newDirectoryName;
      await invoke("create_directory", {
        vaultId: selectedVault.id,
        directoryPath,
      });
      newDirectoryName = "";
      await loadEntries();
    } catch (error) {
      console.error("Error creating directory:", error);
    }
  }

  async function createNote() {
    if (!selectedVault || !newNoteName) return;

    try {
      const notePath = currentPath ? `${currentPath}/${newNoteName}.md` : `${newNoteName}.md`;
      await invoke("create_note", {
        vaultId: selectedVault.id,
        notePath,
      });
      newNoteName = "";
      await loadEntries();
    } catch (error) {
      console.error("Error creating note:", error);
    }
  }

  async function openEntry(entry: Entry) {
    if (entry.entryType === "directory") {
      currentPath = entry.path;
      await loadEntries();
    } else {
      selectedNote = entry.path;
      await loadNote(entry.path);
    }
  }

  async function loadNote(notePath: string) {
    if (!selectedVault) return;

    try {
      noteContent = await invoke<string>("read_note", {
        vaultId: selectedVault.id,
        notePath,
      });
    } catch (error) {
      console.error("Error loading note:", error);
    }
  }

  async function saveNote() {
    if (!selectedVault || !selectedNote) return;

    try {
      await invoke("edit_note", {
        vaultId: selectedVault.id,
        notePath: selectedNote,
        content: noteContent,
      });
      const fileMetadata = getSelectedEntryMetadata();
      if (fileMetadata) {
        fileMetadata.updatedAt = new Date().toISOString();
      }
    } catch (error) {
      console.error("Error saving note:", error);
    }
  }

  async function deleteEntry(entry: Entry) {
    if (!selectedVault) return;

    try {
      await invoke("delete_entry", {
        vaultId: selectedVault.id,
        entryPath: entry.path,
      });
      await loadEntries();
      if (entry.path === selectedNote) {
        selectedNote = null;
        noteContent = "";
      }
    } catch (error) {
      console.error("Error deleting entry:", error);
    }
  }

  async function goBack() {
    if (currentPath.includes("/")) {
      currentPath = currentPath.substring(0, currentPath.lastIndexOf("/"));
    } else {
      currentPath = "";
    }
    await loadEntries();
  }

  async function importVault() {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: "Select Vault Folder",
      });

      if (selectedPath && typeof selectedPath === "string") {
        const vault = await invoke<Vault>("open_vault", {
          vaultPath: selectedPath,
        });

        // Add to vaults list if not already there
        if (!vaults.find(v => v.id === vault.id)) {
          vaults = [...vaults, vault];
        }

        showImportDialog = false;
        console.log("Imported vault:", vault);
      }
    } catch (error) {
      console.error("Error importing vault:", error);
      alert("Failed to import vault. Make sure the selected folder contains a valid vault.");
    }
  }

  function startEditingVault(vault: Vault) {
    editingVault = vault.id;
    editingValue = vault.name;
  }

  function cancelEditingVault() {
    editingVault = null;
    editingValue = "";
  }

  async function saveVaultRename(vaultId: string) {
    if (!editingValue.trim()) return;

    try {
      await invoke("rename_vault", {
        vaultId,
        newName: editingValue.trim(),
      });

      // Reload vaults list to get updated names
      await loadVaults();

      // Update selectedVault if it's the one being renamed
      if (selectedVault?.id === vaultId) {
        const updatedVault = vaults.find(v => v.id === vaultId);
        if (updatedVault) {
          selectedVault = updatedVault;
        }
      }

      editingVault = null;
      editingValue = "";
    } catch (error) {
      console.error("Error renaming vault:", error);
      alert("Failed to rename vault");
    }
  }

  function startEditingEntry(entry: Entry) {
    editingEntry = entry.path;
    // Remove file extension for notes to make editing easier
    editingValue =
      entry.entryType === "note" && entry.name.endsWith(".md")
        ? entry.name.slice(0, -3)
        : entry.name;
  }

  function cancelEditingEntry() {
    editingEntry = null;
    editingValue = "";
  }

  async function saveEntryRename(oldPath: string, entryType: "directory" | "note") {
    if (!selectedVault || !editingValue.trim()) return;

    try {
      // Add .md extension for notes if not present
      let newName = editingValue.trim();
      if (entryType === "note" && !newName.endsWith(".md")) {
        newName += ".md";
      }

      // Construct new path
      let newPath: string;
      if (currentPath === "") {
        newPath = newName;
      } else {
        newPath = `${currentPath}/${newName}`;
      }

      await invoke("rename_entry", {
        vaultId: selectedVault.id,
        oldPath,
        newPath,
      });

      // Update the currently open note path if it was renamed
      if (selectedNote === oldPath) {
        selectedNote = newPath;
      }

      editingEntry = null;
      editingValue = "";
      await loadEntries();
    } catch (error) {
      console.error("Error renaming entry:", error);
      alert("Failed to rename entry");
    }
  }

  function handleKeydown(event: KeyboardEvent, action: () => void) {
    if (event.key === "Enter") {
      event.preventDefault();
      action();
    } else if (event.key === "Escape") {
      event.preventDefault();
      if (editingVault) cancelEditingVault();
      if (editingEntry) cancelEditingEntry();
    }
  }

  function getSelectedEntryMetadata(): Entry | null {
    if (!selectedNote) return null;
    return entries.find(entry => entry.path === selectedNote) || null;
  }

  function formatDate(dateString: string | undefined): string {
    if (!dateString) return "Unknown";
    try {
      return new Date(dateString).toLocaleString();
    } catch {
      return "Invalid date";
    }
  }

  async function handleFileSystemEvent(fsEvent: FileSystemEvent) {
    if (!selectedVault) return;

    const eventPath = fsEvent.path;

    switch (fsEvent.eventType) {
      case "create":
        break;

      case "modify":
        // If this is the currently open note, try to reload its content
        if (selectedNote === eventPath) {
          try {
            const newContent = await invoke<string>("read_note", {
              vaultId: selectedVault.id,
              notePath: eventPath,
            });
            noteContent = newContent;
            console.log(`Reloaded content for: ${eventPath}`);
          } catch (error) {
            // If we can't read the file, it might have been deleted
            // This handles the macOS quirk where deletions are reported as modify events
            console.log(`Could not read file ${eventPath}, treating as deletion`);
            selectedNote = null;
            noteContent = "";
            console.log(`Closed deleted note: ${eventPath}`);
          }
        }
        break;

      case "delete":
        // If this was the currently open note, clear the editor
        if (selectedNote === eventPath) {
          selectedNote = null;
          noteContent = "";
          console.log(`Closed deleted note: ${eventPath}`);
        }
        break;

      case "rename":
        break;
    }
  }
</script>

<div class="container mx-auto p-4">
  <h1 class="mb-6 text-3xl font-bold">Inkrypt</h1>

  <div class="grid grid-cols-12 gap-4">
    <!-- Vaults Sidebar -->
    <div class="col-span-3 border-r pr-4">
      <div class="mb-4 flex items-center justify-between">
        <h2 class="text-xl font-semibold">Vaults</h2>
        <Button onclick={() => (showImportDialog = true)} size="sm" variant="outline">
          Import
        </Button>
      </div>

      <div class="mb-4 flex gap-2">
        <Input bind:value={newVaultName} placeholder="Vault name" class="flex-1" />
        <Button onclick={createVault} size="sm">Create</Button>
      </div>

      <div class="space-y-2">
        {#each vaults as vault (vault.id)}
          <div class="hover:bg-secondary flex items-center justify-between rounded p-2">
            {#if editingVault === vault.id}
              <Input
                bind:value={editingValue}
                class="mr-2 flex-1"
                onkeydown={e => handleKeydown(e, () => saveVaultRename(vault.id))}
                placeholder="Vault name"
              />
              <div class="flex gap-1">
                <Button onclick={() => saveVaultRename(vault.id)} size="sm">✓</Button>
                <Button onclick={cancelEditingVault} size="sm" variant="outline">✕</Button>
              </div>
            {:else}
              <button
                onclick={() => openVault(vault)}
                class="flex-1 truncate text-left"
                class:font-semibold={selectedVault?.id === vault.id}
              >
                {vault.name}
              </button>
              <div class="flex gap-1">
                <Button onclick={() => startEditingVault(vault)} size="sm" variant="outline">
                  ✏️
                </Button>
                <Button onclick={() => deleteVault(vault)} size="sm" variant="destructive">
                  X
                </Button>
              </div>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <!-- File Explorer -->
    <div class="col-span-4 border-r pr-4">
      <h2 class="mb-4 text-xl font-semibold">
        {selectedVault ? `Files in ${selectedVault.name}` : "Select a vault"}
      </h2>

      {#if selectedVault}
        <div class="mb-4 space-y-2">
          {#if currentPath}
            <Button onclick={goBack} variant="outline" size="sm">← Back</Button>
            <p class="text-muted-foreground text-sm">Current: /{currentPath}</p>
          {/if}

          <div class="flex gap-2">
            <Input bind:value={newDirectoryName} placeholder="Directory name" class="flex-1" />
            <Button onclick={createDirectory} size="sm">+ Directory</Button>
          </div>

          <div class="flex gap-2">
            <Input bind:value={newNoteName} placeholder="Note name" class="flex-1" />
            <Button onclick={createNote} size="sm">+ Note</Button>
          </div>
        </div>

        <div class="space-y-1">
          {#each entries as entry (entry.path)}
            <div class="hover:bg-secondary flex items-center justify-between rounded p-2">
              {#if editingEntry === entry.path}
                <div class="flex flex-1 items-center gap-2">
                  {#if entry.entryType === "directory"}
                    📁
                  {:else}
                    📄
                  {/if}
                  <Input
                    bind:value={editingValue}
                    class="flex-1"
                    onkeydown={e =>
                      handleKeydown(e, () => saveEntryRename(entry.path, entry.entryType))}
                    placeholder={entry.entryType === "directory" ? "Directory name" : "Note name"}
                  />
                </div>
                <div class="flex gap-1">
                  <Button onclick={() => saveEntryRename(entry.path, entry.entryType)} size="sm"
                    >✓</Button
                  >
                  <Button onclick={cancelEditingEntry} size="sm" variant="outline">✕</Button>
                </div>
              {:else}
                <button
                  onclick={() => openEntry(entry)}
                  class="flex flex-1 items-center gap-2 truncate text-left"
                >
                  {#if entry.entryType === "directory"}
                    📁
                  {:else}
                    📄
                  {/if}
                  {entry.name}
                </button>
                <div class="flex gap-1">
                  <Button onclick={() => startEditingEntry(entry)} size="sm" variant="outline">
                    ✏️
                  </Button>
                  <Button onclick={() => deleteEntry(entry)} size="sm" variant="destructive">
                    X
                  </Button>
                </div>
              {/if}
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- Note Editor -->
    <div class="col-span-5">
      <h2 class="mb-4 text-xl font-semibold">
        {selectedNote ? `Editing: ${selectedNote}` : "Select a note"}
      </h2>

      {#if selectedNote}
        {@const fileMetadata = getSelectedEntryMetadata()}

        {#if fileMetadata}
          <div class="mb-4 rounded-lg bg-gray-50 p-3 text-sm">
            <div class="grid grid-cols-2 gap-4">
              <div>
                <span class="font-medium text-gray-600">Created:</span>
                <span class="text-gray-800">{formatDate(fileMetadata.createdAt)}</span>
              </div>
              <div>
                <span class="font-medium text-gray-600">Last edited:</span>
                <span class="text-gray-800">{formatDate(fileMetadata.updatedAt)}</span>
              </div>
            </div>
          </div>
        {/if}

        <div class="space-y-4">
          <textarea
            bind:value={noteContent}
            class="h-96 w-full rounded-lg border p-4 font-mono text-sm"
            placeholder="Start typing..."
          ></textarea>
          <Button onclick={saveNote}>Save Note</Button>
        </div>
      {/if}
    </div>
  </div>
</div>

<!-- Import Vault Dialog -->
{#if showImportDialog}
  <div class="bg-opacity-50 fixed inset-0 z-50 flex items-center justify-center bg-black">
    <div class="mx-4 w-full max-w-md rounded-lg bg-white p-6 shadow-lg">
      <h3 class="mb-4 text-lg font-semibold">Import Existing Vault</h3>
      <p class="mb-4 text-sm text-gray-600">
        Select a folder that contains a previously created vault (.inkrypt folder).
      </p>
      <div class="flex justify-end gap-2">
        <Button onclick={() => (showImportDialog = false)} variant="outline">Cancel</Button>
        <Button onclick={importVault}>Select Folder</Button>
      </div>
    </div>
  </div>
{/if}
