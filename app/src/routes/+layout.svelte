<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  const { children }: Props = $props();
</script>

<a href="/">Home</a>
{@render children()}

<style lang="scss">
  @font-face {
    font-family: Rubik;
    src: url(/fonts/Rubik-Medium.ttf);
  }

  :global(html, body) {
    font-family: Rubik, Arial, Helvetica, sans-serif;
  }
</style>
