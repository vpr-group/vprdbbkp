<script lang="ts">
  import BackupSourceCard from "../components/BackupSourceCard.svelte";
  import BackupSourceDialog from "../components/BackupSourceDialog.svelte";
  import Grid from "../components/Grid.svelte";
  import Separation from "../components/Separation.svelte";
  import StorageProviderCard from "../components/StorageProviderCard.svelte";
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

{#if isLoading}
  <span>Is Loading</span>
{:else}
  {#snippet storageProviderActions()}
    <StorageProviderDialog
      onsubmit={async (storageProvider) => {
        await storeService.saveStorageProvider(storageProvider);
        loadProjects();
      }}
    />
  {/snippet}

  <Separation label="Storage Providers" sideSection={storageProviderActions} />

  <Grid>
    {#each storageProviders as storageProvider}
      <StorageProviderCard {storageProvider} />
    {/each}
  </Grid>

  {#snippet backupSourcesActions()}
    <BackupSourceDialog
      onsubmit={async (backupSource) => {
        await storeService.saveBackupSource(backupSource);
        loadBackupSources();
      }}
    />
  {/snippet}

  <Separation label="Backup Sources" sideSection={backupSourcesActions} />

  <Grid>
    {#each backupSources as backupSource}
      <BackupSourceCard {backupSource} />
    {/each}
  </Grid>
{/if}
