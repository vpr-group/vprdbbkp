<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type DatabaseConfig } from "../services/store";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Checkbox from "./Checkbox.svelte";

  interface Props {
    backupKey: string;
    onrestore?: (value: {
      databaseConfig: DatabaseConfig;
      dropDatabase: boolean;
    }) => void;
  }

  const { onrestore, backupKey }: Props = $props();
  const storeService = new StoreService();

  let sourcesConfig = $state<DatabaseConfig[]>([]);
  const dialogStates = $state<Record<string, boolean>>({});
  let isMainDialogOpen = $state(false);
  let dropDatabase = $state(false);

  const loaddatabaseConfigs = async () => {
    await storeService.waitForInitialized();
    sourcesConfig = await storeService.getDatabaseConfigs();

    sourcesConfig.forEach((config) => {
      dialogStates[config.id] = false;
    });
  };

  onMount(() => {
    loaddatabaseConfigs();
  });
</script>

<Dialog bind:open={isMainDialogOpen} title="Restore Source" icon="arrow-right">
  {#each sourcesConfig as databaseConfig}
    <Dialog
      bind:open={dialogStates[databaseConfig.id]}
      icon="arrow-right"
      label={databaseConfig.name}
      title="Restoration"
      buttonStyle={{
        justifyContent: "space-between",
        backgroundColor: "var(--color-light-grey)",
        color: "black",
      }}
    >
      <p>
        You are about ro override the following data source: <strong>
          {databaseConfig.name}
        </strong>
        with <strong>{backupKey}</strong>
      </p>

      <Checkbox bind:checked={dropDatabase} label="Drop Database" />

      <DialogActions>
        <Button
          icon="cross"
          onclick={() => (dialogStates[databaseConfig.id] = false)}
        >
          Cancel
        </Button>
        <Button
          icon="arrow-right"
          onclick={() => {
            dialogStates[databaseConfig.id] = false;
            isMainDialogOpen = false;
            onrestore?.({ databaseConfig, dropDatabase });
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
