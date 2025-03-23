<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type SourceConfig } from "../services/store";
  import Card from "./Card.svelte";
  import { ActionsService } from "../services/actions";
  import Icon from "./Icon.svelte";
  import { getCss } from "../utils/css";

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
  <div class="source-config-card__title">
    <Icon icon="database" />

    {sourceConfig.name}
  </div>
{/snippet}

<Card href={`/source-configs/${sourceConfig.id}`} {title}>
  <div class="source-config-card__content">
    <span class="source-config-card__type">
      {#if sourceConfig.type === "pg"}
        PostgreSQL •
      {/if}

      <div
        class="source-config-card__badge"
        style={getCss({
          backgroundColor: connected ? "var(--color-light-green)" : undefined,
        })}
      >
        {connected ? "Connected" : "Offline"}
      </div>
      <!-- <StatusDot status={connected ? "success" : undefined} /> -->
    </span>

    {#if sourceConfig.type === "pg"}
      <div class="source-config-card__row">
        <span>Host:</span>
        <span>{sourceConfig.host}</span>
      </div>
      <div class="source-config-card__row">
        <span>Port:</span>
        <span>{sourceConfig.port}</span>
      </div>
      <div class="source-config-card__row">
        <span>Database:</span>
        <span>{sourceConfig.database}</span>
      </div>
      <div class="source-config-card__row">
        <span>Use Tunnel:</span>
        <span>{Boolean(sourceConfig.tunnelConfig?.useTunnel)}</span>
      </div>
    {/if}
  </div>
</Card>

<style lang="scss">
  .source-config-card {
    &__title {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }

    &__content {
      display: flex;
      flex-direction: column;
      font-family: var(--mono-font-family);
    }

    &__type {
      margin-top: 0.2rem;
      margin-bottom: 1rem;
      color: var(--color-grey);
      display: flex;
      align-items: center;
      gap: 0.7em;
      line-height: 1;
    }

    &__badge {
      background-color: var(--color-light);
      padding: 0.35rem 0.5rem 0.2rem 0.5rem;
      transform: translate(0, -0.15rem);
      border-radius: 1rem;
    }

    &__row {
      display: flex;
      justify-content: space-between;
      gap: 2rem;

      span {
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;

        &:first-child {
          flex: 0 0 auto;
        }
      }
    }
  }
</style>
