<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type { StorageConfig } from "../services/store";
  import Dialog from "./Dialog.svelte";
  import Input from "./Input.svelte";
  import Button from "./Button.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Separation from "./Separation.svelte";

  interface Props {
    storageConfig?: StorageConfig;
    onchange?: (storageConfig: StorageConfig) => void;
    onsubmit?: (storageConfig: StorageConfig) => void;
  }

  const { storageConfig, onchange, onsubmit }: Props = $props();
  let isConfigDialogOpen = $state(false);
  let isCreateDialogOpen = $state(false);

  let currentStorageConfig = $state<StorageConfig>(
    storageConfig || {
      type: "s3",
      id: createId(),
      name: "",
      accessKey: "",
      bucket: "",
      endpoint: "",
      region: "",
      secretKey: "",
    },
  );

  const updateStorageConfigData = (data: Partial<StorageConfig>) => {
    currentStorageConfig = {
      ...currentStorageConfig,
      ...data,
    } as StorageConfig;
  };

  $effect(() => {
    onchange?.(currentStorageConfig);
  });
</script>

{#snippet s3Dialog(storageConfig?: StorageConfig)}
  <Dialog
    bind:open={isConfigDialogOpen}
    label={storageConfig ? "" : "S3"}
    icon={storageConfig ? "pencil" : "arrow-right"}
    title={storageConfig ? "Edit " : "Create "}
    buttonStyle={{
      justifyContent: "space-between",
      backgroundColor: storageConfig ? undefined : "var(--color-light-grey)",
      color: storageConfig ? undefined : "black",
    }}
  >
    <Input
      type="text"
      name="Name"
      value={currentStorageConfig.name}
      oninput={(e) => {
        updateStorageConfigData({
          name: e.currentTarget.value,
        });
      }}
    />

    {#if currentStorageConfig.type === "s3"}
      <Input
        type="text"
        name="Endpoint"
        value={currentStorageConfig.endpoint}
        oninput={(e) => {
          updateStorageConfigData({
            endpoint: e.currentTarget.value,
          });
        }}
      />
      <Input
        type="text"
        name="Region"
        value={currentStorageConfig.region}
        oninput={(e) => {
          updateStorageConfigData({
            region: e.currentTarget.value,
          });
        }}
      />
      <Input
        type="text"
        name="Bucket"
        value={currentStorageConfig.bucket}
        oninput={(e) => {
          updateStorageConfigData({
            bucket: e.currentTarget.value,
          });
        }}
      />
      <Input
        type="text"
        name="Access Key"
        value={currentStorageConfig.accessKey}
        oninput={(e) => {
          updateStorageConfigData({
            accessKey: e.currentTarget.value,
          });
        }}
      />
      <Input
        type="text"
        name="Secret Key"
        value={currentStorageConfig.secretKey}
        oninput={(e) => {
          updateStorageConfigData({
            secretKey: e.currentTarget.value,
          });
        }}
      />
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
          onsubmit?.(currentStorageConfig);
          isConfigDialogOpen = false;
          isCreateDialogOpen = false;
        }}
      >
        {storageConfig ? "Update" : "Create"}
      </Button>
    </DialogActions>
  </Dialog>
{/snippet}

{#if storageConfig}
  {@render s3Dialog(storageConfig)}
{:else}
  <Dialog
    icon="plus"
    bind:open={isCreateDialogOpen}
    title="Create File Storage"
  >
    {@render s3Dialog()}
  </Dialog>
{/if}
