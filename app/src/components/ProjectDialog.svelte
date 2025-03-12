<script lang="ts">
  import { createId } from "@paralleldrive/cuid2";
  import type {
    Project,
    S3StorageConfig,
    StorageConfig,
  } from "../services/dataStore";
  import Dialog from "./Dialog.svelte";
  import type { RequiredBy } from "../utils/types";
  import Input from "./Input.svelte";

  interface Props {
    onchange?: (project: Project) => void;
    oncreate?: (project: Project) => void;
  }

  const { onchange, oncreate }: Props = $props();

  const defaultS3Config: S3StorageConfig = {
    type: "S3",
    endpoint: "",
    region: "",
    bucket: "",
    accessKey: "",
    secretKey: "",
  };

  const defaultConfigs = {
    s3: defaultS3Config,
  };

  let project = $state<Project>({
    id: createId(),
    name: "",
    config: { ...defaultConfigs.s3 },
  });

  const updateConfig = (
    newConfig: RequiredBy<Partial<StorageConfig>, "type">
  ) => {
    project = {
      ...project,
      config: {
        ...project.config,
        ...newConfig,
      },
    };
  };

  $effect(() => {
    onchange?.(project);
  });
</script>

<Dialog label="Create project">
  <div class="project__form">
    <Input
      type="text"
      name="Name"
      value={project.name}
      oninput={(e) => {
        project = {
          ...project,
          name: e.currentTarget.value,
        };
      }}
    />

    {#if project.config.type === "S3"}
      <Input
        type="text"
        name="Endpoint"
        value={project.config.endpoint}
        oninput={(e) => {
          const endpoint = e.currentTarget.value;
          updateConfig({ type: "S3", endpoint });
        }}
      />
      <Input
        type="text"
        name="Region"
        value={project.config.region}
        oninput={(e) => {
          const region = e.currentTarget.value;
          updateConfig({ type: "S3", region });
        }}
      />
      <Input
        type="text"
        name="Bucket"
        value={project.config.bucket}
        oninput={(e) => {
          const bucket = e.currentTarget.value;
          updateConfig({ type: "S3", bucket });
        }}
      />
      <Input
        type="text"
        name="Access Key"
        value={project.config.accessKey}
        oninput={(e) => {
          const accessKey = e.currentTarget.value;
          updateConfig({ type: "S3", accessKey });
        }}
      />
      <Input
        type="text"
        name="Secret Key"
        value={project.config.secretKey}
        oninput={(e) => {
          const secretKey = e.currentTarget.value;
          updateConfig({ type: "S3", secretKey });
        }}
      />
    {/if}

    <button onclick={() => oncreate?.(project)}>Create</button>
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
