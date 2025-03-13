<script lang="ts">
  import { Dialog } from "bits-ui";
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";
  import type { IconName } from "./Icon.svelte";
  import type { CSSProperties } from "../utils/css";

  interface Props {
    open?: boolean;
    label?: string;
    children?: Snippet;
    icon?: IconName;
    buttonStyle?: CSSProperties;
  }

  let {
    open = $bindable(false),
    label,
    children,
    icon,
    buttonStyle,
  }: Props = $props();
</script>

<Dialog.Root bind:open>
  <Dialog.Trigger>
    {#snippet child({ props })}
      <Button {...props} {icon} style={buttonStyle}>{label}</Button>
    {/snippet}
  </Dialog.Trigger>
  <Dialog.Portal>
    <Dialog.Overlay>
      <div class="dialog__overlay"></div>
    </Dialog.Overlay>
    <Dialog.Content>
      <div class="dialog__content">
        {#if children}
          {@render children()}
        {/if}
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>

<style lang="scss">
  .dialog {
    &__overlay {
      position: fixed;
      z-index: 20;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background-color: var(--color-light);
      opacity: 0.95;
    }

    &__content {
      position: fixed;
      z-index: 30;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background-color: white;
      padding: 1rem;
      border-radius: 0.5rem;
      box-shadow: var(--shadow);
    }
  }
</style>
