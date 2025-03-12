<script lang="ts">
  import type { Snippet } from "svelte";
  import type { IconName } from "./Icon.svelte";
  import Icon from "./Icon.svelte";

  interface Props {
    children?: Snippet;
    href?: string;
    onclick?: () => void;
    preIcon?: IconName;
    icon?: IconName;
  }

  const { children, href, onclick, icon, preIcon }: Props = $props();
</script>

{#snippet innerButton()}
  {#if children}
    {#if preIcon}
      <Icon icon={preIcon} />
    {/if}
    {@render children()}
    {#if icon}
      <Icon {icon} />
    {/if}
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
    padding: 0.3rem 0.7rem;
    text-decoration: none;
    box-shadow: var(--shadow);
    border-radius: 0.2rem;
    cursor: pointer;
    font-family: var(--mono-font-family);
    font-size: inherit;
    display: flex;
    align-items: center;
    gap: 0.3rem;
  }
</style>
