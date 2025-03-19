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

<DropdownMenu
  bind:open={openMenu}
  icon="arrow-right"
  label="Restore"
  items={sourcesConfig}
  align="end"
>
  {#snippet item(sourceConfig)}
    <Dialog
      icon="arrow-right"
      label={sourceConfig.name}
      buttonStyle={{
        backgroundColor: "var(--color-light-grey)",
        color: "black",
      }}
    >
      <div class="restore-dropdown__content">
        <Separation label="Restoration" subLabel={backupKey} />

        <p>
          You are about ro override the following database: <strong>
            {sourceConfig.database}
          </strong>
        </p>

        <fieldset>
          <label for="">Drop Database & Terminate connections</label>
          <input type="checkbox" bind:checked={dropDatabase} />
        </fieldset>

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
  {/snippet}
</DropdownMenu>

<style lang="scss">
  .restore-dropdown {
    &__content {
      display: flex;
      flex-direction: column;
      align-items: flex-start;
      gap: 1rem;

      p {
        margin: 0;
      }
    }
  }
</style>
