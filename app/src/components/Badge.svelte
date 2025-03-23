<script lang="ts">
  import type { Snippet } from "svelte";
  import { getCss, type CSSProperties } from "../utils/css";

  interface Props {
    children: Snippet;
    style?: CSSProperties;
  }

  const { children, style }: Props = $props();
</script>

<div class="badge" style={getCss(style || {})}>
  {@render children()}
</div>

<style lang="scss">
  .badge {
    background-color: var(--color-light);
    padding: 0.35rem 0.5rem 0.2rem 0.5rem;
    transform: translate(0, -0.15rem);
    border-radius: 1rem;
    font-family: var(--mono-font-family);
  }
</style>
