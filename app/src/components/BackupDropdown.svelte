<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageProvider } from "../services/store";
  import DropdownMenu from "./DropdownMenu.svelte";
  import Button from "./Button.svelte";

  interface Props {
    onbackup?: (storageProvider: StorageProvider) => void;
  }

  const { onbackup }: Props = $props();
  const storeService = new StoreService();

  let storageProviders = $state<StorageProvider[]>([]);
  let open = $state(false);

  const loadStorageProviders = async () => {
    await storeService.waitForInitialized();
    storageProviders = await storeService.getStorageProviders();
  };

  onMount(() => {
    loadStorageProviders();
  });
</script>

<DropdownMenu
  bind:open
  icon="upload"
  label="Backup"
  items={storageProviders}
  align="end"
>
  {#snippet item(storageProvider)}
    <Button
      preIcon="arrow-right"
      style={{ backgroundColor: "var(--color-light-grey)", color: "black" }}
      onclick={() => {
        open = false;
        onbackup?.(storageProvider);
      }}
    >
      {storageProvider.name}
    </Button>
  {/snippet}
</DropdownMenu>
