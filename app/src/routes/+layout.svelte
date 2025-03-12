<script lang="ts">
  import type { Snippet } from "svelte";
  import PageContent from "../components/PageContent.svelte";
  import Button from "../components/Button.svelte";
  import { page } from "$app/state";

  interface Props {
    children: Snippet;
  }

  const { children }: Props = $props();
</script>

<div class="layout">
  {#if page.url.pathname !== "/"}
    <Button href="/">Back</Button>
  {/if}

  <PageContent>
    {@render children()}
  </PageContent>
</div>

<style lang="scss">
  @font-face {
    font-family: Rubik;
    src: url(/fonts/Rubik-Medium.ttf);
  }

  @font-face {
    font-family: TeX Gyre Heros;
    src: url(/fonts/texgyreheros-regular.otf);
    font-weight: 400;
  }

  @font-face {
    font-family: TeX Gyre Heros;
    src: url(/fonts/texgyreheros-bold.otf);
    font-weight: 600;
  }

  @font-face {
    font-family: Liberation Mono;
    src: url(/fonts/LiberationMono-Regular.ttf);
    font-weight: 400;
  }

  @font-face {
    font-family: Liberation Mono;
    src: url(/fonts/LiberationMono-Bold.ttf);
    font-weight: 600;
  }

  :root {
    --main-font-family: TeX Gyre Heros, Arial, Helvetica, sans-serif;
    --mono-font-family: Liberation Mono, monospace;

    --color-light: hsl(0, 0%, calc(94% + 0%));
    --color-light-grey: hsl(0, 0%, calc(80% + 0%));
    --color-grey: hsl(0, 0%, calc(50% + 0%));
    --shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.05), 0 1px 2px 0 rgba(0, 0, 0, 0.025);
    --border-radius: 0.3rem;
  }

  :global(*) {
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    text-rendering: optimizeLegibility;
    box-sizing: border-box;
  }

  :global(html, body) {
    font-family: var(--main-font-family);
    background-color: var(--color-light);
    margin: 0;
    font-size: 14px;
  }

  :global(body) {
    padding: 2rem;
  }

  .layout {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    width: 100%;
    gap: 1rem;
  }
</style>
