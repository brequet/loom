<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { Session } from '$shared/Session';
  import { getSession, stopSession, resumeSession, terminateSession } from '$lib/api/sessions';
  import { Card, CardHeader, CardTitle, CardContent } from '$lib/components/ui/card/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import StateBadge from '$lib/components/StateBadge.svelte';
  import { push } from 'svelte-spa-router';

  let { params = {} }: { params?: { id?: string } } = $props();

  let session = $state<Session | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let actionLoading = $state(false);
  let autoOpened = $state(false);

  // Check if we should auto-open (set by NewSessionDialog after creation)
  // svelte-spa-router uses hash-based routing, so query params are in the hash fragment
  const shouldAutoOpen = window.location.hash.includes('autoOpen=1');

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function getOpenCodeUrl(s: Session): string | null {
    if (!s.opencode_port) return null;
    const base = `http://localhost:${s.opencode_port}`;
    if (s.opencode_path_prefix && s.opencode_session_id) {
      return `${base}/${s.opencode_path_prefix}/session/${s.opencode_session_id}`;
    }
    // Fallback: even without prefix, try to navigate to the session
    // OpenCode's web UI will redirect from the base URL to the correct prefix
    if (s.opencode_session_id) {
      return `${base}/session/${s.opencode_session_id}`;
    }
    return base;
  }

  async function fetchSession() {
    if (!params.id) return;
    try {
      session = await getSession(params.id);
      error = null;
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to fetch session';
    } finally {
      loading = false;
    }
  }

  fetchSession();

  let interval: ReturnType<typeof setInterval> | undefined;

  $effect(() => {
    clearInterval(interval);
    if (session && (session.state === 'provisioning' || session.state === 'running')) {
      interval = setInterval(fetchSession, 3000);
    }
  });

  // Auto-open OpenCode tab when session first becomes running (only after fresh creation)
  $effect(() => {
    if (shouldAutoOpen && session && session.state === 'running' && !autoOpened) {
      const url = getOpenCodeUrl(session);
      if (url) {
        autoOpened = true;
        window.open(url, '_blank');
      }
    }
  });

  onDestroy(() => clearInterval(interval));

  async function handleStop() {
    if (!session || actionLoading) return;
    actionLoading = true;
    try {
      session = await stopSession(session.id);
    } finally {
      actionLoading = false;
    }
  }

  async function handleResume() {
    if (!session || actionLoading) return;
    actionLoading = true;
    try {
      session = await resumeSession(session.id);
    } finally {
      actionLoading = false;
    }
  }

  async function handleTerminate() {
    if (!session || actionLoading) return;
    actionLoading = true;
    try {
      session = await terminateSession(session.id);
    } finally {
      actionLoading = false;
    }
  }
</script>

<svelte:head>
  <title>{session ? `${session.title} | Loom` : 'Loom'}</title>
</svelte:head>

<div class="space-y-6">
  <button class="text-sm text-muted-foreground hover:text-foreground transition-colors" onclick={() => push('/')}>
    &larr; Back to Dashboard
  </button>

  {#if loading}
    <p class="text-muted-foreground">Loading session...</p>
  {:else if error}
    <p class="text-destructive">{error}</p>
  {:else if session}
    <Card>
      <CardHeader class="flex flex-row items-center justify-between gap-4 space-y-0">
        <div class="space-y-1">
          <CardTitle class="text-xl">{session.title}</CardTitle>
          <p class="text-sm text-muted-foreground capitalize">{session.source_type} source{#if session.source_ref} &middot; <span class="font-mono">{session.source_ref}</span>{/if}</p>
        </div>
        <StateBadge state={session.state} />
      </CardHeader>
      <Separator />
      <CardContent class="grid gap-4 pt-6 sm:grid-cols-2">
        <div>
          <p class="text-sm font-medium text-muted-foreground">Workspace</p>
          <p class="text-sm font-mono">{session.workspace_path ?? 'N/A'}</p>
        </div>
        <div>
          <p class="text-sm font-medium text-muted-foreground">Port</p>
          <p class="text-sm">{session.opencode_port ?? 'N/A'}</p>
        </div>
        <div>
          <p class="text-sm font-medium text-muted-foreground">Project ID</p>
          <p class="text-sm font-mono">{session.project_id ?? 'N/A'}</p>
        </div>
        <div>
          <p class="text-sm font-medium text-muted-foreground">Session ID</p>
          <p class="text-sm font-mono">{session.id}</p>
        </div>
        <div>
          <p class="text-sm font-medium text-muted-foreground">Created</p>
          <p class="text-sm">{formatDate(session.created_at)}</p>
        </div>
        <div>
          <p class="text-sm font-medium text-muted-foreground">Updated</p>
          <p class="text-sm">{formatDate(session.updated_at)}</p>
        </div>
      </CardContent>
    </Card>

    <div class="flex gap-3">
      {#if session.state === 'running'}
        {#if getOpenCodeUrl(session)}
          <Button onclick={() => window.open(getOpenCodeUrl(session)!, '_blank')}>
            Open OpenCode
          </Button>
        {/if}
        <Button variant="outline" onclick={handleStop} disabled={actionLoading}>
          {actionLoading ? 'Stopping...' : 'Stop'}
        </Button>
      {:else if session.state === 'stopped'}
        <Button onclick={handleResume} disabled={actionLoading}>
          {actionLoading ? 'Resuming...' : 'Resume'}
        </Button>
        <Button variant="destructive" onclick={handleTerminate} disabled={actionLoading}>
          {actionLoading ? 'Terminating...' : 'Terminate'}
        </Button>
      {:else if session.state === 'provisioning'}
        <div class="flex items-center gap-2 text-sm text-muted-foreground">
          <div class="h-4 w-4 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
          Provisioning...
        </div>
      {/if}
    </div>
  {/if}
</div>
