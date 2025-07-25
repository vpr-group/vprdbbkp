<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type { DatabaseConfig, TunnelConfig } from "../services/store";
  import Dialog from "./Dialog.svelte";
  import Input from "./Input.svelte";
  import Button from "./Button.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Checkbox from "./Checkbox.svelte";

  interface Props {
    databaseConfig?: DatabaseConfig;
    onchange?: (databaseConfig: DatabaseConfig) => void;
    onsubmit?: (databaseConfig: DatabaseConfig) => void;
  }

  const databaseConfigTypes: DatabaseConfig["connection_type"][] = [
    "PostgreSql",
  ];

  const {
    databaseConfig: databaseConfig,
    onchange,
    onsubmit,
  }: Props = $props();

  let isConfigDialogOpen = $state(false);
  let isCreateDialogOpen = $state(false);

  let showTunnelConfig = $state(
    databaseConfig?.tunnelConfig?.useTunnel || false
  );

  const defaultTunnelConfig: TunnelConfig = {
    useTunnel: false,
    username: "",
    keyPath: "",
  };

  let currentDatabaseConfig = $state<DatabaseConfig>(
    databaseConfig || {
      id: createId(),
      name: "",
      connection_type: "PostgreSql",
      database: "",
      host: "",
      password: "",
      port: 0,
      username: "",
      tunnelConfig: defaultTunnelConfig,
    }
  );

  $effect(() => {
    onchange?.(currentDatabaseConfig);
  });
</script>

{#snippet pgDialog(databaseConfig?: DatabaseConfig)}
  <Dialog
    bind:open={isConfigDialogOpen}
    label={databaseConfig ? "" : "PostgreSQL"}
    icon={databaseConfig ? "pencil" : "arrow-right"}
    title={databaseConfig ? "Edit" : "Create"}
    buttonStyle={{
      justifyContent: "space-between",
      backgroundColor: databaseConfig ? undefined : "var(--color-light-grey)",
      color: databaseConfig ? undefined : "black",
    }}
  >
    <Input
      type="text"
      name="Name"
      value={currentDatabaseConfig.name}
      oninput={(e) => {
        currentDatabaseConfig = {
          ...currentDatabaseConfig,
          name: e.currentTarget.value,
        };
      }}
    />

    {#if currentDatabaseConfig.connection_type === "PostgreSql"}
      <Input
        name="Database"
        value={currentDatabaseConfig.database}
        oninput={(e) => {
          const database = e.currentTarget.value;
          currentDatabaseConfig = {
            ...currentDatabaseConfig,
            database,
          };
        }}
      />

      <Input
        name="Host"
        value={currentDatabaseConfig.host}
        oninput={(e) => {
          const host = e.currentTarget.value;
          currentDatabaseConfig = {
            ...currentDatabaseConfig,
            host,
          };
        }}
      />

      <Input
        name="Port"
        value={currentDatabaseConfig.port.toString()}
        oninput={(e) => {
          const port = parseInt(e.currentTarget.value);
          currentDatabaseConfig = {
            ...currentDatabaseConfig,
            port,
          };
        }}
      />

      <Input
        name="Username"
        value={currentDatabaseConfig.username}
        oninput={(e) => {
          const username = e.currentTarget.value;
          currentDatabaseConfig = {
            ...currentDatabaseConfig,
            username,
          };
        }}
      />

      <Input
        name="Password"
        value={currentDatabaseConfig.password}
        oninput={(e) => {
          const password = e.currentTarget.value;
          currentDatabaseConfig = {
            ...currentDatabaseConfig,
            password,
          };
        }}
      />

      <Checkbox
        label="Use SSH Tunnel"
        bind:checked={showTunnelConfig}
        oncheckedchange={(checked) => {
          if (!checked) {
            currentDatabaseConfig = {
              ...currentDatabaseConfig,
              tunnelConfig: defaultTunnelConfig,
            };
          }
        }}
        style={{
          padding: showTunnelConfig ? "2rem 0 1rem 0" : "2rem 0 0 0",
        }}
      />

      {#if showTunnelConfig}
        <Input
          name="Host Username"
          value={currentDatabaseConfig.tunnelConfig?.username || ""}
          oninput={(e) => {
            const username = e.currentTarget.value;
            const useTunnel = Boolean(
              currentDatabaseConfig.tunnelConfig?.keyPath && username
            );
            currentDatabaseConfig = {
              ...currentDatabaseConfig,
              tunnelConfig: {
                ...(currentDatabaseConfig.tunnelConfig || defaultTunnelConfig),
                username,
                useTunnel,
              },
            };
          }}
        />

        <Input
          name="Path to SSH Key"
          value={currentDatabaseConfig.tunnelConfig?.keyPath || ""}
          oninput={(e) => {
            const keyPath = e.currentTarget.value;
            const useTunnel = Boolean(
              currentDatabaseConfig.tunnelConfig?.username && keyPath
            );
            currentDatabaseConfig = {
              ...currentDatabaseConfig,
              tunnelConfig: {
                ...(currentDatabaseConfig.tunnelConfig || defaultTunnelConfig),
                keyPath,
                useTunnel,
              },
            };
          }}
        />
      {/if}
    {/if}

    <DialogActions>
      <Button
        icon="cross"
        onclick={() => {
          isConfigDialogOpen = false;
          isCreateDialogOpen = false;
        }}>Cancel</Button
      >

      <Button
        icon="plus"
        onclick={() => {
          onsubmit?.(currentDatabaseConfig);
          isConfigDialogOpen = false;
          isCreateDialogOpen = false;
        }}
      >
        {databaseConfig ? "Update" : "Create"}
      </Button>
    </DialogActions>
  </Dialog>
{/snippet}

{#if databaseConfig}
  {#if databaseConfig.connection_type === "PostgreSql"}
    {@render pgDialog(databaseConfig)}
  {/if}
{:else}
  <Dialog icon="plus" title="Create Data Source" bind:open={isCreateDialogOpen}>
    {@render pgDialog()}
  </Dialog>
{/if}
