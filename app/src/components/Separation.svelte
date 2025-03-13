<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    preLabel?: string;
    label?: string | Snippet;
    subLabel?: string;
    sideSection?: Snippet;
  }

  const { preLabel, label, subLabel, sideSection }: Props = $props();
</script>

<div class="separation">
  {#if preLabel}
    <span class="separation__pre-label">{preLabel}</span>
  {/if}

  {#if label}
    <div class="separation__label">
      <h4>
        {#if label}
          {#if typeof label === "string"}
            {label}
          {:else}
            {@render label()}
          {/if}
        {/if}
      </h4>

      {#if sideSection}
        <div class="separation__side-section">
          {@render sideSection()}
        </div>
      {/if}
    </div>
  {/if}

  {#if subLabel}
    <span class="separation__sub-label">{subLabel}</span>
  {/if}
</div>

<style lang="scss">
  .separation {
    width: 100%;
    border-bottom: solid 1px var(--color-light-grey);
    padding: 0.5rem 0;
    line-height: 1.3;

    &__label {
      display: flex;
      justify-content: space-between;
      align-items: center;
    }

    &__pre-label,
    &__sub-label {
      font-family: var(--mono-font-family);
      color: var(--color-grey);
    }

    &__side-section {
      display: flex;
      gap: 4px;
    }

    h4 {
      font-size: 1.5rem;
      font-weight: 600;
      margin: 0;
    }
  }
</style>
