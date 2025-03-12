<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type { BackupSource } from "../services/store";
  import Dialog from "./Dialog.svelte";
  import Input from "./Input.svelte";

  interface Props {
    backupSource?: BackupSource;
    onchange?: (backupSource: BackupSource) => void;
    onsubmit?: (backupSource: BackupSource) => void;
  }

  const { backupSource, onchange, onsubmit }: Props = $props();

  let currentBackupSource = $state<BackupSource>(
    backupSource || {
      id: createId(),
      name: "",
      type: "database",
      databaseType: "postgresql",
      database: "",
      host: "",
      password: "",
      port: 0,
      username: "",
    }
  );

  $effect(() => {
    onchange?.(currentBackupSource);
  });
</script>

<Dialog
  label={backupSource ? "Edit" : "Create"}
  icon={backupSource ? "pencil" : "plus"}
>
  <div class="project__form">
    <Input
      type="text"
      name="Name"
      value={currentBackupSource.name}
      oninput={(e) => {
        currentBackupSource = {
          ...currentBackupSource,
          name: e.currentTarget.value,
        };
      }}
    />

    {#if currentBackupSource.type === "database"}
      {#if currentBackupSource.databaseType === "postgresql"}
        <Input
          name="Database"
          value={currentBackupSource.database}
          oninput={(e) => {
            const database = e.currentTarget.value;
            currentBackupSource = {
              ...currentBackupSource,
              database,
            };
          }}
        />

        <Input
          name="Host"
          value={currentBackupSource.host}
          oninput={(e) => {
            const host = e.currentTarget.value;
            currentBackupSource = {
              ...currentBackupSource,
              host,
            };
          }}
        />

        <Input
          name="Port"
          value={currentBackupSource.port.toString()}
          oninput={(e) => {
            const port = parseInt(e.currentTarget.value);
            currentBackupSource = {
              ...currentBackupSource,
              port,
            };
          }}
        />

        <Input
          name="Username"
          value={currentBackupSource.username}
          oninput={(e) => {
            const username = e.currentTarget.value;
            currentBackupSource = {
              ...currentBackupSource,
              username,
            };
          }}
        />

        <Input
          name="Password"
          value={currentBackupSource.password}
          oninput={(e) => {
            const password = e.currentTarget.value;
            currentBackupSource = {
              ...currentBackupSource,
              password,
            };
          }}
        />
      {/if}
    {/if}

    <button onclick={() => onsubmit?.(currentBackupSource)}>
      {backupSource ? "Update" : "Create"}
    </button>
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
