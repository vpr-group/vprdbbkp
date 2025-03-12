<script lang="ts">
  import BackupSourceDialog from "../components/BackupSourceDialog.svelte";
  import StorageProviderDialog from "../components/StorageProviderDialog.svelte";
  import {
    StoreService,
    type BackupSource,
    type StorageProvider,
  } from "../services/store";
  import { onMount } from "svelte";

  const storeService = new StoreService();

  let isLoading = $state(true);
  let storageProviders = $state<StorageProvider[]>([]);
  let backupSources = $state<BackupSource[]>([]);

  const loadProjects = async () => {
    await storeService.waitForInitialized();
    storeService.getStorageProviders().then((res) => {
      storageProviders = res;
    });
  };

  const loadBackupSources = async () => {
    await storeService.waitForInitialized();
    storeService.getBackupSources().then((res) => {
      backupSources = res;
    });
  };

  onMount(() => {
    Promise.all([loadProjects(), loadBackupSources()]).finally(() => {
      isLoading = false;
    });
  });
</script>

<main class="container">
  {#if isLoading}
    <span>Is Loading</span>
  {:else}
    <StorageProviderDialog
      onsubmit={async (storageProvider) => {
        await storeService.saveStorageProvider(storageProvider);
        loadProjects();
      }}
    />

    <BackupSourceDialog
      onsubmit={async (backupSource) => {
        await storeService.saveBackupSource(backupSource);
        loadBackupSources();
      }}
    />

    {#each storageProviders as storageProvider}
      <a href={`/storage-providers/${storageProvider.id}`}>
        {storageProvider.name}
      </a>
    {/each}

    {#each backupSources as backupSource}
      <a href={`/backup-sources/${backupSource.id}`}>
        {backupSource.name}
      </a>
    {/each}
  {/if}
</main>
