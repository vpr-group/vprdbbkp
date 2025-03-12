<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    href?: string;
    onclick?: () => void;
  }

  const { children, href, onclick }: Props = $props();
</script>

{#snippet innerButton()}
  {#if children}
    {@render children()}
  {/if}
{/snippet}

{#if href}
  <a class="button" {onclick} {href}>{@render innerButton()}</a>
{:else}
  <button class="button" {onclick}>{@render innerButton()}</button>
{/if}

<style lang="scss">
  .button {
    background-color: black;
    color: white;
    border: none;
    padding: 0.4rem 0.7rem;
    text-decoration: none;
    box-shadow: var(--shadow);
    border-radius: 0.2rem;
    cursor: pointer;
  }
</style>
