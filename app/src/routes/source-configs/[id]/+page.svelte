<script lang="ts">
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { StoreService, type SourceConfig } from "../../../services/store";
  import Separation from "../../../components/Separation.svelte";
  import BackupSourceDialog from "../../../components/SourceConfigDialog.svelte";
  import Button from "../../../components/Button.svelte";
  import Table from "../../../components/Table.svelte";
  import { goto } from "$app/navigation";
  import type { CSSProperties } from "../../../utils/css";
  import { ActionsService } from "../../../services/actions";
  import StatusDot from "../../../components/StatusDot.svelte";
  import BackupDropdown from "../../../components/BackupDropdown.svelte";
  import {
    notificationsStore,
    type Notification,
  } from "../../../components/Notifications.svelte";

  const { addNotification, removeNotification } = notificationsStore;
  const actionService = new ActionsService();
  const storeService = new StoreService();

  let sourceConfig = $state<SourceConfig | null>(null);
  let connected = $state(false);

  const loadSourceConfig = async () => {
    await storeService.waitForInitialized();
    sourceConfig = await storeService.getSourceConfig(page.params.id);
  };

  const cellLabelStyle: CSSProperties = {
    color: "var(--color-grey)",
  };

  onMount(async () => {
    loadSourceConfig();
  });

  $effect(() => {
    if (!sourceConfig) return;
    let notification: Notification | undefined = undefined;

    actionService.verifySourceConnection(sourceConfig).then((res) => {
      connected = res.connected;

      if (connected) {
        notification = addNotification({
          title: "Source Connected",
          status: "success",
          dismissTimeout: 3000,
        });
      } else {
        notification = addNotification({
          title: "Source Not Connected",
          status: "warning",
          dismissTimeout: 3000,
        });
      }
    });

    return () => {
      if (notification) {
        removeNotification(notification.id);
      }
    };
  });
</script>

{#snippet sideSection()}
  {#if sourceConfig}
    <Button
      onclick={async () => {
        if (!sourceConfig) return;
        await storeService.deleteSourceConfig(sourceConfig?.id);
        goto("/");
      }}
      icon="cross">Delete</Button
    >
    <BackupSourceDialog
      {sourceConfig}
      onsubmit={async (sourceConfig) => {
        await storeService.saveSourceConfig(sourceConfig);
        loadSourceConfig();
      }}
    />
    <BackupDropdown
      onbackup={async (storageConfig) => {
        if (!sourceConfig) return;

        const backupInProgressNotification = addNotification({
          title: "Backup in progress...",
          status: "info",
          dismissTimeout: null,
        });

        try {
          await actionService.backup(sourceConfig, storageConfig);
          removeNotification(backupInProgressNotification.id);
          addNotification({
            title: "Backup successful",
            status: "success",
          });
        } catch (error) {
          removeNotification(backupInProgressNotification.id);
          addNotification({
            title: "Backup failed",
            message: `${error}`,
            status: "error",
          });
        }
      }}
    />
  {/if}
{/snippet}

{#snippet label()}
  {#if sourceConfig}
    <div class="backup-source__label">
      <StatusDot status={connected ? "success" : undefined} />
      {sourceConfig.name}
    </div>
  {/if}
{/snippet}

{#if sourceConfig}
  <Separation {label} subLabel={sourceConfig.type} {sideSection} />

  <Table
    rows={[
      {
        cells: [
          { label: "database type", width: "10rem", style: cellLabelStyle },
          { label: sourceConfig.type || "-" },
        ],
      },
      {
        cells: [
          { label: "database", width: "10rem", style: cellLabelStyle },
          { label: sourceConfig.database || "-" },
        ],
      },
      {
        cells: [
          { label: "host", width: "10rem", style: cellLabelStyle },
          { label: sourceConfig.host || "-" },
        ],
      },
      {
        cells: [
          { label: "port", width: "10rem", style: cellLabelStyle },
          { label: sourceConfig.port.toString() || "-" },
        ],
      },
      {
        cells: [
          { label: "username", width: "10rem", style: cellLabelStyle },
          { label: sourceConfig.username || "-" },
        ],
      },
      {
        cells: [
          { label: "password", width: "10rem", style: cellLabelStyle },
          {
            label:
              (sourceConfig.password || "")
                .split("")
                .map((it) => "•")
                .join("") || "-",
          },
        ],
      },
    ]}
  />
{/if}

<style lang="scss">
  .backup-source {
    &__label {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }
  }
</style>
