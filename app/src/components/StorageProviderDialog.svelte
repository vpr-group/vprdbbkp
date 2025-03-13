<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type { StorageProvider } from "../services/store";
  import Dialog from "./Dialog.svelte";
  import Input from "./Input.svelte";
  import Button from "./Button.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Separation from "./Separation.svelte";

  interface Props {
    storageProvider?: StorageProvider;
    onchange?: (storageProvider: StorageProvider) => void;
    onsubmit?: (storageProvider: StorageProvider) => void;
  }

  const { storageProvider, onchange, onsubmit }: Props = $props();
  let open = $state(false);

  let currentStorageProvider = $state<StorageProvider>(
    storageProvider || {
      id: createId(),
      name: "",
      type: "s3",
      accessKey: "",
      bucket: "",
      endpoint: "",
      region: "",
      secretKey: "",
    }
  );

  $effect(() => {
    onchange?.(currentStorageProvider);
  });
</script>

<Dialog
  bind:open
  label={storageProvider ? "Edit" : "Create"}
  icon={storageProvider ? "pencil" : "plus"}
>
  <div class="project">
    <Separation
      label={storageProvider
        ? "Edit Storage Provider"
        : "Create Storage Provider"}
    />
    <div class="project__form">
      <Input
        type="text"
        name="Name"
        value={currentStorageProvider.name}
        oninput={(e) => {
          currentStorageProvider = {
            ...currentStorageProvider,
            name: e.currentTarget.value,
          };
        }}
      />

      {#if currentStorageProvider.type === "s3"}
        <Input
          type="text"
          name="Endpoint"
          value={currentStorageProvider.endpoint}
          oninput={(e) => {
            const endpoint = e.currentTarget.value;
            currentStorageProvider = {
              ...currentStorageProvider,
              endpoint,
            };
          }}
        />
        <Input
          type="text"
          name="Region"
          value={currentStorageProvider.region}
          oninput={(e) => {
            const region = e.currentTarget.value;
            currentStorageProvider = {
              ...currentStorageProvider,
              region,
            };
          }}
        />
        <Input
          type="text"
          name="Bucket"
          value={currentStorageProvider.bucket}
          oninput={(e) => {
            const bucket = e.currentTarget.value;
            currentStorageProvider = {
              ...currentStorageProvider,
              bucket,
            };
          }}
        />
        <Input
          type="text"
          name="Access Key"
          value={currentStorageProvider.accessKey}
          oninput={(e) => {
            const accessKey = e.currentTarget.value;
            currentStorageProvider = {
              ...currentStorageProvider,
              accessKey,
            };
          }}
        />
        <Input
          type="text"
          name="Secret Key"
          value={currentStorageProvider.secretKey}
          oninput={(e) => {
            const secretKey = e.currentTarget.value;
            currentStorageProvider = {
              ...currentStorageProvider,
              secretKey,
            };
          }}
        />
      {/if}

      <DialogActions>
        <Button icon="cross" onclick={() => (open = false)}>Cancel</Button>
        <Button icon="plus" onclick={() => onsubmit?.(currentStorageProvider)}>
          {storageProvider ? "Update" : "Create"}
        </Button>
      </DialogActions>
    </div>
  </div>
</Dialog>

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
