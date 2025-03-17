import { createId } from "@paralleldrive/cuid2";
import { path } from "@tauri-apps/api";
import { load, Store } from "@tauri-apps/plugin-store";

const STORAGE_CONFIGS_KEY = "storage-configs";
const SOURCE_CONFIGS_KEY = "source-configs";

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

export interface BaseSourceConfig {
  id: string;
  name: string;
  type: "pg";
}

export interface PostgresSourceConfig extends BaseSourceConfig {
  type: "pg";
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
}

export type SourceConfig = PostgresSourceConfig;

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
      (it) => it.name === "Default Local Storage",
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
  private async getRawSourceConfigs(): Promise<Record<string, SourceConfig>> {
    const sourceConfigs =
      ((await this.store.get(SOURCE_CONFIGS_KEY)) as Record<
        string,
        SourceConfig
      >) || {};
    return sourceConfigs;
  }

  async getSourceConfigs(): Promise<SourceConfig[]> {
    const sourceConfigs = await this.getRawSourceConfigs();
    return Object.values(sourceConfigs);
  }

  async getSourceConfig(id: string): Promise<SourceConfig | null> {
    const sourceConfigs = await this.getRawSourceConfigs();
    return sourceConfigs[id] || null;
  }

  async saveSourceConfig(backupSource: SourceConfig): Promise<SourceConfig> {
    const sourceConfigs = await this.getRawSourceConfigs();
    const completeSourceConfig = {
      ...backupSource,
      id: backupSource.id || createId(),
    } as SourceConfig;

    const updatedSourceConfigs = {
      ...sourceConfigs,
      [completeSourceConfig.id]: completeSourceConfig,
    };

    await this.store.set(SOURCE_CONFIGS_KEY, updatedSourceConfigs);
    await this.store.save();
    return completeSourceConfig;
  }

  async deleteSourceConfig(id: string): Promise<void> {
    const sourceConfigs = await this.getRawSourceConfigs();
    if (sourceConfigs[id]) {
      console.log("deleting")
      delete sourceConfigs[id];
      await this.store.set(SOURCE_CONFIGS_KEY, sourceConfigs);
    }
    await this.store.save();
  }
}
