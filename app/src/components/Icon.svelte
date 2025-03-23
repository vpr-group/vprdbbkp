<script lang="ts">
  export type IconName =
    | "plus-circle"
    | "plus"
    | "pencil"
    | "reload"
    | "arrow-left"
    | "arrow-right"
    | "cross"
    | "upload"
    | "checkbox-blank"
    | "checkbox"
    | "cloud"
    | "hard-drive"
    | "database";

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
{:else if icon === "upload"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M7.818 1.182a.45.45 0 0 0-.636 0l-3 3a.45.45 0 1 0 .636.636L7.05 2.586V9.5a.45.45 0 1 0 .9 0V2.586l2.232 2.232a.45.45 0 1 0 .636-.636zM2.5 10a.5.5 0 0 1 .5.5V12c0 .554.446 1 .996 1h7.005A1 1 0 0 0 12 12v-1.5a.5.5 0 1 1 1 0V12a2 2 0 0 1-1.999 2H3.996A1.997 1.997 0 0 1 2 12v-1.5a.5.5 0 0 1 .5-.5"
      clip-rule="evenodd"
    /></svg
  >
{:else if icon === "arrow-right"}
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 15 15"
    ><!-- Icon from Radix Icons by WorkOS - https://github.com/radix-ui/icons/blob/master/LICENSE --><path
      fill="currentColor"
      fill-rule="evenodd"
      d="M8.146 3.146a.5.5 0 0 1 .708 0l4 4a.5.5 0 0 1 0 .708l-4 4a.5.5 0 0 1-.708-.708L11.293 8H2.5a.5.5 0 0 1 0-1h8.793L8.146 3.854a.5.5 0 0 1 0-.708"
      clip-rule="evenodd"
    /></svg
  >
{:else if icon === "checkbox"}
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="1em"
    height="1em"
    viewBox="0 0 24 24"
    ><path
      fill="currentColor"
      d="m10.6 13.4l-2.15-2.15q-.275-.275-.7-.275t-.7.275t-.275.7t.275.7L9.9 15.5q.3.3.7.3t.7-.3l5.65-5.65q.275-.275.275-.7t-.275-.7t-.7-.275t-.7.275zM5 21q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h14q.825 0 1.413.588T21 5v14q0 .825-.587 1.413T19 21z"
    /></svg
  >
{:else if icon === "checkbox-blank"}
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="1em"
    height="1em"
    viewBox="0 0 24 24"
    ><path
      fill="currentColor"
      d="M5 21q-.825 0-1.412-.587T3 19V5q0-.825.588-1.412T5 3h14q.825 0 1.413.588T21 5v14q0 .825-.587 1.413T19 21zm0-2h14V5H5z"
    /></svg
  >
{:else if icon === "cloud"}
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="1em"
    height="1em"
    viewBox="0 0 24 24"
    ><!-- Icon from Material Symbols by Google - https://github.com/google/material-design-icons/blob/master/LICENSE --><path
      fill="currentColor"
      d="M6.5 20q-2.275 0-3.887-1.575T1 14.575q0-1.95 1.175-3.475T5.25 9.15q.625-2.3 2.5-3.725T12 4q2.925 0 4.963 2.038T19 11q1.725.2 2.863 1.488T23 15.5q0 1.875-1.312 3.188T18.5 20zm0-2h12q1.05 0 1.775-.725T21 15.5t-.725-1.775T18.5 13H17v-2q0-2.075-1.463-3.538T12 6T8.463 7.463T7 11h-.5q-1.45 0-2.475 1.025T3 14.5t1.025 2.475T6.5 18m5.5-6"
    /></svg
  >
{:else if icon === "hard-drive"}
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="1em"
    height="1em"
    viewBox="0 0 24 24"
    ><!-- Icon from Material Symbols by Google - https://github.com/google/material-design-icons/blob/master/LICENSE --><path
      fill="currentColor"
      d="M4 17h16v-6H4zm13-1.5q.625 0 1.063-.437T18.5 14t-.437-1.062T17 12.5t-1.062.438T15.5 14t.438 1.063T17 15.5M22 9h-2.825l-2-2H6.825l-2 2H2l4-4h12zM2 19V9h20v10z"
    /></svg
  >
{:else if icon === "database"}
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="1em"
    height="1em"
    viewBox="0 0 24 24"
    ><!-- Icon from Material Symbols by Google - https://github.com/google/material-design-icons/blob/master/LICENSE --><path
      fill="currentColor"
      d="M12 21q-3.775 0-6.387-1.162T3 17V7q0-1.65 2.638-2.825T12 3t6.363 1.175T21 7v10q0 1.675-2.613 2.838T12 21m0-11.975q2.225 0 4.475-.638T19 7.025q-.275-.725-2.512-1.375T12 5q-2.275 0-4.462.638T5 7.025q.35.75 2.538 1.375T12 9.025M12 14q1.05 0 2.025-.1t1.863-.288t1.675-.462T19 12.525v-3q-.65.35-1.437.625t-1.675.463t-1.863.287T12 11t-2.05-.1t-1.888-.288T6.4 10.15T5 9.525v3q.625.35 1.4.625t1.663.463t1.887.287T12 14m0 5q1.15 0 2.338-.175t2.187-.462t1.675-.65t.8-.738v-2.45q-.65.35-1.437.625t-1.675.463t-1.863.287T12 16t-2.05-.1t-1.888-.288T6.4 15.15T5 14.525V17q.125.375.788.725t1.662.638t2.2.462T12 19"
    /></svg
  >
{/if}

<style lang="scss">
  svg {
    width: 1.1rem;
    height: 1.1rem;
  }
</style>
