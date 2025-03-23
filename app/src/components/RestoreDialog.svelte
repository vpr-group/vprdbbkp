<script lang="ts">
  import { onMount } from "svelte";
  import {
    StoreService,
    type SourceConfig,
    type StorageConfig,
  } from "../services/store";
  import DropdownMenu from "./DropdownMenu.svelte";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import Separation from "./Separation.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Checkbox from "./Checkbox.svelte";

  interface Props {
    backupKey: string;
    onrestore?: (value: {
      sourceConfig: SourceConfig;
      dropDatabase: boolean;
    }) => void;
  }

  const { backupKey, onrestore }: Props = $props();
  const storeService = new StoreService();

  let sourcesConfig = $state<SourceConfig[]>([]);
  let openMenu = $state(false);
  let openDialog = $state(false);
  let dropDatabase = $state(false);

  const loadSourceConfigs = async () => {
    await storeService.waitForInitialized();
    sourcesConfig = await storeService.getSourceConfigs();
  };

  onMount(() => {
    loadSourceConfigs();
  });
</script>

<Dialog bind:open={openMenu} icon="arrow-right">
  <Separation label="Restore Source" />
  <div class="restore-dialog__sources">
    {#each sourcesConfig as sourceConfig}
      <Dialog
        icon="arrow-right"
        label={sourceConfig.name}
        buttonStyle={{
          justifyContent: "space-between",
          backgroundColor: "var(--color-light-grey)",
          color: "black",
        }}
      >
        <div class="restore-dialog__content">
          <Separation label="Restoration" subLabel={backupKey} />

          <p>
            You are about ro override the following database: <strong>
              {sourceConfig.database}
            </strong>
          </p>

          <Checkbox bind:checked={dropDatabase} label="Drop Database" />

          <DialogActions>
            <Button icon="cross" onclick={() => (openMenu = false)}>
              Cancel
            </Button>
            <Button
              icon="arrow-right"
              onclick={() => {
                openMenu = false;
                onrestore?.({ sourceConfig, dropDatabase });
              }}>Continue</Button
            >
          </DialogActions>
        </div>
      </Dialog>
    {/each}
  </div>
</Dialog>

<style lang="scss">
  .restore-dialog {
    &__content {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;

      p {
        margin: 0;
      }
    }

    &__sources {
      padding: 1rem 0;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }
  }
</style>
