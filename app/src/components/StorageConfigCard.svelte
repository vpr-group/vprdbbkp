<script lang="ts">
  import { onMount } from "svelte";
  import type { StorageConfig } from "../services/store";
  import Card from "./Card.svelte";
  import { ActionsService, type Entry } from "../services/actions";
  import Icon from "./Icon.svelte";

  interface Props {
    hideTitle?: boolean;
    storageConfig: StorageConfig;
  }

  const actionsService = new ActionsService();
  const { storageConfig, hideTitle }: Props = $props();

  let loadingBackups = $state(true);
  let backups = $state<Entry[]>([]);

  const entriesLabel = $derived(
    backups.length > 1 ? `${backups.length} Entries` : `${backups.length} Entry`
  );

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

{#snippet title()}
  {#if !hideTitle}
    <div class="storage-config-card__title">
      <Icon icon={storageConfig.type === "s3" ? "cloud" : "hard-drive"} />
      <span>{storageConfig.name}</span>
    </div>
  {/if}
{/snippet}

<Card href={`/storage-configs/${storageConfig.id}`} {title}>
  <div class="storage-config-card__content">
    {#if !hideTitle}
      <span class="storage-config-card__type">
        {#if storageConfig.type === "local"}
          Local • {entriesLabel}
        {:else if storageConfig.type === "s3"}
          S3 • {entriesLabel}
        {/if}
      </span>
    {/if}

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
      flex: 1 1 auto;
      justify-content: space-between;
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
