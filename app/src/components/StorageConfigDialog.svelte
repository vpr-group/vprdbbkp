<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type { StorageConfig } from "../services/store";
  import Dialog from "./Dialog.svelte";
  import Input from "./Input.svelte";
  import Button from "./Button.svelte";
  import DialogActions from "./DialogActions.svelte";
  import Separation from "./Separation.svelte";
  import DropdownMenu from "./DropdownMenu.svelte";

  interface Props {
    storageConfig?: StorageConfig;
    onchange?: (storageConfig: StorageConfig) => void;
    onsubmit?: (storageConfig: StorageConfig) => void;
  }

  const { storageConfig, onchange, onsubmit }: Props = $props();
  let open = $state(false);
  const storageConfigTypes: StorageConfig["type"][] = ["s3", "local"];

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
    }
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
    bind:open
    label={storageConfig ? "Edit" : "S3 Storage"}
    icon={storageConfig ? "pencil" : undefined}
    buttonStyle={storageConfig
      ? undefined
      : {
          backgroundColor: "var(--color-light-grey)",
          color: "black",
        }}
  >
    <div class="project">
      <Separation
        label={storageConfig
          ? "Edit S3 Storage Config"
          : "Create S3 Storage Config"}
      />
      <div class="project__form">
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
          <Button icon="cross" onclick={() => (open = false)}>Cancel</Button>
          <Button
            icon="plus"
            onclick={() => {
              onsubmit?.(currentStorageConfig);
              open = false;
            }}
          >
            {storageConfig ? "Update" : "Create"}
          </Button>
        </DialogActions>
      </div>
    </div>
  </Dialog>
{/snippet}

{#if storageConfig}
  {@render s3Dialog(storageConfig)}
{:else}
  <DropdownMenu
    label="Create"
    icon="plus"
    items={storageConfigTypes}
    align="end"
  >
    {#snippet item(item)}
      {#if item === "s3"}
        {@render s3Dialog()}
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
