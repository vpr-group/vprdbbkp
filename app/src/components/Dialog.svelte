<script lang="ts">
  import { Dialog } from "bits-ui";
  import type { Snippet } from "svelte";

  interface Props {
    label?: string;
    children?: Snippet;
    trigger?: Snippet;
  }

  const { label, children, trigger }: Props = $props();
</script>

<Dialog.Root>
  <Dialog.Trigger child={trigger}>{label || "Dialog"}</Dialog.Trigger>
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
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background-color: rgb(224, 217, 209);
      opacity: 0.8;
    }

    &__content {
      position: fixed;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background-color: rgb(245, 243, 240);
      padding: 1rem;
      border-radius: 0.5rem;
      border: solid 1px rgb(216, 210, 203);
    }
  }
</style>
