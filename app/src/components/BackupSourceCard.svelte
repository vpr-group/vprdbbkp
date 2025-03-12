<script lang="ts">
  import { onMount } from "svelte";
  import { StoreService, type BackupSource } from "../services/store";
  import Card from "./Card.svelte";
  import { ActionsService } from "../services/actions";
  import StatusDot from "./StatusDot.svelte";

  interface Props {
    backupSource: BackupSource;
  }

  const actionService = new ActionsService();
  const storeService = new StoreService();
  const { backupSource }: Props = $props();

  let connected = $state(false);

  onMount(async () => {
    await storeService.waitForInitialized();
    actionService.verifyBackupSourceConnection(backupSource).then((res) => {
      connected = res.connected;
    });
  });
</script>

{#snippet title()}
  <div class="backup-source-card__title">
    <StatusDot status={connected ? "success" : undefined} />
    {backupSource.name}
  </div>
{/snippet}

<Card
  href={`/backup-sources/${backupSource.id}`}
  subTitle={`${backupSource.type.toLowerCase()}`}
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
