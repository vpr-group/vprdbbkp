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

  const actionService = new ActionsService();
  const storeService = new StoreService();

  let sourceConfig = $state<SourceConfig | null>(null);
  let connected = $state(false);

  const loadBackupSource = async () => {
    await storeService.waitForInitialized();
    sourceConfig = await storeService.getSourceConfig(page.params.id);
  };

  const cellLabelStyle: CSSProperties = {
    color: "var(--color-grey)",
  };

  onMount(async () => {
    loadBackupSource();
  });

  $effect(() => {
    if (!sourceConfig) return;
    actionService.verifySourceonnection(sourceConfig).then((res) => {
      connected = res.connected;
    });
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
      onsubmit={async (backupSource) => {
        await storeService.saveSourceConfig(backupSource);
        loadBackupSource();
      }}
    />
    <BackupDropdown
      onbackup={(storageProvider) => {
        if (!sourceConfig) return;
        actionService.backupSource(sourceConfig, storageProvider);
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
