<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type { SourceConfig } from "../services/store";
  import Dialog from "./Dialog.svelte";
  import Input from "./Input.svelte";
  import Separation from "./Separation.svelte";
  import Button from "./Button.svelte";
  import DialogActions from "./DialogActions.svelte";
  import DropdownMenu from "./DropdownMenu.svelte";

  interface Props {
    sourceConfig?: SourceConfig;
    onchange?: (sourceConfig: SourceConfig) => void;
    onsubmit?: (sourceConfig: SourceConfig) => void;
  }

  const sourceConfigTypes: SourceConfig["type"][] = ["pg"];
  const { sourceConfig, onchange, onsubmit }: Props = $props();
  let open = $state(false);

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
    }
  );

  $effect(() => {
    onchange?.(currentSourceConfig);
  });
</script>

{#snippet pgDialog(sourceConfig?: SourceConfig)}
  <Dialog
    bind:open
    label={sourceConfig ? "Edit" : "PostgreSQL Source"}
    icon={sourceConfig ? "pencil" : undefined}
    buttonStyle={sourceConfig
      ? undefined
      : {
          backgroundColor: "var(--color-light-grey)",
          color: "black",
        }}
  >
    <div class="project">
      <Separation
        label={sourceConfig
          ? "Edit PostgreSQL Source"
          : "Create PostgreSQL Source"}
      />

      <div class="project__form">
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
        {/if}

        <DialogActions>
          <Button icon="cross" onclick={() => (open = false)}>Cancel</Button>
          <Button icon="plus" onclick={() => onsubmit?.(currentSourceConfig)}>
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
  <DropdownMenu
    items={sourceConfigTypes}
    label="Create"
    icon="plus"
    align="end"
  >
    {#snippet item(type)}
      {#if type === "pg"}
        {@render pgDialog()}
      {/if}
    {/snippet}
  </DropdownMenu>
{/if}

<style lang="scss">
  .project {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;

    &__form {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      min-width: 30rem;
    }
  }
</style>
