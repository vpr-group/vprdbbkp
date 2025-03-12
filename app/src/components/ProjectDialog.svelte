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
    project?: Project;
    onchange?: (project: Project) => void;
    oncreate?: (project: Project) => void;
  }

  const { project, onchange, oncreate }: Props = $props();

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

  let currentProject = $state<Project>(
    project || {
      id: createId(),
      name: "",
      config: { ...defaultConfigs.s3 },
    }
  );

  const updateConfig = (
    newConfig: RequiredBy<Partial<StorageConfig>, "type">
  ) => {
    currentProject = {
      ...currentProject,
      config: {
        ...currentProject.config,
        ...newConfig,
      },
    };
  };

  $effect(() => {
    onchange?.(currentProject);
  });
</script>

<Dialog label="Create project">
  <div class="project__form">
    <Input
      type="text"
      name="Name"
      value={currentProject.name}
      oninput={(e) => {
        currentProject = {
          ...currentProject,
          name: e.currentTarget.value,
        };
      }}
    />

    {#if currentProject.config.type === "S3"}
      <Input
        type="text"
        name="Endpoint"
        value={currentProject.config.endpoint}
        oninput={(e) => {
          const endpoint = e.currentTarget.value;
          updateConfig({ type: "S3", endpoint });
        }}
      />
      <Input
        type="text"
        name="Region"
        value={currentProject.config.region}
        oninput={(e) => {
          const region = e.currentTarget.value;
          updateConfig({ type: "S3", region });
        }}
      />
      <Input
        type="text"
        name="Bucket"
        value={currentProject.config.bucket}
        oninput={(e) => {
          const bucket = e.currentTarget.value;
          updateConfig({ type: "S3", bucket });
        }}
      />
      <Input
        type="text"
        name="Access Key"
        value={currentProject.config.accessKey}
        oninput={(e) => {
          const accessKey = e.currentTarget.value;
          updateConfig({ type: "S3", accessKey });
        }}
      />
      <Input
        type="text"
        name="Secret Key"
        value={currentProject.config.secretKey}
        oninput={(e) => {
          const secretKey = e.currentTarget.value;
          updateConfig({ type: "S3", secretKey });
        }}
      />
    {/if}

    <button onclick={() => oncreate?.(currentProject)}>Create</button>
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
