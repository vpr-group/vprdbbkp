<script lang="ts">
  interface Props {
    name: string;
    type?: "text";
    value?: string;
    oninput?: (
      event: Event & { currentTarget: EventTarget & HTMLInputElement },
    ) => void;
  }

  const { name, type = "text", value = $bindable(), oninput }: Props = $props();
</script>

<fieldset class="input">
  <label for={name}>{name}</label>
  <input {type} {name} {value} {oninput} />
</fieldset>

<style lang="scss">
  .input {
    border: none;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    padding: 0;

    label {
      color: var(--color-grey);
      font-family: var(--mono-font-family);
    }

    input {
      background-color: var(--color-light);
      border: none;
      font-size: inherit;
      font-family: inherit;

      padding: 0.3rem 0.7rem;
      border-radius: 0.2rem;

      &:focus {
        outline: none;
        box-shadow: var(--shadow);
      }
    }
  }
</style>
