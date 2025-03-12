<script lang="ts">
  import { onMount } from "svelte";
  import type { StorageProvider } from "../services/store";
  import Card from "./Card.svelte";
  import { ActionsService, type BackupListItem } from "../services/actions";

  interface Props {
    storageProvider: StorageProvider;
  }

  const actionsService = new ActionsService();
  const { storageProvider }: Props = $props();

  let loadingBackups = $state(true);
  let backups = $state<BackupListItem[]>([]);

  const loadBackups = async () => {
    try {
      if (!storageProvider) return;
      backups = await actionsService.listBackups(storageProvider);
    } finally {
      loadingBackups = false;
    }
  };

  onMount(() => {
    loadBackups();
  });
</script>

<Card href={`/storage-providers/${storageProvider.id}`}>
  <span>{storageProvider.type}</span>
  <h3>{storageProvider.name}</h3>

  {#if loadingBackups}
    <span>Loading</span>
  {:else}
    <span>{backups.length} backups</span>
  {/if}
</Card>

<style lang="scss">
  h3 {
    font-size: 2rem;
    margin: 0;
    text-transform: uppercase;
  }
</style>
