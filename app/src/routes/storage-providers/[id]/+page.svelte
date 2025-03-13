<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageProvider } from "../../../services/store";
  import { page } from "$app/state";
  import StorageProviderDialog from "../../../components/StorageProviderDialog.svelte";
  import {
    ActionsService,
    type BackupListItem,
  } from "../../../services/actions";
  import Table, { type Cell, type Row } from "../../../components/Table.svelte";
  import Separation from "../../../components/Separation.svelte";
  import Button from "../../../components/Button.svelte";
  import RestoreDropdown from "../../../components/RestoreDropdown.svelte";

  const storeService = new StoreService();
  const actionsService = new ActionsService();

  let storageProvider = $state<StorageProvider | null>(null);
  let backups = $state<BackupListItem[]>([]);

  const loadStorageProvider = async () => {
    await storeService.waitForInitialized();
    storageProvider = await storeService.getStorageProvider(page.params.id);
  };

  const loadBackups = async () => {
    if (!storageProvider) return;
    backups = await actionsService.listBackups(storageProvider);
  };

  onMount(async () => {
    loadStorageProvider();
  });

  $effect(() => {
    loadBackups();
  });
</script>

{#snippet sideSection()}
  {#if storageProvider}
    <Button onclick={() => loadBackups()} icon="cross">Delete</Button>
    <StorageProviderDialog
      {storageProvider}
      onsubmit={async (storageProvider) => {
        await storeService.saveStorageProvider(storageProvider);
        loadStorageProvider();
      }}
    />
    <Button onclick={() => loadBackups()} icon="reload">Refresh</Button>
  {/if}
{/snippet}

{#if storageProvider}
  <Separation
    label={storageProvider.name}
    subLabel={`${storageProvider.type} - ${storageProvider.bucket}`}
    {sideSection}
  />

  {#snippet actions(cell: Cell, row?: Row)}
    <RestoreDropdown
      backupKey={cell.label || ""}
      onrestore={(backupSource) => {
        if (!storageProvider) return;
        actionsService.restoreBackup(
          cell.label || "",
          backupSource,
          storageProvider
        );
        console.log(cell.label);
        console.log(backupSource);
      }}
    />
  {/snippet}

  <Table
    headers={[
      { label: "#", width: "3rem" },
      { label: "key", width: "40%" },
      { label: "type", width: "10%" },
      { label: "Timestamp" },
    ]}
    rows={backups.map((row, index) => ({
      cells: [
        { label: (index + 1).toString().padStart(2, "0") },
        { label: row.key },
        { label: row.backupType },
        { label: row.timestamp },
        {
          label: row.key,
          renderHandler: actions,
          style: {
            padding: "0 0.5rem",
            alignItems: "center",
            justifyContent: "flex-end",
            flex: "1 1 auto",
            width: "100%",
          },
        },
      ],
    }))}
  />
{/if}
