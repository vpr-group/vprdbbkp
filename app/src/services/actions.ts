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
    const entries = await invoke<Entry[]>("list", {
      storageConfig,
    });

    return entries.filter((it) => it.metadata.mode === "FILE");
  }

  async verifySourceConnection(
    sourceConfig: SourceConfig
  ): Promise<BackupSourceConnection> {
    const result = await invoke<BackupSourceConnection>("verify_connection", {
      sourceConfig,
    });

    return result;
  }

  async backup(
    sourceConfig: SourceConfig,
    storageConfig: StorageConfig
  ): Promise<void> {
    await invoke<string>("backup", {
      sourceConfig,
      storageConfig,
    });
  }

  async restore(
    filename: string,
    sourceConfig: SourceConfig,
    storageConfig: StorageConfig,
    dropDatabase: boolean
  ): Promise<void> {
    await invoke<string>("restore", {
      filename,
      sourceConfig,
      storageConfig,
      dropDatabase,
    });
  }
}
