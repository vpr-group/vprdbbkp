<script lang="ts">
  import { onMount } from "svelte";
  import {
    StoreService,
    type BackupSource,
    type StorageProvider,
  } from "../services/store";
  import DropdownMenu from "./DropdownMenu.svelte";
  import Button from "./Button.svelte";
  import Dialog from "./Dialog.svelte";
  import Separation from "./Separation.svelte";
  import DialogActions from "./DialogActions.svelte";

  interface Props {
    backupKey: string;
    onrestore?: (backupSrouce: BackupSource) => void;
  }

  const { backupKey, onrestore }: Props = $props();
  const storeService = new StoreService();

  let backupSources = $state<BackupSource[]>([]);
  let openMenu = $state(false);
  let openDialog = $state(false);

  const loadBackupSources = async () => {
    await storeService.waitForInitialized();
    backupSources = await storeService.getBackupSources();
  };

  onMount(() => {
    loadBackupSources();
  });
</script>

<DropdownMenu
  bind:open={openMenu}
  icon="arrow-right"
  label="Restore"
  items={backupSources}
  align="end"
>
  {#snippet item(backupSource)}
    <Dialog
      icon="arrow-right"
      label={backupSource.name}
      buttonStyle={{
        backgroundColor: "var(--color-light-grey)",
        color: "black",
      }}
    >
      <div class="restore-dropdown__content">
        <Separation label="Restoration" subLabel={backupKey} />

        <p>
          You are about ro override the following database: <strong>
            {backupSource.database}
          </strong>
        </p>

        <DialogActions>
          <Button icon="cross" onclick={() => (openMenu = false)}>
            Cancel
          </Button>
          <Button
            icon="arrow-right"
            onclick={() => {
              openMenu = false;
              onrestore?.(backupSource);
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
