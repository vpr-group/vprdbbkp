<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageConfig } from "../services/store";
  import DropdownMenu from "./DropdownMenu.svelte";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import Separation from "./Separation.svelte";
  import DialogActions from "./DialogActions.svelte";

  interface Props {
    onbackup?: (storageConfig: StorageConfig) => void;
  }

  const { onbackup }: Props = $props();
  const storeService = new StoreService();

  let storageConfigs = $state<StorageConfig[]>([]);
  let open = $state(false);

  const loadStorageProviders = async () => {
    await storeService.waitForInitialized();
    storageConfigs = await storeService.getStorageConfigs();
  };

  onMount(() => {
    loadStorageProviders();
  });
</script>

<Dialog icon="upload">
  <Separation label="Backup to" />
  <div class="backup-dialog__storages">
    {#each storageConfigs as storageConfig}
      <Dialog
        icon="arrow-right"
        label={storageConfig.name}
        buttonStyle={{
          justifyContent: "space-between",
          backgroundColor: "var(--color-light-grey)",
          color: "black",
        }}
      >
        <div class="restore-dropdown__content">
          <Separation label="Backup" />

          <p>
            You are about backup in the following bucket: <strong>
              {storageConfig.bucket}
            </strong>
          </p>

          <DialogActions>
            <Button icon="cross" onclick={() => (open = false)}>Cancel</Button>
            <Button
              icon="arrow-right"
              onclick={() => {
                open = false;
                onbackup?.(storageConfig);
              }}
            >
              Continue
            </Button>
          </DialogActions>
        </div>
      </Dialog>
    {/each}
  </div>
</Dialog>

<!-- <DropdownMenu
  bind:open
  icon="upload"
  label=""
  items={storageConfig}
  align="end"
>
  {#snippet item(storageConfig)}
    
  {/snippet}
</DropdownMenu> -->

<style lang="scss">
  .backup-dialog {
    &__storages {
      padding: 1rem 0;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
  }
</style>
