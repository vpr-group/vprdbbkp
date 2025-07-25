import { createId } from "@paralleldrive/cuid2";
import { path } from "@tauri-apps/api";
import { load, Store } from "@tauri-apps/plugin-store";

const STORAGE_CONFIGS_KEY = "storage-configs";
const DATABASE_CONFIGS_KEY = "source-configs";

export interface BaseStorageConfig {
  id: string;
  name: string;
  type: "s3" | "local";
}

export interface S3StorageConfig extends BaseStorageConfig {
  type: "s3";
  region: string;
  bucket: string;
  endpoint: string;
  accessKey: string;
  secretKey: string;
}

export interface LocalStorageConfig extends BaseStorageConfig {
  type: "local";
  root: string;
}

export type StorageConfig = S3StorageConfig | LocalStorageConfig;

export interface TunnelConfig {
  useTunnel: boolean;
  username: string;
  keyPath: string;
}

export interface BaseDatabaseConfig {
  id: string;
  name: string;
  connection_type: "PostgreSql";
}

export interface PostgresdatabaseConfig extends BaseDatabaseConfig {
  connection_type: "PostgreSql";
  database: string;
  host: string;
  port: number;
  username: string;
  password: string;
  tunnelConfig?: TunnelConfig;
}

export type DatabaseConfig = PostgresdatabaseConfig;

export class StoreService {
  private internalStore: Store | undefined;
  private initializationPromise: Promise<void>;

  constructor() {
    this.initializationPromise = this.initialize();
  }

  private async initialize() {
    this.internalStore = await load("store.json", { autoSave: false });

    const storageConfigs = await this.getStorageConfigs();
    const defaultStorageConfig = storageConfigs.find(
      (it) => it.name === "Default Local Storage"
    );

    if (!defaultStorageConfig) {
      const documentsDir = await path.documentDir();
      this.saveStorageConfig({
        id: createId(),
        name: "Default Local Storage",
        type: "local",
        root: `${documentsDir}/vprdbbkp`,
      });
    }
  }

  async waitForInitialized(): Promise<void> {
    return this.initializationPromise;
  }

  get store() {
    if (!this.internalStore) throw new Error("Store is not initialized");
    return this.internalStore;
  }

  // Storage Provider Methods
  private async getRawStorageConfigs(): Promise<Record<string, StorageConfig>> {
    const storageProviders =
      ((await this.store.get(STORAGE_CONFIGS_KEY)) as Record<
        string,
        StorageConfig
      >) || {};
    return storageProviders;
  }

  async getStorageConfigs(): Promise<StorageConfig[]> {
    const storageProviders = await this.getRawStorageConfigs();
    return Object.values(storageProviders);
  }

  async getStorageConfig(id: string): Promise<StorageConfig | null> {
    const storageProviders = await this.getRawStorageConfigs();
    return storageProviders[id] || null;
  }

  async saveStorageConfig(
    storageConfig: StorageConfig
  ): Promise<StorageConfig> {
    const storageProviders = await this.getRawStorageConfigs();

    const completeStorageConfig = {
      ...storageConfig,
      id: storageConfig.id || createId(),
    };

    const updatedStorageProviders = {
      ...storageProviders,
      [completeStorageConfig.id]: completeStorageConfig,
    };

    await this.store.set(STORAGE_CONFIGS_KEY, updatedStorageProviders);
    await this.store.save();

    return completeStorageConfig;
  }

  async deleteStorageConfig(id: string): Promise<void> {
    const storageProviders = await this.getRawStorageConfigs();
    if (storageProviders[id]) {
      delete storageProviders[id];
      await this.store.set(STORAGE_CONFIGS_KEY, storageProviders);
    }
    await this.store.save();
  }

  // Backup Source Methods
  private async getRawDatabaseConfigs(): Promise<
    Record<string, DatabaseConfig>
  > {
    const databaseConfigs =
      ((await this.store.get(DATABASE_CONFIGS_KEY)) as Record<
        string,
        DatabaseConfig
      >) || {};
    return databaseConfigs;
  }

  async getDatabaseConfigs(): Promise<DatabaseConfig[]> {
    const databaseConfigs = await this.getRawDatabaseConfigs();
    return Object.values(databaseConfigs);
  }

  async getDatabaseConfig(id: string): Promise<DatabaseConfig | null> {
    const databaseConfigs = await this.getRawDatabaseConfigs();
    return databaseConfigs[id] || null;
  }

  async saveDatabaseConfig(
    backupSource: DatabaseConfig
  ): Promise<DatabaseConfig> {
    const databaseConfigs = await this.getRawDatabaseConfigs();
    const completedatabaseConfig = {
      ...backupSource,
      id: backupSource.id || createId(),
    } as DatabaseConfig;

    const updateddatabaseConfigs = {
      ...databaseConfigs,
      [completedatabaseConfig.id]: completedatabaseConfig,
    };

    await this.store.set(DATABASE_CONFIGS_KEY, updateddatabaseConfigs);
    await this.store.save();
    return completedatabaseConfig;
  }

  async deleteDatabaseConfig(id: string): Promise<void> {
    const databaseConfigs = await this.getRawDatabaseConfigs();
    if (databaseConfigs[id]) {
      console.log("deleting");
      delete databaseConfigs[id];
      await this.store.set(DATABASE_CONFIGS_KEY, databaseConfigs);
    }
    await this.store.save();
  }
}
