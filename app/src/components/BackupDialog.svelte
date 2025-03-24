<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageConfig } from "../services/store";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import DialogActions from "./DialogActions.svelte";

  interface Props {
    onbackup?: (storageConfig: StorageConfig) => void;
  }

  const { onbackup }: Props = $props();
  const storeService = new StoreService();

  let storageConfigs = $state<StorageConfig[]>([]);
  let isMainDialogOpen = $state(false);
  let isConfirmationOpen = $state(false);

  const loadStorageProviders = async () => {
    await storeService.waitForInitialized();
    storageConfigs = await storeService.getStorageConfigs();
  };

  onMount(() => {
    loadStorageProviders();
  });
</script>

<Dialog bind:open={isMainDialogOpen} icon="upload" title="Backup to">
  {#each storageConfigs as storageConfig}
    <Dialog
      bind:open={isConfirmationOpen}
      icon="arrow-right"
      title="Backup"
      label={storageConfig.name}
      buttonStyle={{
        justifyContent: "space-between",
        backgroundColor: "var(--color-light-grey)",
        color: "black",
      }}
    >
      <p>
        You are about backup in the following bucket: <strong>
          {storageConfig.bucket}
        </strong>
      </p>

      <DialogActions>
        <Button icon="cross" onclick={() => (isConfirmationOpen = false)}>
          Cancel
        </Button>
        <Button
          icon="arrow-right"
          onclick={() => {
            isConfirmationOpen = false;
            isMainDialogOpen = false;
            onbackup?.(storageConfig);
          }}
        >
          Continue
        </Button>
      </DialogActions>
    </Dialog>
  {/each}
</Dialog>

<style lang="scss">
  p {
    margin: 0;
  }
</style>
