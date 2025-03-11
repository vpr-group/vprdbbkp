<script lang="ts">
  import { StoreService, type Project } from "../services/dataStore";
  import { onMount } from "svelte";

  const storeService = new StoreService();

  let isLoading = $state(true);
  let projects = $state<Project[]>([]);

  onMount(async () => {
    await storeService.waitForInitialized();
    storeService.getProjects().then((res) => {
      projects = res;
      isLoading = false;
    });
  });
</script>

<main class="container">
  {#if isLoading}
    <span>Is Loding</span>
  {:else if projects.length === 0}
    <button>Create new project</button>
  {/if}
</main>

<style>
</style>
