<script lang="ts">
  import ProjectDialog from "../components/ProjectDialog.svelte";
  import { StoreService, type Project } from "../services/dataStore";
  import { onMount } from "svelte";

  const storeService = new StoreService();

  let isLoading = $state(true);
  let projects = $state<Project[]>([]);

  const loadProjects = async () => {
    await storeService.waitForInitialized();
    storeService.getProjects().then((res) => {
      projects = res;
      isLoading = false;
    });
  };

  onMount(() => {
    loadProjects();
  });
</script>

<main class="container">
  {#if isLoading}
    <span>Is Loding</span>
  {:else if projects.length === 0}
    <ProjectDialog
      oncreate={(project) => {
        console.log(project);
        storeService.saveProject(project);
        loadProjects();
      }}
    />
  {:else}
    <ProjectDialog
      oncreate={async (project) => {
        console.log(project);
        await storeService.saveProject(project);
        loadProjects();
      }}
    />
    {#each projects as project}
      <span>{project.name}</span>
    {/each}
  {/if}
</main>

<style>
</style>
