<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type SourceConfig } from "../services/store";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Checkbox from "./Checkbox.svelte";

  interface Props {
    backupKey: string;
    onrestore?: (value: {
      sourceConfig: SourceConfig;
      dropDatabase: boolean;
    }) => void;
  }

  const { onrestore }: Props = $props();
  const storeService = new StoreService();

  let sourcesConfig = $state<SourceConfig[]>([]);
  const dialogStates = $state<Record<string, boolean>>({});
  let isMainDialogOpen = $state(false);
  let dropDatabase = $state(false);

  const loadSourceConfigs = async () => {
    await storeService.waitForInitialized();
    sourcesConfig = await storeService.getSourceConfigs();

    sourcesConfig.forEach((config) => {
      dialogStates[config.id] = false;
    });
  };

  onMount(() => {
    loadSourceConfigs();
  });
</script>

<Dialog bind:open={isMainDialogOpen} title="Restore Source" icon="arrow-right">
  {#each sourcesConfig as sourceConfig}
    <Dialog
      bind:open={dialogStates[sourceConfig.id]}
      icon="arrow-right"
      label={sourceConfig.name}
      title="Restoration"
      buttonStyle={{
        justifyContent: "space-between",
        backgroundColor: "var(--color-light-grey)",
        color: "black",
      }}
    >
      <p>
        You are about ro override the following data source: <strong>
          {sourceConfig.name}
        </strong>
      </p>

      <Checkbox bind:checked={dropDatabase} label="Drop Database" />

      <DialogActions>
        <Button
          icon="cross"
          onclick={() => (dialogStates[sourceConfig.id] = false)}
        >
          Cancel
        </Button>
        <Button
          icon="arrow-right"
          onclick={() => {
            dialogStates[sourceConfig.id] = false;
            isMainDialogOpen = false;
            onrestore?.({ sourceConfig, dropDatabase });
          }}>Continue</Button
        >
      </DialogActions>
    </Dialog>
  {/each}
</Dialog>

<style lang="scss">
  p {
    margin: 0;
  }
</style>
