<script>
  import Button from "./Button.svelte";
</script>

<div class="sidebar">
  <div class="sidebar__inner">
    <span></span>
    <img src="/logo_vprs3bkp.svg" alt="" />

    <!-- <div class="sidebar__buttons">
      <Button style={{ width: "100%" }} icon="reload"></Button>
      <Button style={{ width: "100%" }} icon="plus"></Button>
    </div> -->
    <!-- <Button icon="database">+</Button> -->
    <!-- <span>Storage</span> -->
  </div>
</div>

<style lang="scss">
  .sidebar {
    position: sticky;
    top: 0;
    width: 5em;
    height: 100vh;
    padding: 1rem 0;
    flex: 0 0 auto;
    /* background-color: var(--color-light-grey); */
    /* box-shadow: var(--shadow); */

    &__inner {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      justify-content: space-between;
      width: 100%;
      height: 100%;
      border-right: solid 1px var(--color-light-grey);
      padding: 0 1rem;
    }

    &__buttons {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    img {
      display: flex;
      width: 3rem;
      height: 3rem;
      background-color: var(--color-light-grey);
      border-radius: var(--border-radius);
      padding: 0.3rem;
      box-sizing: border-box;
    }
  }
</style>
