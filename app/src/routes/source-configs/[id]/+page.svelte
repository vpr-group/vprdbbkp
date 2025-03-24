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
  import BackupDialog from "../../../components/BackupDialog.svelte";
  import {
    notificationsStore,
    type Notification,
  } from "../../../components/Notifications.svelte";
  import Badge from "../../../components/Badge.svelte";
  import Icon from "../../../components/Icon.svelte";
  import Dialog from "../../../components/Dialog.svelte";

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

    actionService
      .verifySourceConnection(sourceConfig)
      .then((res) => {
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
      })
      .catch((error) => {
        notification = addNotification({
          title: "Source Connection Failed",
          status: "error",
          message: error,
          dismissTimeout: 6000,
        });
      });

    return () => {
      if (notification) {
        removeNotification(notification.id);
      }
    };
  });
</script>

{#if sourceConfig}
  <Separation>
    {#snippet label()}
      {#if sourceConfig}
        <div class="source-config__label">
          <Icon icon="database" />
          <!-- <StatusDot status={connected ? "success" : undefined} /> -->
          {sourceConfig.name}
        </div>
      {/if}
    {/snippet}

    {#snippet subLabel()}
      <div class="source-config__sub-label">
        {#if sourceConfig?.type === "pg"}
          PostgreSQL •
          <Badge
            style={{
              backgroundColor: connected
                ? "var(--color-light-green)"
                : "var(--color-light-grey)",
              color: "black",
            }}
          >
            {connected ? "Connected" : "Offline"}
          </Badge>
        {/if}
      </div>
    {/snippet}

    {#snippet sideSection()}
      {#if sourceConfig}
        <Dialog title="Delete Data Source" icon="cross">
          <Button
            onclick={async () => {
              if (!sourceConfig) return;
              await storeService.deleteSourceConfig(sourceConfig?.id);
              goto("/");
            }}
            icon="cross"
            style={{
              justifyContent: "space-between",
              backgroundColor: "var(--color-light-grey)",
              color: "black",
            }}>Delete</Button
          >
        </Dialog>

        <BackupSourceDialog
          {sourceConfig}
          onsubmit={async (sourceConfig) => {
            await storeService.saveSourceConfig(sourceConfig);
            loadSourceConfig();
          }}
        />
        <BackupDialog
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
  </Separation>

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
  .source-config {
    &__label {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }

    &__sub-label {
      display: flex;
      align-items: center;
      gap: 0.6rem;
    }
  }
</style>
