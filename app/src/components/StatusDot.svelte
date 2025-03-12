<script lang="ts">
  interface Props {
    status?: "success";
  }

  const { status }: Props = $props();
</script>

<div class="status-dot" class:status-dot--success={status === "success"}></div>

<style lang="scss">
  .status-dot {
    width: 10px;
    height: 10px;
    background-color: var(--color-grey);
    border-radius: 50%;

    &--success {
      background-color: rgb(93, 212, 93);
    }
  }
</style>
