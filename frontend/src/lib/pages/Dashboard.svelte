<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { Session } from '$shared/Session';
  import type { SessionState } from '$shared/SessionState';
  import { listSessions } from '$lib/api/sessions';
  import { Button } from '$lib/components/ui/button/index.js';
  import SessionCard from '$lib/components/SessionCard.svelte';
  import NewSessionDialog from '$lib/components/NewSessionDialog.svelte';

  let sessions = $state<Session[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let dialogOpen = $state(false);

  const stateOrder: Record<SessionState, number> = {
    running: 0,
    provisioning: 1,
    stopped: 2,
    terminated: 3,
  };

  let sorted = $derived(
    [...sessions].sort((a, b) => stateOrder[a.state] - stateOrder[b.state])
  );

  async function fetchSessions() {
    try {
      const data = await listSessions();
      sessions = data.sessions;
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to fetch sessions';
    } finally {
      loading = false;
    }
  }

  fetchSessions();
  const interval = setInterval(fetchSessions, 5000);
  onDestroy(() => clearInterval(interval));
</script>

<div class="space-y-6">
  <div class="flex items-center justify-between">
    <h1 class="text-2xl font-bold">Sessions</h1>
    <Button onclick={() => (dialogOpen = true)}>New Session</Button>
  </div>

  {#if loading}
    <p class="text-muted-foreground">Loading sessions...</p>
  {:else if error}
    <p class="text-destructive">{error}</p>
  {:else if sorted.length === 0}
    <div class="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed p-12 text-center">
      <p class="text-muted-foreground">No sessions yet.</p>
      <Button variant="outline" onclick={() => (dialogOpen = true)}>Create your first session</Button>
    </div>
  {:else}
    <div class="grid gap-3">
      {#each sorted as session (session.id)}
        <SessionCard {session} />
      {/each}
    </div>
  {/if}
</div>

<NewSessionDialog bind:open={dialogOpen} />
