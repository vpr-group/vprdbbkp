<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type DatabaseConfig } from "../services/store";
  import Card from "./Card.svelte";
  import { ActionsService } from "../services/actions";
  import Icon from "./Icon.svelte";
  import { getCss } from "../utils/css";

  interface Props {
    databaseConfig: DatabaseConfig;
  }

  const actionService = new ActionsService();
  const storeService = new StoreService();
  const { databaseConfig }: Props = $props();

  let connected = $state(false);

  onMount(async () => {
    await storeService.waitForInitialized();
    actionService.testConnection(databaseConfig).then((res) => {
      connected = res.connected;
    });
  });
</script>

{#snippet title()}
  <div class="database-config-card__title">
    <Icon icon="database" />
    {databaseConfig.name}
  </div>
{/snippet}

<Card href={`/database-configs/${databaseConfig.id}`} {title}>
  <div class="database-config-card__content">
    <span class="database-config-card__type">
      {#if databaseConfig.connection_type === "PostgreSql"}
        PostgreSQL •
      {/if}

      <div
        class="database-config-card__badge"
        style={getCss({
          backgroundColor: connected ? "var(--color-light-green)" : undefined,
        })}
      >
        {connected ? "Connected" : "Offline"}
      </div>
      <!-- <StatusDot status={connected ? "success" : undefined} /> -->
    </span>

    {#if databaseConfig.connection_type === "PostgreSql"}
      <div class="database-config-card__row">
        <span>Host:</span>
        <span>{databaseConfig.host}</span>
      </div>
      <div class="database-config-card__row">
        <span>Port:</span>
        <span>{databaseConfig.port}</span>
      </div>
      <div class="database-config-card__row">
        <span>Database:</span>
        <span>{databaseConfig.database}</span>
      </div>
      <div class="database-config-card__row">
        <span>SSH Tunnel:</span>
        <span>{Boolean(databaseConfig.tunnelConfig?.useTunnel)}</span>
      </div>
    {/if}
  </div>
</Card>

<style lang="scss">
  .database-config-card {
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
      margin-bottom: 1.5rem;
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
