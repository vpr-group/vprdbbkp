<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type { SourceConfig, TunnelConfig } from "../services/store";
  import Dialog from "./Dialog.svelte";
  import Input from "./Input.svelte";
  import Separation from "./Separation.svelte";
  import Button from "./Button.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Checkbox from "./Checkbox.svelte";

  interface Props {
    sourceConfig?: SourceConfig;
    onchange?: (sourceConfig: SourceConfig) => void;
    onsubmit?: (sourceConfig: SourceConfig) => void;
  }

  const sourceConfigTypes: SourceConfig["type"][] = ["pg"];
  const { sourceConfig, onchange, onsubmit }: Props = $props();
  let isConfigDialogOpen = $state(false);
  let isCreateDialogOpen = $state(false);

  let showTunnelConfig = $state(sourceConfig?.tunnelConfig?.useTunnel || false);

  const defaultTunnelConfig: TunnelConfig = {
    useTunnel: false,
    username: "",
    keyPath: "",
  };

  let currentSourceConfig = $state<SourceConfig>(
    sourceConfig || {
      id: createId(),
      name: "",
      type: "pg",
      database: "",
      host: "",
      password: "",
      port: 0,
      username: "",
      tunnelConfig: defaultTunnelConfig,
    }
  );

  $effect(() => {
    onchange?.(currentSourceConfig);
  });
</script>

{#snippet pgDialog(sourceConfig?: SourceConfig)}
  <Dialog
    bind:open={isConfigDialogOpen}
    label={sourceConfig ? "" : "PostgreSQL"}
    icon={sourceConfig ? "pencil" : "arrow-right"}
    buttonStyle={{
      justifyContent: "space-between",
      backgroundColor: sourceConfig ? undefined : "var(--color-light-grey)",
      color: sourceConfig ? undefined : "black",
    }}
  >
    <div class="source-config-dialog">
      <Separation label={sourceConfig ? "Edit" : "Create"} />

      <div class="source-config-dialog__form">
        <Input
          type="text"
          name="Name"
          value={currentSourceConfig.name}
          oninput={(e) => {
            currentSourceConfig = {
              ...currentSourceConfig,
              name: e.currentTarget.value,
            };
          }}
        />

        {#if currentSourceConfig.type === "pg"}
          <Input
            name="Database"
            value={currentSourceConfig.database}
            oninput={(e) => {
              const database = e.currentTarget.value;
              currentSourceConfig = {
                ...currentSourceConfig,
                database,
              };
            }}
          />

          <Input
            name="Host"
            value={currentSourceConfig.host}
            oninput={(e) => {
              const host = e.currentTarget.value;
              currentSourceConfig = {
                ...currentSourceConfig,
                host,
              };
            }}
          />

          <Input
            name="Port"
            value={currentSourceConfig.port.toString()}
            oninput={(e) => {
              const port = parseInt(e.currentTarget.value);
              currentSourceConfig = {
                ...currentSourceConfig,
                port,
              };
            }}
          />

          <Input
            name="Username"
            value={currentSourceConfig.username}
            oninput={(e) => {
              const username = e.currentTarget.value;
              currentSourceConfig = {
                ...currentSourceConfig,
                username,
              };
            }}
          />

          <Input
            name="Password"
            value={currentSourceConfig.password}
            oninput={(e) => {
              const password = e.currentTarget.value;
              currentSourceConfig = {
                ...currentSourceConfig,
                password,
              };
            }}
          />

          <Checkbox
            label="Use SSH Tunnel"
            bind:checked={showTunnelConfig}
            oncheckedchange={(checked) => {
              if (!checked) {
                currentSourceConfig = {
                  ...currentSourceConfig,
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
              value={currentSourceConfig.tunnelConfig?.username || ""}
              oninput={(e) => {
                const username = e.currentTarget.value;
                const useTunnel = Boolean(
                  currentSourceConfig.tunnelConfig?.keyPath && username
                );
                currentSourceConfig = {
                  ...currentSourceConfig,
                  tunnelConfig: {
                    ...(currentSourceConfig.tunnelConfig ||
                      defaultTunnelConfig),
                    username,
                    useTunnel,
                  },
                };
              }}
            />

            <Input
              name="Path to SSH Key"
              value={currentSourceConfig.tunnelConfig?.keyPath || ""}
              oninput={(e) => {
                const keyPath = e.currentTarget.value;
                const useTunnel = Boolean(
                  currentSourceConfig.tunnelConfig?.username && keyPath
                );
                currentSourceConfig = {
                  ...currentSourceConfig,
                  tunnelConfig: {
                    ...(currentSourceConfig.tunnelConfig ||
                      defaultTunnelConfig),
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
              onsubmit?.(currentSourceConfig);
              isConfigDialogOpen = false;
              isCreateDialogOpen = false;
            }}
          >
            {sourceConfig ? "Update" : "Create"}
          </Button>
        </DialogActions>
      </div>
    </div>
  </Dialog>
{/snippet}

{#if sourceConfig}
  {#if sourceConfig.type === "pg"}
    {@render pgDialog(sourceConfig)}
  {/if}
{:else}
  <Dialog icon="plus" bind:open={isCreateDialogOpen}>
    <Separation label="Create Data Source" />
    <div class="source-config-dialog__data-sources">
      {@render pgDialog()}
    </div>
  </Dialog>
{/if}

<style lang="scss">
  .source-config-dialog {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;

    &__form {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      min-width: 30rem;
    }

    &__data-sources {
      display: flex;
      flex-direction: column;
      padding: 1rem 0;
    }
  }
</style>
