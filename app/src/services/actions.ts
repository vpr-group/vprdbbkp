import { invoke } from "@tauri-apps/api/core";
import type { StorageConfig, SourceConfig } from "./store";

export interface Entry {
  path: string;
  metadata: {
    cache_control?: string;
    content_disposition?: string;
    content_encoding?: string;
    content_length?: number;
    content_md5?: string;
    content_type?: string;
    etag?: string;
    is_current?: boolean;
    is_deleted?: boolean;
    last_modified?: string;
    mode?: string;
    user_metadata?: unknown;
    version?: string;
  };
}

export interface BackupSourceConnection {
  connected: boolean;
}

export class ActionsService {
  async list(storageConfig: StorageConfig): Promise<Entry[]> {
    try {
      const entries = await invoke<Entry[]>("list", {
        storageConfig,
      });

      return entries;
    } catch (error) {
      console.error("Failed to list backups:", error);
      throw new Error(`Failed to list backups: ${error}`);
    }
  }

  async verifySourceConnection(
    sourceConfig: SourceConfig
  ): Promise<BackupSourceConnection> {
    try {
      const result = await invoke<BackupSourceConnection>("verify_connection", {
        sourceConfig,
      });
      return result;
    } catch (error) {
      console.error("Failed to test backup source connection:", error);
      return {
        connected: false,
      };
    }
  }

  async backup(
    sourceConfig: SourceConfig,
    storageConfig: StorageConfig
  ): Promise<void> {
    try {
      const result = await invoke<string>("backup", {
        sourceConfig,
        storageConfig,
      });
      console.log(result);
    } catch (error) {
      console.error("Failed to backup source:", error);
    }
  }

  async restore(
    filename: string,
    sourceConfig: SourceConfig,
    storageConfig: StorageConfig
  ): Promise<void> {
    try {
      const result = await invoke<string>("restore", {
        filename,
        sourceConfig,
        storageConfig,
      });
      console.log(result);
    } catch (error) {
      console.error("Failed to restore backup:", error);
    }
  }
}
