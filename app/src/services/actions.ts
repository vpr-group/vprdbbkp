import { invoke } from "@tauri-apps/api/core";
import type { StorageConfig, DatabaseConfig } from "./store";

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

function mapStorageConfig(storageConfig: StorageConfig) {
  return storageConfig.type === "local"
    ? {
        Local: storageConfig,
      }
    : storageConfig.type === "s3"
    ? {
        S3: storageConfig,
      }
    : undefined;
}

export class ActionsService {
  async list(storageConfig: StorageConfig): Promise<Entry[]> {
    const entries = await invoke<Entry[]>("list", {
      storageConfig: mapStorageConfig(storageConfig),
    });

    return entries.filter((it) => it.metadata.mode === "FILE");
  }

  async testConnection(
    databaseConfig: DatabaseConfig
  ): Promise<BackupSourceConnection> {
    const result = await invoke<BackupSourceConnection>("test_connection", {
      databaseConfig,
    });

    return result;
  }

  async backup(
    databaseConfig: DatabaseConfig,
    storageConfig: StorageConfig
  ): Promise<void> {
    await invoke<string>("backup", {
      databaseConfig,
      storageConfig: mapStorageConfig(storageConfig),
    });
  }

  async restore(
    filename: string,
    databaseConfig: DatabaseConfig,
    storageConfig: StorageConfig,
    dropDatabase: boolean
  ): Promise<void> {
    await invoke<string>("restore", {
      filename,
      databaseConfig,
      storageConfig: mapStorageConfig(storageConfig),
      dropDatabase,
    });
  }
}
