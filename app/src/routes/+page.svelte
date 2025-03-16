<script lang="ts">
  import SourceConfigCard from "../components/SourceConfigCard.svelte";
  import SourceConfigDialog from "../components/SourceConfigDialog.svelte";
  import Grid from "../components/Grid.svelte";
  import Separation from "../components/Separation.svelte";
  import StorageConfigCard from "../components/StorageConfigCard.svelte";
  import StorageConfigDialog from "../components/StorageConfigDialog.svelte";
  import {
    StoreService,
    type SourceConfig,
    type StorageConfig,
  } from "../services/store";
  import { onMount } from "svelte";

  const storeService = new StoreService();

  let isLoading = $state(true);
  let storageConfigs = $state<StorageConfig[]>([]);
  let sourceConfigs = $state<SourceConfig[]>([]);

  const loadProjects = async () => {
    await storeService.waitForInitialized();
    storeService.getStorageConfigs().then((res) => {
      storageConfigs = res;
    });
  };

  const loadBackupSources = async () => {
    await storeService.waitForInitialized();
    storeService.getSourceConfigs().then((res) => {
      sourceConfigs = res;
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
  {#snippet storageActions()}
    <StorageConfigDialog
      onsubmit={async (storageProvider) => {
        await storeService.saveStorageConfig(storageProvider);
        loadProjects();
      }}
    />
  {/snippet}

  <Separation
    label="Storages"
    subLabel={`${storageConfigs.length} items`}
    sideSection={storageActions}
  />

  <Grid>
    {#each storageConfigs as storageProvider}
      <StorageConfigCard storageConfig={storageProvider} />
    {/each}
  </Grid>

  {#snippet sourcesActions()}
    <SourceConfigDialog
      onsubmit={async (backupSource) => {
        await storeService.saveSourceConfig(backupSource);
        loadBackupSources();
      }}
    />
  {/snippet}

  <Separation
    label="Sources"
    subLabel={`${sourceConfigs.length} items`}
    sideSection={sourcesActions}
  />

  <Grid>
    {#each sourceConfigs as backupSource}
      <SourceConfigCard sourceConfig={backupSource} />
    {/each}
  </Grid>
{/if}
