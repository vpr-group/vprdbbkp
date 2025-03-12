<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    href?: string;
  }

  const { children, href }: Props = $props();
</script>

{#snippet cardInner()}
  {#if children}
    {@render children()}
  {/if}
{/snippet}

{#if href}
  <a class="card" {href}>{@render cardInner()}</a>
{:else}
  <div class="card">
    {@render cardInner()}
  </div>
{/if}

<style lang="scss">
  .card {
    display: block;
    color: black;
    background-color: white;
    text-decoration: none;
    padding: 1rem;
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
  }
</style>
