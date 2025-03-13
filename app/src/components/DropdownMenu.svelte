<script lang="ts">
  import { DropdownMenu } from "bits-ui";
  import Button from "./Button.svelte";
  import type { IconName } from "./Icon.svelte";
  import type { Snippet } from "svelte";

  type T = $$Generic;

  interface Props<T> {
    open?: boolean;
    label?: string;
    icon?: IconName;
    items?: T[];
    item?: Snippet<[item: T]>;
    align?: "start" | "center" | "end";
  }

  let {
    open = $bindable(false),
    label,
    icon,
    items,
    item,
    align,
  }: Props<T> = $props();
</script>

<DropdownMenu.Root bind:open>
  <DropdownMenu.Trigger>
    {#snippet child({ props })}
      <Button {...props} {icon}>{label || "Dropdown"}</Button>
    {/snippet}
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content sideOffset={10} {align}>
      <div class="dropdown__content">
        {#each items || [] as it}
          {#if item}
            {@render item(it)}
          {/if}
        {/each}
      </div>
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>

<style lang="scss">
  .dropdown {
    &__content {
      display: flex;
      flex-direction: column;
      gap: 4px;
    }
  }
</style>
