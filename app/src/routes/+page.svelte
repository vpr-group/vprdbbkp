<script lang="ts">
  import BackupSourceCard from "../components/SourceConfigCard.svelte";
  import BackupSourceDialog from "../components/SourceConfigDialog.svelte";
  import Grid from "../components/Grid.svelte";
  import Separation from "../components/Separation.svelte";
  import StorageProviderCard from "../components/StorageConfigCard.svelte";
  import StorageProviderDialog from "../components/StorageConfigDialog.svelte";
  import {
    StoreService,
    type SourceConfig,
    type StorageConfig,
  } from "../services/store";
  import { onMount } from "svelte";

  const storeService = new StoreService();

  let isLoading = $state(true);
  let storageProviders = $state<StorageConfig[]>([]);
  let backupSources = $state<SourceConfig[]>([]);

  const loadProjects = async () => {
    await storeService.waitForInitialized();
    storeService.getStorageConfigs().then((res) => {
      storageProviders = res;
    });
  };

  const loadBackupSources = async () => {
    await storeService.waitForInitialized();
    storeService.getSourceConfigs().then((res) => {
      backupSources = res;
    });
  };

  onMount(() => {
    Promise.all([loadProjects(), loadBackupSources()]).finally(() => {
      isLoading = false;
    });
  });
</script>

{#if isLoading}
  <span>Is Loading</span>
{:else}
  {#snippet storageProviderActions()}
    <StorageProviderDialog
      onsubmit={async (storageProvider) => {
        await storeService.saveStorageConfig(storageProvider);
        loadProjects();
      }}
    />
  {/snippet}

  <Separation
    label="Storage Providers"
    subLabel={`${storageProviders.length} items`}
    sideSection={storageProviderActions}
  />

  <Grid>
    {#each storageProviders as storageProvider}
      <StorageProviderCard storageConfig={storageProvider} />
    {/each}
  </Grid>

  {#snippet backupSourcesActions()}
    <BackupSourceDialog
      onsubmit={async (backupSource) => {
        await storeService.saveSourceConfig(backupSource);
        loadBackupSources();
      }}
    />
  {/snippet}

  <Separation
    label="Backup Sources"
    subLabel={`${backupSources.length} items`}
    sideSection={backupSourcesActions}
  />

  <Grid>
    {#each backupSources as backupSource}
      <BackupSourceCard sourceConfig={backupSource} />
    {/each}
  </Grid>
{/if}
