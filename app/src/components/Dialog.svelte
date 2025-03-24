<script lang="ts">
  import type { Snippet } from "svelte";
  import Button from "./Button.svelte";
  import type { IconName } from "./Icon.svelte";
  import type { CSSProperties } from "../utils/css";
  import { dialogsStore, type Dialog } from "./Dialogs.svelte";

  interface Props {
    open?: boolean;
    label?: string;
    title?: Snippet | string;
    children?: Snippet;
    icon?: IconName;
    buttonStyle?: CSSProperties;
    onopenchange?: (open: boolean) => void;
  }

  let {
    open = $bindable(false),
    label,
    children,
    icon,
    buttonStyle,
    onopenchange,
    title,
  }: Props = $props();

  let dialog: Dialog | undefined = undefined;

  $effect(() => {
    if (open && !dialog) {
      dialog = dialogsStore.addDialog({
        title,
        children,
        onopenchange: (_open) => (open = _open),
      });
    } else if (!open && dialog) {
      dialogsStore.removeDialog(dialog.id);
    }

    // Cleanup
    const existingDialog = dialogsStore.dialogs.find(
      (it) => it.id === dialog?.id,
    );

    if (!existingDialog) {
      open = false;
      dialog = undefined;
    }

    onopenchange?.(open);
  });
</script>

<Button {icon} style={buttonStyle} onclick={() => (open = !open)}>
  {label}
</Button>
