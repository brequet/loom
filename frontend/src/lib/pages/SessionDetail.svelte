<script lang="ts">
  import type { Session } from '$shared/Session';
  import { getSession, stopSession, resumeSession, terminateSession } from '$lib/api/sessions';
  import { createQuery, createMutation, useQueryClient } from '@tanstack/svelte-query';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { Badge } from '$lib/components/ui/badge/index.js';
  import StateBadge from '$lib/components/StateBadge.svelte';
  import { push } from 'svelte-spa-router';

  let { params = {} }: { params?: { id?: string } } = $props();

  const queryClient = useQueryClient();
  let autoOpened = false;
  const shouldAutoOpen = window.location.hash.includes('autoOpen=1');

  // Remove autoOpen from URL immediately so refresh won't re-trigger
  if (shouldAutoOpen) {
    window.location.hash = window.location.hash.replace(/[?&]autoOpen=1/, "");
  }

  const sessionQuery = createQuery(() => ({
    queryKey: ['sessions', params.id],
    queryFn: () => getSession(params.id!),
    enabled: !!params.id,
    refetchInterval: (query: { state: { data?: Session } }) => {
      const s = query.state.data;
      return s && (s.state === 'provisioning' || s.state === 'running') ? 3_000 : false;
    },
  }));

  // Auto-open OpenCode when session becomes running
  $effect(() => {
    const s = sessionQuery.data;
    if (shouldAutoOpen && s && s.state === 'running' && !autoOpened) {
      const url = getOpenCodeUrl(s);
      if (url) {
        autoOpened = true;
        window.open(url, '_blank');
      }
    }
  });

  function formatDate(iso: string): string {
    return new Date(iso).toLocaleString();
  }

  function getOpenCodeUrl(s: Session): string | null {
    if (!s.opencode_port) return null;
    const base = `http://${window.location.hostname}:${s.opencode_port}`;
    if (s.opencode_path_prefix && s.opencode_session_id) {
      return `${base}/${s.opencode_path_prefix}/session/${s.opencode_session_id}`;
    }
    if (s.opencode_session_id) {
      return `${base}/session/${s.opencode_session_id}`;
    }
    return base;
  }

  const stopMutation = createMutation(() => ({
    mutationFn: () => stopSession(sessionQuery.data!.id),
    onSuccess: (data: Session) => {
      queryClient.setQueryData(['sessions', params.id], data);
      queryClient.invalidateQueries({ queryKey: ['sessions'], exact: true });
    },
    onError: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions', params.id] });
    },
  }));

  const resumeMutation = createMutation(() => ({
    mutationFn: () => resumeSession(sessionQuery.data!.id),
    onSuccess: (data: Session) => {
      queryClient.setQueryData(['sessions', params.id], data);
      queryClient.invalidateQueries({ queryKey: ['sessions'], exact: true });
    },
    onError: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions', params.id] });
    },
  }));

  const terminateMutation = createMutation(() => ({
    mutationFn: () => terminateSession(sessionQuery.data!.id),
    onSuccess: (data: Session) => {
      queryClient.setQueryData(['sessions', params.id], data);
      queryClient.invalidateQueries({ queryKey: ['sessions'], exact: true });
    },
    onError: () => {
      queryClient.invalidateQueries({ queryKey: ['sessions', params.id] });
    },
  }));

  function handleTerminate() {
    if (!confirm('Terminate this session? The workspace will be permanently deleted.')) return;
    terminateMutation.mutate();
  }

  let isMutating = $derived(stopMutation.isPending || resumeMutation.isPending || terminateMutation.isPending);

  let session = $derived(sessionQuery.data ?? null);

  let fields = $derived(session ? [
    { label: 'Session ID', value: session.id, mono: true },
    { label: 'Workspace', value: session.workspace_path, mono: true },
    { label: 'Port', value: session.opencode_port ? String(session.opencode_port) : null },
    { label: 'Model', value: session.model, mono: true },
    { label: 'Created', value: formatDate(session.created_at) },
    { label: 'Updated', value: formatDate(session.updated_at) },
  ] : []);
</script>

<svelte:head>
  <title>{session ? `${session.title} | Loom` : 'Loom'}</title>
</svelte:head>

<div class="space-y-8 max-w-3xl mx-auto">
  <button
    class="text-sm text-muted-foreground hover:text-foreground transition-colors inline-flex items-center gap-1"
    onclick={() => push('/')}
  >
    <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m15 18-6-6 6-6"/></svg>
    Back
  </button>

  {#if sessionQuery.isPending}
    <p class="text-muted-foreground">Loading session...</p>
  {:else if sessionQuery.isError}
    <p class="text-destructive">{sessionQuery.error.message}</p>
  {:else if session}
    <!-- Header -->
    <div class="space-y-3">
      <div class="flex items-start justify-between gap-4">
        <div class="space-y-1 min-w-0">
          <h1 class="text-2xl font-bold tracking-tight">{session.title}</h1>
          <div class="flex items-center gap-2 text-sm text-muted-foreground">
            <Badge variant="secondary" class="capitalize text-xs">{session.source_type}</Badge>
            {#if session.source_ref}
              <span class="font-mono text-xs">{session.source_ref}</span>
            {/if}
          </div>
        </div>
        <StateBadge state={session.state} />
      </div>

      <!-- Actions bar -->
      <div class="flex items-center gap-2">
        {#if session.state === 'running'}
          {#if getOpenCodeUrl(session)}
            <Button size="sm" onclick={() => window.open(getOpenCodeUrl(session!)!, '_blank')}>
              Open OpenCode
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="ml-1"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/><polyline points="15 3 21 3 21 9"/><line x1="10" y1="14" x2="21" y2="3"/></svg>
            </Button>
          {/if}
          <Button variant="outline" size="sm" onclick={() => stopMutation.mutate()} disabled={isMutating}>
            {#if stopMutation.isPending}
              <div class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent mr-1"></div>
              Stopping...
            {:else}
              Stop
            {/if}
          </Button>
        {:else if session.state === 'stopped'}
          <Button size="sm" onclick={() => resumeMutation.mutate()} disabled={isMutating}>
            {resumeMutation.isPending ? 'Resuming...' : 'Resume'}
          </Button>
          <Button variant="destructive" size="sm" onclick={handleTerminate} disabled={isMutating}>
            {terminateMutation.isPending ? 'Terminating...' : 'Terminate'}
          </Button>
        {:else if session.state === 'provisioning'}
          <div class="flex items-center gap-2 text-sm text-muted-foreground">
            <div class="h-3.5 w-3.5 animate-spin rounded-full border-2 border-current border-t-transparent"></div>
            Provisioning...
          </div>
        {/if}
      </div>
    </div>

    <Separator />

    <!-- Details -->
    <div class="space-y-1">
      {#each fields as field}
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-28 shrink-0">{field.label}</span>
          <span class="text-sm {field.mono ? 'font-mono' : ''} truncate">{field.value ?? '—'}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>
