<script lang="ts" module>
  import { createId } from "@paralleldrive/cuid2";
  import type { Snippet } from "svelte";

  export interface Dialog {
    id: string;
    title?: string | Snippet;
    children?: string | Snippet;
    onopenchange?: (open: boolean) => void;
  }

  class Dialogs {
    dialogs: Dialog[] = $state([]);

    addDialog(dialog: Omit<Dialog, "id">): Dialog {
      const newDialog = {
        ...dialog,
        id: createId(),
      };

      this.dialogs.push(newDialog);

      return newDialog;
    }

    removeDialog(id: string) {
      this.dialogs = this.dialogs.filter((it) => it.id !== id);
    }
  }

  export const dialogsStore = new Dialogs();
</script>

<script lang="ts">
  import Separation from "./Separation.svelte";

  const lastDialogIndex = $derived(dialogsStore.dialogs.length - 1);
  const lastDialog = $derived<Dialog | undefined>(
    dialogsStore.dialogs[lastDialogIndex]
  );

  const onkeydown = (
    event: KeyboardEvent & { currentTarget: EventTarget & Window }
  ) => {
    if (event.key === "Escape" && lastDialog) {
      dialogsStore.removeDialog(lastDialog.id);
    }
  };
</script>

<svelte:window {onkeydown} />

{#each dialogsStore.dialogs as dialog, index}
  {#key dialog.id}
    <div
      class="dialog"
      class:dialog--active={index === dialogsStore.dialogs.length - 1}
    >
      {#if dialog.title}
        <Separation label={dialog.title} />
      {/if}

      {#if dialog.children}
        <div class="dialog__content">
          {#if typeof dialog.children === "string"}
            {dialog.children}
          {:else}
            {@render dialog.children()}
          {/if}
        </div>
      {/if}
    </div>
  {/key}
{/each}

{#if lastDialog}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="dialog__overlay"
    onclick={() => {
      lastDialog.onopenchange?.(false);
      dialogsStore.removeDialog(lastDialog.id);
    }}
  ></div>
{/if}

<style lang="scss">
  .dialog {
    display: none;
    flex-direction: column;
    gap: 1rem;
    position: fixed;
    z-index: 30;
    top: 50%;
    left: 50%;
    min-width: 25rem;
    transform: translate(-50%, -50%);
    background-color: white;
    padding: 1rem 1rem;
    border-radius: 0.5rem;
    box-shadow: var(--shadow);

    &--active {
      display: flex;
    }

    &__content {
      display: flex;
      flex-direction: column;
      /* padding: 1rem 0; */
      min-width: 30rem;
      gap: 0.5rem;
    }

    &__overlay {
      position: fixed;
      z-index: 20;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      background-color: rgba(242, 242, 242, 0.7);
      backdrop-filter: blur(10px);
      -webkit-backdrop-filter: blur(10px);
    }
  }
</style>
