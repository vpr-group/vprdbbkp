<script lang="ts">
  import { Checkbox } from "bits-ui";
  import Icon from "./Icon.svelte";
  import { getCss, type CSSProperties } from "../utils/css";

  interface Props {
    checked?: boolean;
    label?: string;
    oncheckedchange?: (checked: boolean) => void;
    style?: Partial<CSSProperties>;
  }

  let {
    checked = $bindable(false),
    label,
    oncheckedchange,
    style,
  }: Props = $props();
</script>

<Checkbox.Root
  class="checkbox"
  bind:checked
  {oninput}
  onCheckedChange={oncheckedchange}
  style={getCss(style || {})}
>
  {#snippet children({ checked })}
    {#if checked}
      <Icon icon="checkbox" />
    {:else}
      <Icon icon="checkbox-blank" />
    {/if}

    {#if label}
      <span>{label}</span>
    {/if}
  {/snippet}
</Checkbox.Root>

<style lang="scss">
  :global(.checkbox) {
    background-color: transparent;
    border: none;
    padding: 0;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    cursor: pointer;

    span {
      color: var(--color-grey);
      font-family: var(--mono-font-family);
      font-size: 1rem;
    }
  }
</style>
