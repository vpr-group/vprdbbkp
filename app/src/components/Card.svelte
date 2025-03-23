<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    children?: Snippet;
    href?: string;
    preTitle?: string;
    title?: string | Snippet;
    subTitle?: string;
  }

  const { children, href, preTitle, title, subTitle }: Props = $props();
</script>

{#snippet cardInner()}
  {#if preTitle}
    <span class="card__pre-title">{preTitle}</span>
  {/if}

  {#if title}
    <h4 class="card__title">
      {#if title}
        {#if typeof title === "string"}
          {title}
        {:else}
          {@render title()}
        {/if}
      {/if}
    </h4>
  {/if}

  {#if subTitle}
    <span class="card__sub-title">{subTitle}</span>
  {/if}

  {#if children}
    {@render children()}
  {/if}
{/snippet}

{#if href}
  <a class="card card--link" {href}>{@render cardInner()}</a>
{:else}
  <div class="card">
    {@render cardInner()}
  </div>
{/if}

<style lang="scss">
  .card {
    display: flex;
    flex-direction: column;
    color: black;
    background-color: white;
    text-decoration: none;
    padding: 1rem;
    border-radius: var(--border-radius);
    box-shadow: var(--shadow);
    line-height: 1.5;

    &--link {
      &:hover {
        background-color: var(--color-light-grey);
      }
    }

    &__pre-title,
    &__sub-title {
      font-family: var(--mono-font-family);
    }

    &__pre-title {
      display: block;
    }

    &__sub-title {
      color: var(--color-grey);
    }

    &__title {
      font-size: 1.5rem;
      margin: 0;
    }
  }
</style>
