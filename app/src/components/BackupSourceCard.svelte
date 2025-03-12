<script lang="ts">
  import type { BackupSource } from "../services/store";
  import Card from "./Card.svelte";

  interface Props {
    backupSource: BackupSource;
  }

  const { backupSource }: Props = $props();
</script>

<Card
  href={`/backup-sources/${backupSource.id}`}
  subTitle={`${backupSource.type.toLowerCase()}`}
  title={backupSource.name}
></Card>
