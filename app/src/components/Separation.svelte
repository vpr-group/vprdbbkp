<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    preLabel?: string;
    label?: string;
    subLabel?: string;
    sideSection?: Snippet;
  }

  const { preLabel, label, subLabel, sideSection }: Props = $props();
</script>

<div class="separation">
  {#if preLabel}
    <span>{preLabel}</span>
  {/if}

  {#if label}
    <div class="separation__label">
      <h4>{label}</h4>

      {#if sideSection}
        <div class="separation__side-section">
          {@render sideSection()}
        </div>
      {/if}
    </div>
  {/if}

  {#if subLabel}
    <span>{subLabel}</span>
  {/if}
</div>

<style lang="scss">
  .separation {
    border-bottom: solid 1px var(--color-light-grey);
    padding: 0.5rem 0;

    &__label {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    h4 {
      font-size: 1.5rem;
      font-weight: 600;
      margin: 0;
    }
  }
</style>
