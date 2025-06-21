export interface Vault {
  id: string;
  name: string;
  path: string;
  version: number;
  createdAt: string;
  updatedAt: string;
}

export interface Entry {
  name: string;
  path: string;
  entryType: "directory" | "note";
  createdAt?: string;
  updatedAt?: string;
}

export interface FileSystemEvent {
  eventType: "create" | "modify" | "delete" | "rename";
  path: string;
  vaultId: string;
}
