<script lang="ts">
  import type { Snippet } from "svelte";
  import PageContent from "../components/PageContent.svelte";
  import Button from "../components/Button.svelte";
  import { page } from "$app/state";
  import Notifications from "../components/Notifications.svelte";
  import Sidebar from "../components/Sidebar.svelte";
  import Dialogs from "../components/Dialogs.svelte";

  interface Props {
    children: Snippet;
  }

  const { children }: Props = $props();
</script>

<div class="layout">
  <Sidebar />

  <PageContent>
    {#if page.url.pathname !== "/"}
      <div class="layout__header">
        <Button href="/" preIcon="arrow-left" />
      </div>
    {/if}

    {@render children()}
  </PageContent>

  <Notifications />
  <Dialogs />
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

    --color-light: hsl(0, 0%, calc(95% + 0%));
    --color-light-grey: hsl(0, 0%, calc(80% + 0%));
    --color-grey: hsl(0, 0%, calc(50% + 0%));
    --color-green: rgb(109, 248, 109);
    --color-light-green: rgba(109, 248, 109, 0.4);
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
    /* padding: 1.5rem; */
  }

  .layout {
    display: flex;
    flex-direction: row;
    align-items: flex-start;
    width: 100%;
    /* gap: 1rem; */

    &__header {
      padding-bottom: 1rem;
      border-bottom: solid 1px var(--color-light-grey);
      display: flex;
    }
  }
</style>
