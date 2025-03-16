<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageConfig } from "../services/store";
  import DropdownMenu from "./DropdownMenu.svelte";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import Separation from "./Separation.svelte";
  import DialogActions from "./DialogActions.svelte";

  interface Props {
    onbackup?: (storageProvider: StorageConfig) => void;
  }

  const { onbackup }: Props = $props();
  const storeService = new StoreService();

  let storageProviders = $state<StorageConfig[]>([]);
  let open = $state(false);

  const loadStorageProviders = async () => {
    await storeService.waitForInitialized();
    storageProviders = await storeService.getStorageConfigs();
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
    <Dialog
      icon="arrow-right"
      label={storageProvider.name}
      buttonStyle={{
        backgroundColor: "var(--color-light-grey)",
        color: "black",
      }}
    >
      <div class="restore-dropdown__content">
        <Separation label="Backup" />

        <p>
          You are about backup in the following bucket: <strong>
            {storageProvider.bucket}
          </strong>
        </p>

        <DialogActions>
          <Button icon="cross" onclick={() => (open = false)}>Cancel</Button>
          <Button
            icon="arrow-right"
            onclick={() => {
              open = false;
              onbackup?.(storageProvider);
            }}
          >
            Continue
          </Button>
        </DialogActions>
      </div>
    </Dialog>
  {/snippet}
</DropdownMenu>
