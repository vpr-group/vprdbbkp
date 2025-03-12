<script lang="ts">
  import { Dialog } from "bits-ui";
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";

  interface Props {
    label?: string;
    children?: Snippet;
    trigger?: Snippet;
  }

  const { label, children, trigger }: Props = $props();
</script>

{#snippet defaultTriggerChild()}
  <Button>{label || "Dialog"}</Button>
{/snippet}

<Dialog.Root>
  <Dialog.Trigger child={trigger || defaultTriggerChild}></Dialog.Trigger>
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
