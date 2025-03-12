<script lang="ts">
  import type { BackupSource } from "../services/store";
  import Card from "./Card.svelte";

  interface Props {
    backupSource: BackupSource;
  }

  const { backupSource }: Props = $props();
</script>

<Card href={`/backup-sources/${backupSource.id}`}>
  <span>{backupSource.type}</span>
  <h3>{backupSource.name}</h3>
</Card>

<style lang="scss">
  h3 {
    font-size: 2rem;
    margin: 0;
    text-transform: uppercase;
  }
</style>
