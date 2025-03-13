import { invoke } from "@tauri-apps/api/core";
import type { StorageProvider, BackupSource } from "./store";

export interface BackupListItem {
  backupType: string;
  dbName: string;
  key: string;
  lastModified: string;
  size: number;
  timestamp: string;
}

export interface BackupSourceConnection {
  connected: boolean;
}

export class ActionsService {
  async listBackups(
    storageProvider: StorageProvider
  ): Promise<BackupListItem[]> {
    try {
      const backups = await invoke<BackupListItem[]>("list_backups", {
        storageProvider,
      });
      return backups;
    } catch (error) {
      console.error("Failed to list backups:", error);
      throw new Error(`Failed to list backups: ${error}`);
    }
  }

  async verifyBackupSourceConnection(
    backupSource: BackupSource
  ): Promise<BackupSourceConnection> {
    try {
      const result = await invoke<BackupSourceConnection>(
        "verify_backup_source_connection",
        { backupSource }
      );
      return result;
    } catch (error) {
      console.error("Failed to test backup source connection:", error);
      return {
        connected: false,
      };
    }
  }

  async backupSource(
    backupSource: BackupSource,
    storageProvider: StorageProvider
  ): Promise<void> {
    try {
      const result = await invoke<BackupSourceConnection>("backup_source", {
        backupSource,
        storageProvider,
      });
      console.log(result);
    } catch (error) {
      console.error("Failed to test backup source connection:", error);
    }
  }
}
