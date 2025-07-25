<script lang="ts">
  import DatabaseConfigCard from "../components/DatabaseConfigCard.svelte";
  import DatabaseConfigDialog from "../components/DatabaseConfigDialog.svelte";
  import Grid from "../components/Grid.svelte";
  import Separation from "../components/Separation.svelte";
  import StorageConfigCard from "../components/StorageConfigCard.svelte";
  import StorageConfigDialog from "../components/StorageConfigDialog.svelte";
  import {
    StoreService,
    type DatabaseConfig,
    type StorageConfig,
  } from "../services/store";
  import { onMount } from "svelte";

  const storeService = new StoreService();

  let isLoading = $state(true);
  let storageConfigs = $state<StorageConfig[]>([]);
  let databaseConfigs = $state<DatabaseConfig[]>([]);

  const loadStorageConfigs = async () => {
    await storeService.waitForInitialized();
    storeService.getStorageConfigs().then((res) => {
      storageConfigs = res;
    });
  };

  const loadDatabaseConfigs = async () => {
    await storeService.waitForInitialized();
    storeService.getDatabaseConfigs().then((res) => {
      databaseConfigs = res;
    });
  };

  onMount(() => {
    Promise.all([loadStorageConfigs(), loadDatabaseConfigs()]).finally(() => {
      isLoading = false;
    });
  });
</script>

{#if isLoading}
  <span>Is Loading</span>
{:else}
  <Separation label="File Storage" subLabel={`${storageConfigs.length} items`}>
    {#snippet sideSection()}
      <StorageConfigDialog
        onsubmit={async (storageProvider) => {
          await storeService.saveStorageConfig(storageProvider);
          loadStorageConfigs();
        }}
      />
    {/snippet}
  </Separation>

  <Grid>
    {#each storageConfigs as storageProvider}
      <StorageConfigCard storageConfig={storageProvider} />
    {/each}
  </Grid>

  <Separation label="Databases" subLabel={`${databaseConfigs.length} items`}>
    {#snippet sideSection()}
      <DatabaseConfigDialog
        onsubmit={async (backupSource) => {
          await storeService.saveDatabaseConfig(backupSource);
          loadDatabaseConfigs();
        }}
      />
    {/snippet}
  </Separation>

  <Grid>
    {#each databaseConfigs as backupSource}
      <DatabaseConfigCard databaseConfig={backupSource} />
    {/each}
  </Grid>
{/if}
