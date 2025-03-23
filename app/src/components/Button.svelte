<script lang="ts">
  import type { Snippet } from "svelte";
  import type { IconName } from "./Icon.svelte";
  import Icon from "./Icon.svelte";
  import { getCss, type CSSProperties } from "../utils/css";

  interface Props {
    children?: Snippet;
    href?: string;
    onclick?: () => void;
    preIcon?: IconName;
    icon?: IconName;
    style?: CSSProperties;
  }

  const { children, href, onclick, icon, preIcon, style, ...props }: Props =
    $props();
</script>

{#snippet innerButton()}
  {#if preIcon}
    <Icon icon={preIcon} />
  {/if}
  {#if children}
    {@render children()}
  {/if}
  {#if icon}
    <Icon {icon} />
  {/if}
{/snippet}

{#if href}
  <a
    class="button"
    {onclick}
    {href}
    style={style ? getCss(style || {}) : undefined}
    {...props}
  >
    {@render innerButton()}
  </a>
{:else}
  <button
    class="button"
    {onclick}
    style={style ? getCss(style || {}) : undefined}
    {...props}
  >
    {@render innerButton()}
  </button>
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
    justify-content: center;
    gap: 0.3rem;
  }
</style>
