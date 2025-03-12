<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children: Snippet;
  }

  const { children }: Props = $props();
</script>

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
