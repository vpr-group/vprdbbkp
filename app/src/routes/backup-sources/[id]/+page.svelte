<script lang="ts">
  import { page } from "$app/state";
  import { onMount } from "svelte";
  import { StoreService, type SourceConfig } from "../../../services/store";
  import Separation from "../../../components/Separation.svelte";
  import BackupSourceDialog from "../../../components/BackupSourceDialog.svelte";
  import Button from "../../../components/Button.svelte";
  import Table from "../../../components/Table.svelte";
  import { goto } from "$app/navigation";
  import type { CSSProperties } from "../../../utils/css";
  import { ActionsService } from "../../../services/actions";
  import StatusDot from "../../../components/StatusDot.svelte";
  import BackupDropdown from "../../../components/BackupDropdown.svelte";

  const actionService = new ActionsService();
  const storeService = new StoreService();

  let backupSource = $state<SourceConfig | null>(null);
  let connected = $state(false);

  const loadBackupSource = async () => {
    await storeService.waitForInitialized();
    backupSource = await storeService.getSourceConfig(page.params.id);
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
    <Button
      onclick={async () => {
        if (!backupSource) return;
        await storeService.deleteSourceConfig(backupSource?.id);
        goto("/");
      }}
      icon="cross">Delete</Button
    >
    <BackupSourceDialog
      {backupSource}
      onsubmit={async (backupSource) => {
        await storeService.saveSourceConfig(backupSource);
        loadBackupSource();
      }}
    />
    <BackupDropdown
      onbackup={(storageProvider) => {
        if (!backupSource) return;
        actionService.backupSource(backupSource, storageProvider);
      }}
    />
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
