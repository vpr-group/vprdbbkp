import { createId } from "@paralleldrive/cuid2";
import { load, Store } from "@tauri-apps/plugin-store";

const STORAGE_PROVIDERS_KEY = "storage-providers";
const BACKUP_SOURCES_KEY = "backup-sources";

export interface BaseStorageProvider {
  id: string;
  type: "s3";
  name: string;
}

export interface S3StorageProvider extends BaseStorageProvider {
  type: "s3";
  region: string;
  bucket: string;
  endpoint: string;
  accessKey: string;
  secretKey: string;
}

export type StorageProvider = S3StorageProvider;

export interface BaseBackupSource {
  id: string;
  name: string;
  type: "database";
}

export interface DatabaseBackupSource extends BaseBackupSource {
  type: "database";
  databaseType: "mysql" | "postgresql";
  host: string;
  port: number;
  username: string;
  password: string;
  database: string;
}

export type BackupSource = DatabaseBackupSource;

export class StoreService {
  private internalStore: Store | undefined;
  private initializationPromise: Promise<void>;

  constructor() {
    this.initializationPromise = this.initialize();
  }

  private async initialize() {
    this.internalStore = await load("store.json", { autoSave: false });
  }

  async waitForInitialized(): Promise<void> {
    return this.initializationPromise;
  }

  get store() {
    if (!this.internalStore) throw new Error("Store is not initialized");
    return this.internalStore;
  }

  // Storage Provider Methods
  private async getRawStorageProviders(): Promise<
    Record<string, StorageProvider>
  > {
    const storageProviders =
      ((await this.store.get(STORAGE_PROVIDERS_KEY)) as Record<
        string,
        StorageProvider
      >) || {};
    return storageProviders;
  }

  async getStorageProviders(): Promise<StorageProvider[]> {
    const storageProviders = await this.getRawStorageProviders();
    return Object.values(storageProviders);
  }

  async getStorageProvider(id: string): Promise<StorageProvider | null> {
    const storageProviders = await this.getRawStorageProviders();
    return storageProviders[id] || null;
  }

  async saveStorageProvider(
    storageprovider: StorageProvider
  ): Promise<StorageProvider> {
    const storageProviders = await this.getRawStorageProviders();
    const completeStorageProvider: StorageProvider = {
      ...storageprovider,
      id: storageprovider.id || createId(), // Generate ID if not provided
    };
    const updatedStorageProviders = {
      ...storageProviders,
      [completeStorageProvider.id]: completeStorageProvider,
    };
    await this.store.set(STORAGE_PROVIDERS_KEY, updatedStorageProviders);
    await this.store.save();
    return completeStorageProvider;
  }

  async deleteStorageProvider(id: string): Promise<void> {
    const storageProviders = await this.getRawStorageProviders();
    if (storageProviders[id]) {
      delete storageProviders[id];
      await this.store.set(STORAGE_PROVIDERS_KEY, storageProviders);
    }
    await this.store.save();
  }

  // Backup Source Methods
  private async getRawBackupSources(): Promise<Record<string, BackupSource>> {
    const backupSources =
      ((await this.store.get(BACKUP_SOURCES_KEY)) as Record<
        string,
        BackupSource
      >) || {};
    return backupSources;
  }

  async getBackupSources(): Promise<BackupSource[]> {
    const backupSources = await this.getRawBackupSources();
    return Object.values(backupSources);
  }

  async getBackupSource(id: string): Promise<BackupSource | null> {
    const backupSources = await this.getRawBackupSources();
    return backupSources[id] || null;
  }

  async saveBackupSource(backupSource: BackupSource): Promise<BackupSource> {
    const backupSources = await this.getRawBackupSources();
    const completeBackupSource: BackupSource = {
      ...backupSource,
      id: backupSource.id || createId(), // Generate ID if not provided
    };
    const updatedBackupSources = {
      ...backupSources,
      [completeBackupSource.id]: completeBackupSource,
    };
    await this.store.set(BACKUP_SOURCES_KEY, updatedBackupSources);
    await this.store.save();
    return completeBackupSource;
  }

  async deleteBackupSource(id: string): Promise<void> {
    const backupSources = await this.getRawBackupSources();
    if (backupSources[id]) {
      delete backupSources[id];
      await this.store.set(BACKUP_SOURCES_KEY, backupSources);
    }
    await this.store.save();
  }
}
