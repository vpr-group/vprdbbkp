<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageProvider } from "../../../services/store";
  import { page } from "$app/state";
  import StorageProviderDialog from "../../../components/StorageProviderDialog.svelte";
  import {
    ActionsService,
    type BackupListItem,
  } from "../../../services/actions";

  const storeService = new StoreService();
  const actionsService = new ActionsService();

  let storageProvider = $state<StorageProvider | null>(null);
  let backups = $state<BackupListItem[]>([]);

  const loadStorageProvider = async () => {
    await storeService.waitForInitialized();
    storageProvider = await storeService.getStorageProvider(page.params.id);
  };

  const loadBackups = async () => {
    if (!storageProvider) return;
    backups = await actionsService.listBackups(storageProvider);
  };

  onMount(async () => {
    loadStorageProvider();
  });

  $effect(() => {
    loadBackups();
  });
</script>

{#if storageProvider}
  <div>
    <h1>{storageProvider.name}</h1>
    <span>{storageProvider.type} - {storageProvider.bucket}</span>
  </div>

  <StorageProviderDialog
    {storageProvider}
    onsubmit={async (storageProvider) => {
      await storeService.saveStorageProvider(storageProvider);
      loadStorageProvider();
    }}
  />
  <button onclick={() => loadBackups()}>Refresh backups</button>

  {#each backups as backup}
    <span>{backup.key}</span>
  {/each}
{/if}
