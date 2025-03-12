<script lang="ts">
  interface Props {
    name: string;
    type?: "text";
    value?: string;
    oninput?: (
      event: Event & { currentTarget: EventTarget & HTMLInputElement }
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

    input {
      background-color: rgb(220, 220, 220);
      border: none;
      font-size: inherit;
      padding: 0.5rem;
      border-radius: 0.2rem;
    }
  }
</style>
