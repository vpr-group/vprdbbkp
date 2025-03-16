<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type SourceConfig } from "../services/store";
  import Card from "./Card.svelte";
  import { ActionsService } from "../services/actions";
  import StatusDot from "./StatusDot.svelte";

  interface Props {
    sourceConfig: SourceConfig;
  }

  const actionService = new ActionsService();
  const storeService = new StoreService();
  const { sourceConfig }: Props = $props();

  let connected = $state(false);

  onMount(async () => {
    await storeService.waitForInitialized();
    actionService.verifySourceConnection(sourceConfig).then((res) => {
      connected = res.connected;
    });
  });
</script>

{#snippet title()}
  <div class="backup-source-card__title">
    <StatusDot status={connected ? "success" : undefined} />
    {sourceConfig.name}
  </div>
{/snippet}

<Card
  href={`/backup-sources/${sourceConfig.id}`}
  subTitle={`${sourceConfig.type.toLowerCase()}`}
  {title}
></Card>

<style lang="scss">
  .backup-source-card {
    &__title {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }
  }
</style>
