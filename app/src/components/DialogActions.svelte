<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
  }

  const { children }: Props = $props();
</script>

<div class="dialog-actions">
  {#if children}
    {@render children()}
  {/if}
</div>

<style lang="scss">
  .dialog-actions {
    margin-top: 2rem;
    display: flex;
    gap: 4px;
  }
</style>
