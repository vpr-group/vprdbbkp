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

export class ActionsService {
  /**
   * List all backups for a storage provider
   */
  async listBackups(
    storageProvider: StorageProvider
  ): Promise<BackupListItem[]> {
    try {
      // Call the Tauri backend function
      const backups = await invoke<BackupListItem[]>("list_backups", {
        storageProvider,
      });
      return backups;
    } catch (error) {
      console.error("Failed to list backups:", error);
      throw new Error(`Failed to list backups: ${error}`);
    }
  }

  /**
   * Create a backup from a backup source to a storage provider
   */
  async createBackup(backupSource: BackupSource): Promise<string> {
    try {
      // Call the Tauri backend function
      const backupId = await invoke<string>("create_backup", {
        backupSource,
      });
      return backupId; // Returns a unique ID for the created backup
    } catch (error) {
      console.error("Failed to create backup:", error);
      throw new Error(`Failed to create backup: ${error}`);
    }
  }

  /**
   * Restore a backup from a storage provider
   */
  async restoreBackup(
    storageProvider: StorageProvider,
    backupName: string,
    destination?: string
  ): Promise<boolean> {
    try {
      // Call the Tauri backend function
      const success = await invoke<boolean>("restore_backup", {
        storageProvider,
        backupName,
        destination,
      });
      return success;
    } catch (error) {
      console.error("Failed to restore backup:", error);
      throw new Error(`Failed to restore backup: ${error}`);
    }
  }

  /**
   * Delete a backup from a storage provider
   */
  async deleteBackup(
    storageProvider: StorageProvider,
    backupName: string
  ): Promise<boolean> {
    try {
      // Call the Tauri backend function
      const success = await invoke<boolean>("delete_backup", {
        storageProvider,
        backupName,
      });
      return success;
    } catch (error) {
      console.error("Failed to delete backup:", error);
      throw new Error(`Failed to delete backup: ${error}`);
    }
  }

  /**
   * Test the connection to a storage provider
   */
  async testStorageConnection(
    storageProvider: StorageProvider
  ): Promise<boolean> {
    try {
      // Call the Tauri backend function
      const isConnected = await invoke<boolean>("test_storage_connection", {
        storageProvider,
      });
      return isConnected;
    } catch (error) {
      console.error("Failed to test storage connection:", error);
      return false;
    }
  }

  /**
   * Test the connection to a database backup source
   */
  async testDatabaseConnection(backupSource: BackupSource): Promise<boolean> {
    if (backupSource.type !== "database") {
      throw new Error("Backup source is not a database");
    }

    try {
      // Call the Tauri backend function
      const isConnected = await invoke<boolean>("test_database_connection", {
        backupSource,
      });
      return isConnected;
    } catch (error) {
      console.error("Failed to test database connection:", error);
      return false;
    }
  }
}
