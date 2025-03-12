<script lang="ts">
  import type { Snippet } from "svelte";
  import { getCss } from "../utils/css";

  interface Props {
    children?: Snippet;
    minmax?: string;
    style?: Partial<CSSStyleDeclaration>;
  }

  const { children, minmax = "33%", style }: Props = $props();

  const computedStyle = $derived(
    getCss({
      gridTemplateColumns: minmax
        ? `repeat(auto-fill, minmax(${minmax}, 1fr))`
        : undefined,
      ...style,
    })
  );
</script>

<div class="grid" style={computedStyle}>
  {#if children}
    {@render children()}
  {/if}
</div>

<style lang="scss">
  .grid {
    display: grid;
    gap: 0.5rem;
  }
</style>
