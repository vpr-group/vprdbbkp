<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type {
    PostgresConfig,
    Workspace,
    WorkspaceConfig,
  } from "../services/dataStore";
  import Dialog from "./Dialog.svelte";
  import type { RequiredBy } from "../utils/types";
  import Input from "./Input.svelte";

  interface Props {
    workspace?: Workspace;
    onchange?: (workspace: Workspace) => void;
    oncreate?: (workspace: Workspace) => void;
  }

  const { workspace, onchange, oncreate }: Props = $props();

  const defaultPostgresConfig: PostgresConfig = {
    type: "postgres",
    database: "",
    host: "",
    password: "",
    port: 0,
    username: "",
  };

  const defaultConfigs = {
    postgres: defaultPostgresConfig,
  };

  let currentWorkspace = $state<Workspace>(
    workspace || {
      id: createId(),
      name: "",
      projectId: "",
      config: { ...defaultConfigs.postgres },
    }
  );

  const updateConfig = (
    newConfig: RequiredBy<Partial<WorkspaceConfig>, "type">
  ) => {
    currentWorkspace = {
      ...currentWorkspace,
      config: {
        ...currentWorkspace.config,
        ...newConfig,
      },
    };
  };

  $effect(() => {
    onchange?.(currentWorkspace);
  });
</script>

<Dialog label="Create workspace">
  <div class="project__form">
    <Input
      type="text"
      name="Name"
      value={currentWorkspace.name}
      oninput={(e) => {
        currentWorkspace = {
          ...currentWorkspace,
          name: e.currentTarget.value,
        };
      }}
    />

    {#if currentWorkspace.config.type === "postgres"}
      <Input
        name="Database"
        value={currentWorkspace.config.database}
        oninput={(e) =>
          updateConfig({ type: "postgres", database: e.currentTarget.value })}
      />
    {/if}

    <button onclick={() => oncreate?.(currentWorkspace)}>Create</button>
  </div>
</Dialog>

<style lang="scss">
  .project {
    &__form {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      min-width: 30rem;
    }
  }
</style>
