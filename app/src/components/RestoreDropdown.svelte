<script lang="ts">
  import { onMount } from "svelte";
  import {
    StoreService,
    type BackupSource,
    type StorageProvider,
  } from "../services/store";
  import DropdownMenu from "./DropdownMenu.svelte";
  import Button from "./Button.svelte";

  interface Props {
    onrestore?: (backupSrouce: BackupSource) => void;
  }

  const { onrestore }: Props = $props();
  const storeService = new StoreService();

  let backupSources = $state<BackupSource[]>([]);
  let open = $state(false);

  const loadBackupSources = async () => {
    await storeService.waitForInitialized();
    backupSources = await storeService.getBackupSources();
  };

  onMount(() => {
    loadBackupSources();
  });
</script>

<DropdownMenu
  bind:open
  icon="arrow-right"
  label="Restore"
  items={backupSources}
  align="end"
>
  {#snippet item(backupSource)}
    <Button
      preIcon="arrow-right"
      style={{ backgroundColor: "var(--color-light-grey)", color: "black" }}
      onclick={() => {
        open = false;
        onrestore?.(backupSource);
      }}
    >
      {backupSource.name}
    </Button>
  {/snippet}
</DropdownMenu>
