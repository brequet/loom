<script lang="ts">
  import type { Session } from '$shared/Session';
  import { Card, CardHeader, CardTitle, CardContent } from '$lib/components/ui/card/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import StateBadge from './StateBadge.svelte';
  import { push } from 'svelte-spa-router';

  let { session }: { session: Session } = $props();

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function handleClick() {
    push(`/sessions/${session.id}`);
  }
</script>

<button class="w-full text-left" onclick={handleClick}>
  <Card class="cursor-pointer transition-colors hover:bg-muted/50">
    <CardHeader class="flex flex-row items-center justify-between gap-2 space-y-0 pb-2">
      <CardTitle class="text-base font-medium truncate">{session.title}</CardTitle>
      <div class="flex items-center gap-2 shrink-0">
        <Badge variant="secondary" class="capitalize">{session.source_type}</Badge>
        <StateBadge state={session.state} />
      </div>
    </CardHeader>
    <CardContent class="flex items-center gap-4 text-sm text-muted-foreground">
      {#if session.source_ref}
        <span class="font-mono">{session.source_ref}</span>
      {/if}
      {#if session.opencode_port}
        <span>Port {session.opencode_port}</span>
      {/if}
      <span class="ml-auto">{formatDate(session.created_at)}</span>
    </CardContent>
  </Card>
</button>
