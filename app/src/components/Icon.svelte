<script lang="ts">
  export type IconName =
    | "plus-circle"
    | "plus"
    | "pencil"
    | "reload"
    | "arrow-left"
    | "cross";

  interface Props {
    icon: IconName;
  }

  const { icon }: Props = $props();
</script>

{#if icon === "plus-circle"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M7.5.877a6.623 6.623 0 1 0 0 13.246A6.623 6.623 0 0 0 7.5.877M1.827 7.5a5.673 5.673 0 1 1 11.346 0a5.673 5.673 0 0 1-11.346 0M7.5 4a.5.5 0 0 1 .5.5V7h2.5a.5.5 0 1 1 0 1H8v2.5a.5.5 0 0 1-1 0V8H4.5a.5.5 0 0 1 0-1H7V4.5a.5.5 0 0 1 .5-.5"
      clip-rule="evenodd"
    /></svg
  >
{:else if icon === "plus"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M8 2.75a.5.5 0 0 0-1 0V7H2.75a.5.5 0 0 0 0 1H7v4.25a.5.5 0 0 0 1 0V8h4.25a.5.5 0 0 0 0-1H8z"
      clip-rule="evenodd"
    /></svg
  >
{:else if icon === "pencil"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M11.854 1.146a.5.5 0 0 0-.707 0L3.714 8.578a1 1 0 0 0-.212.314L2.04 12.303a.5.5 0 0 0 .657.657l3.411-1.463a1 1 0 0 0 .314-.211l7.432-7.432a.5.5 0 0 0 0-.708zm-7.432 8.14l7.078-7.08L12.793 3.5l-7.078 7.078l-1.496.641l-.438-.438z"
      clip-rule="evenodd"
    /></svg
  >
{:else if icon === "reload"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M1.85 7.5c0-2.835 2.21-5.65 5.65-5.65c2.778 0 4.152 2.056 4.737 3.15H10.5a.5.5 0 0 0 0 1h3a.5.5 0 0 0 .5-.5v-3a.5.5 0 0 0-1 0v1.813C12.296 3.071 10.666.85 7.5.85C3.437.85.85 4.185.85 7.5s2.587 6.65 6.65 6.65c1.944 0 3.562-.77 4.714-1.942a6.8 6.8 0 0 0 1.428-2.167a.5.5 0 1 0-.925-.38a5.8 5.8 0 0 1-1.216 1.846c-.971.99-2.336 1.643-4.001 1.643c-3.44 0-5.65-2.815-5.65-5.65"
      clip-rule="evenodd"
    /></svg
  >
{:else if icon === "arrow-left"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M6.854 3.146a.5.5 0 0 1 0 .708L3.707 7H12.5a.5.5 0 0 1 0 1H3.707l3.147 3.146a.5.5 0 0 1-.708.708l-4-4a.5.5 0 0 1 0-.708l4-4a.5.5 0 0 1 .708 0"
      clip-rule="evenodd"
    /></svg
  >
{:else if icon === "cross"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M12.854 2.854a.5.5 0 0 0-.708-.708L7.5 6.793L2.854 2.146a.5.5 0 1 0-.708.708L6.793 7.5l-4.647 4.646a.5.5 0 0 0 .708.708L7.5 8.207l4.646 4.647a.5.5 0 0 0 .708-.708L8.207 7.5z"
      clip-rule="evenodd"
    /></svg
  >
{/if}

<style lang="scss">
  svg {
    width: 1.1rem;
    height: 1.1rem;
  }
</style>
