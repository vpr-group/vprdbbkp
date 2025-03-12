<script lang="ts">
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { StoreService, type BackupSource } from "../../../services/store";
  import Separation from "../../../components/Separation.svelte";
  import BackupSourceDialog from "../../../components/BackupSourceDialog.svelte";
  import Button from "../../../components/Button.svelte";
  import Table from "../../../components/Table.svelte";
  import { goto } from "$app/navigation";
  import type { CSSProperties } from "../../../utils/css";
  import { ActionsService } from "../../../services/actions";
  import StatusDot from "../../../components/StatusDot.svelte";

  const actionService = new ActionsService();
  const storeService = new StoreService();

  let backupSource = $state<BackupSource | null>(null);
  let connected = $state(false);

  const loadBackupSource = async () => {
    await storeService.waitForInitialized();
    backupSource = await storeService.getBackupSource(page.params.id);
  };

  const cellLabelStyle: CSSProperties = {
    color: "var(--color-grey)",
  };

  onMount(async () => {
    loadBackupSource();
  });

  $effect(() => {
    if (!backupSource) return;
    actionService.verifyBackupSourceConnection(backupSource).then((res) => {
      connected = res.connected;
    });
  });
</script>

{#snippet sideSection()}
  {#if backupSource}
    <BackupSourceDialog
      {backupSource}
      onsubmit={async (backupSource) => {
        await storeService.saveBackupSource(backupSource);
        loadBackupSource();
      }}
    />
    <Button
      onclick={async () => {
        if (!backupSource) return;
        await storeService.deleteBackupSource(backupSource?.id);
        goto("/");
      }}
      icon="cross">Delete</Button
    >
  {/if}
{/snippet}

{#snippet label()}
  {#if backupSource}
    <div class="backup-source__label">
      <StatusDot status={connected ? "success" : undefined} />
      {backupSource.name}
    </div>
  {/if}
{/snippet}

{#if backupSource}
  <Separation {label} subLabel={backupSource.type} {sideSection} />

  <Table
    rows={[
      {
        cells: [
          { label: "database type", width: "10rem", style: cellLabelStyle },
          { label: backupSource.databaseType || "-" },
        ],
      },
      {
        cells: [
          { label: "database", width: "10rem", style: cellLabelStyle },
          { label: backupSource.database || "-" },
        ],
      },
      {
        cells: [
          { label: "host", width: "10rem", style: cellLabelStyle },
          { label: backupSource.host || "-" },
        ],
      },
      {
        cells: [
          { label: "port", width: "10rem", style: cellLabelStyle },
          { label: backupSource.port.toString() || "-" },
        ],
      },
      {
        cells: [
          { label: "username", width: "10rem", style: cellLabelStyle },
          { label: backupSource.username || "-" },
        ],
      },
      {
        cells: [
          { label: "password", width: "10rem", style: cellLabelStyle },
          {
            label:
              (backupSource.password || "")
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
