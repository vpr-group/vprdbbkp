<script lang="ts">
  import { page } from "$app/state";
  import ProjectDialog from "../../../components/ProjectDialog.svelte";
  import WorkspaceDialog from "../../../components/WorkspaceDialog.svelte";
  import { StoreService, type Project } from "../../../services/dataStore";
  import { onMount } from "svelte";

  const storeService = new StoreService();
  let project = $state<Project | null>(null);

  const loadProject = async () => {
    await storeService.waitForInitialized();
    project = await storeService.getProject(page.params.id);
  };

  onMount(() => {
    loadProject();
  });
</script>

{#if project}
  <h1>{project.name}</h1>
  <ProjectDialog
    {project}
    oncreate={async (project) => {
      await storeService.saveProject(project);
      loadProject();
    }}
  />
  <WorkspaceDialog />
{/if}
