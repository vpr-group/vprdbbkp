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
  const dialogStates = $state<Record<string, boolean>>({});

  const loadStorageProviders = async () => {
    await storeService.waitForInitialized();
    storageConfigs = await storeService.getStorageConfigs();

    storageConfigs.forEach((config) => {
      dialogStates[config.id] = false;
    });
  };

  onMount(() => {
    loadStorageProviders();
  });
</script>

<Dialog bind:open={isMainDialogOpen} icon="upload" title="Backup to">
  {#each storageConfigs as storageConfig}
    <Dialog
      open={dialogStates[storageConfig.id]}
      onopenchange={(open) => {
        dialogStates[storageConfig.id] = open;
      }}
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
        You are about backup in the following storage: <strong>
          {storageConfig.name}
        </strong>
      </p>

      <DialogActions>
        <Button
          icon="cross"
          onclick={() => (dialogStates[storageConfig.id] = false)}
        >
          Cancel
        </Button>
        <Button
          icon="arrow-right"
          onclick={() => {
            dialogStates[storageConfig.id] = false;
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
