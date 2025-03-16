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

<Card
  href={`/storage-configs/${storageConfig.id}`}
  title={storageConfig.name}
  subTitle={`${backups.length} backups`}
></Card>
