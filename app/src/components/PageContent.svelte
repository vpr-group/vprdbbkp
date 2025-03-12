<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  const { children }: Props = $props();
</script>

<div class="page-content">
  {@render children()}
</div>

<style lang="scss">
  .page-content {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }
</style>
