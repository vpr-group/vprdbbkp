<script lang="ts">
  import { onMount } from "svelte";
  import type { StorageConfig } from "../services/store";
  import Card from "./Card.svelte";
  import { ActionsService, type Entry } from "../services/actions";

  interface Props {
    storageConfig: StorageConfig;
  }

  const actionsService = new ActionsService();
  const { storageConfig }: Props = $props();

  let loadingBackups = $state(true);
  let backups = $state<Entry[]>([]);

  const loadBackups = async () => {
    try {
      if (!storageConfig) return;
      backups = await actionsService.list(storageConfig);
    } finally {
      loadingBackups = false;
    }
  };

  onMount(() => {
    loadBackups();
  });
</script>

<Card href={`/storage-configs/${storageConfig.id}`} title={storageConfig.name}>
  <div class="storage-config-card__content">
    <span class="storage-config-card__type">
      {#if storageConfig.type === "local"}
        Local • {backups.length} Entries
      {:else if storageConfig.type === "s3"}
        S3 • {backups.length} Entries
      {/if}
    </span>

    {#if storageConfig.type === "local"}
      <div class="storage-config-card__row">
        <span>Path:</span>
        <span>{storageConfig.root}</span>
      </div>
    {:else if storageConfig.type === "s3"}
      <div class="storage-config-card__row">
        <span>Region:</span>
        <span>{storageConfig.region}</span>
      </div>
      <div class="storage-config-card__row">
        <span>Bucket:</span>
        <span>{storageConfig.bucket}</span>
      </div>
    {/if}
  </div>
</Card>

<style lang="scss">
  .storage-config-card {
    &__content {
      display: flex;
      flex-direction: column;
      font-family: var(--mono-font-family);
    }

    &__type {
      margin-bottom: 1rem;
      color: var(--color-grey);
    }

    &__row {
      display: flex;
      justify-content: space-between;
    }
  }
</style>
