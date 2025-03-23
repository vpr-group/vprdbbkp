<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type StorageConfig } from "../../../services/store";
  import { page } from "$app/state";
  import StorageProviderDialog from "../../../components/StorageConfigDialog.svelte";
  import { ActionsService, type Entry } from "../../../services/actions";
  import Table, { type Cell, type Row } from "../../../components/Table.svelte";
  import Separation from "../../../components/Separation.svelte";
  import Button from "../../../components/Button.svelte";
  import RestoreDropdown from "../../../components/RestoreDialog.svelte";
  import { notificationsStore } from "../../../components/Notifications.svelte";
  import { goto } from "$app/navigation";
  import {
    extractDateTimeFromEntryName,
    formatDate,
  } from "../../../utils/dates";
  import Icon from "../../../components/Icon.svelte";
  import StorageConfigCard from "../../../components/StorageConfigCard.svelte";

  const { addNotification, removeNotification } = notificationsStore;
  const storeService = new StoreService();
  const actionsService = new ActionsService();

  let storageConfig = $state<StorageConfig | null>(null);
  let backups = $state<Entry[]>([]);

  const sortedBackups = $derived<{ entry: Entry; date: Date | null }[]>(
    backups
      .map((it) => ({
        entry: it,
        date: extractDateTimeFromEntryName(it.path),
      }))
      .sort((a, b) => (a.date && b.date ? (a.date > b.date ? -1 : 1) : -1))
  );

  const loadStorageConfig = async () => {
    await storeService.waitForInitialized();
    storageConfig = await storeService.getStorageConfig(page.params.id);
  };

  const loadBackups = async () => {
    if (!storageConfig) return;

    try {
      backups = await actionsService.list(storageConfig);
    } catch (error) {
      addNotification({
        title: "Failed to load backups",
        message: `${error}`,
        status: "error",
      });
    }
  };

  onMount(async () => {
    loadStorageConfig();
  });

  $effect(() => {
    loadBackups();
  });
</script>

{#snippet sideSection()}
  {#if storageConfig}
    <Button
      icon="cross"
      onclick={async () => {
        if (!storageConfig) return;
        await storeService.deleteStorageConfig(storageConfig.id);
        goto("/");
      }}
    />

    <StorageProviderDialog
      {storageConfig}
      onsubmit={async (storageProvider) => {
        await storeService.saveStorageConfig(storageProvider);
        loadStorageConfig();
      }}
    />
    <Button onclick={() => loadBackups()} icon="reload" />
  {/if}
{/snippet}

{#if storageConfig}
  <Separation subLabel={`${storageConfig.type}`} {sideSection}>
    {#snippet label()}
      <Icon icon={storageConfig?.type === "s3" ? "cloud" : "hard-drive"} />
      {storageConfig?.name}
    {/snippet}
  </Separation>

  <StorageConfigCard {storageConfig} hideTitle />

  <Separation label="Entries">
    {#snippet sideSection()}
      <!-- <Button icon="arrow-right">Restore Latest</Button> -->
    {/snippet}
  </Separation>

  {#snippet actions(cell: Cell, row?: Row)}
    <RestoreDropdown
      backupKey={cell.label || ""}
      onrestore={async ({ sourceConfig, dropDatabase }) => {
        if (!storageConfig) return;

        const progressNotifications = addNotification({
          title: "Restore in progress...",
          status: "info",
          dismissTimeout: null,
        });

        try {
          await actionsService.restore(
            cell.label || "",
            sourceConfig,
            storageConfig,
            dropDatabase
          );

          removeNotification(progressNotifications.id);
          addNotification({
            title: "Restore successful",
            status: "success",
          });
        } catch (error) {
          removeNotification(progressNotifications.id);
          addNotification({
            title: "Failed to restore",
            message: `${error}`,
            status: "error",
          });
        }
      }}
    />
  {/snippet}

  <Table
    headers={[
      { label: "#", width: "3rem" },
      { label: "key", width: "40%" },
      { label: "date", width: "20%" },
    ]}
    rows={sortedBackups.map((row, index) => ({
      cells: [
        { label: (index + 1).toString() },
        { label: row.entry.path },
        { label: row.date ? formatDate(row.date) : "-" },
        {
          label: row.entry.path,
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
