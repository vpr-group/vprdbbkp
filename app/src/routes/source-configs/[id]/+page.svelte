<script lang="ts">
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { StoreService, type DatabaseConfig } from "../../../services/store";
  import Separation from "../../../components/Separation.svelte";
  import BackupSourceDialog from "../../../components/DatabaseConfigDialog.svelte";
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

  let databaseConfig = $state<DatabaseConfig | null>(null);
  let connected = $state(false);

  const loaddatabaseConfig = async () => {
    await storeService.waitForInitialized();
    databaseConfig = await storeService.getDatabaseConfig(page.params.id);
  };

  const cellLabelStyle: CSSProperties = {
    color: "var(--color-grey)",
  };

  onMount(async () => {
    loaddatabaseConfig();
  });

  $effect(() => {
    if (!databaseConfig) return;
    let notification: Notification | undefined = undefined;

    actionService
      .testConnection(databaseConfig)
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

{#if databaseConfig}
  <Separation>
    {#snippet label()}
      {#if databaseConfig}
        <div class="database-config__label">
          <Icon icon="database" />
          <!-- <StatusDot status={connected ? "success" : undefined} /> -->
          {databaseConfig.name}
        </div>
      {/if}
    {/snippet}

    {#snippet subLabel()}
      <div class="database-config__sub-label">
        {#if databaseConfig?.connection_type === "PostgreSql"}
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
      {#if databaseConfig}
        <Dialog title="Delete Data Source" icon="cross">
          <Button
            onclick={async () => {
              if (!databaseConfig) return;
              await storeService.deleteDatabaseConfig(databaseConfig?.id);
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
          {databaseConfig}
          onsubmit={async (databaseConfig) => {
            await storeService.saveDatabaseConfig(databaseConfig);
            loaddatabaseConfig();
          }}
        />
        <BackupDialog
          onbackup={async (storageConfig) => {
            if (!databaseConfig) return;

            const backupInProgressNotification = addNotification({
              title: "Backup in progress...",
              status: "info",
              dismissTimeout: null,
            });

            try {
              await actionService.backup(databaseConfig, storageConfig);
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
          { label: databaseConfig.connection_type || "-" },
        ],
      },
      {
        cells: [
          { label: "database", width: "10rem", style: cellLabelStyle },
          { label: databaseConfig.database || "-" },
        ],
      },
      {
        cells: [
          { label: "host", width: "10rem", style: cellLabelStyle },
          { label: databaseConfig.host || "-" },
        ],
      },
      {
        cells: [
          { label: "port", width: "10rem", style: cellLabelStyle },
          { label: databaseConfig.port.toString() || "-" },
        ],
      },
      {
        cells: [
          { label: "username", width: "10rem", style: cellLabelStyle },
          { label: databaseConfig.username || "-" },
        ],
      },
      {
        cells: [
          { label: "password", width: "10rem", style: cellLabelStyle },
          {
            label:
              (databaseConfig.password || "")
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
  .database-config {
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
