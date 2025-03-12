<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageProvider } from "../../../services/store";
  import { page } from "$app/state";
  import StorageProviderDialog from "../../../components/StorageProviderDialog.svelte";
  import { ActionsService } from "../../../services/actions";

  const storeService = new StoreService();
  const actionsService = new ActionsService();

  let storageProvider = $state<StorageProvider | null>(null);

  const loadStorageProvider = async () => {
    await storeService.waitForInitialized();
    storageProvider = await storeService.getStorageProvider(page.params.id);
  };

  onMount(async () => {
    loadStorageProvider();
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
  <button
    onclick={() => {
      if (!storageProvider) return;
      actionsService.listBackups(storageProvider);
    }}
  >
    Refresh backups</button
  >
{/if}
