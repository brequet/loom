<script lang="ts">
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { get } from '$lib/api/client';
  import { createQuery } from '@tanstack/svelte-query';
  import type { HealthResponse } from '$shared/HealthResponse';
  import type { AppConfig } from '$shared/AppConfig';

  const healthQuery = createQuery(() => ({
    queryKey: ['health'],
    queryFn: () => get<HealthResponse>('/health'),
    staleTime: 30_000,
    retry: false,
  }));

  const configQuery = createQuery(() => ({
    queryKey: ['config'],
    queryFn: () => get<AppConfig>('/config'),
    staleTime: Infinity,
  }));

  let loading = $derived(healthQuery.isPending || configQuery.isPending);
  let health = $derived(healthQuery.data ?? null);
  let config = $derived(configQuery.data ?? null);
</script>

<div class="space-y-8 max-w-2xl mx-auto">
  <h1 class="text-2xl font-bold tracking-tight">Settings</h1>

  {#if loading}
    <p class="text-muted-foreground">Loading...</p>
  {:else}
    <!-- Server Status -->
    <section class="space-y-3">
      <h2 class="text-sm font-medium text-muted-foreground uppercase tracking-wide">Server</h2>
      <div class="space-y-1">
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-32 shrink-0">Status</span>
          {#if health}
            <span class="text-sm font-medium text-green-600">{health.status}</span>
          {:else}
            <span class="text-sm text-destructive">Unreachable</span>
          {/if}
        </div>
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-32 shrink-0">Version</span>
          <span class="text-sm font-mono">{health?.version ?? '—'}</span>
        </div>
      </div>
    </section>

    <Separator />

    <!-- Integrations -->
    <section class="space-y-3">
      <h2 class="text-sm font-medium text-muted-foreground uppercase tracking-wide">Integrations</h2>
      <div class="space-y-1">
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-32 shrink-0">Jira</span>
          {#if config?.jira_configured}
            <span class="text-sm text-green-600">Configured</span>
          {:else}
            <span class="text-sm text-muted-foreground">Not configured</span>
          {/if}
        </div>
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-32 shrink-0">GitLab</span>
          {#if config?.gitlab_configured}
            <span class="text-sm text-green-600">Configured</span>
          {:else}
            <span class="text-sm text-muted-foreground">Not configured</span>
          {/if}
        </div>
      </div>
    </section>

    <Separator />

    <!-- Models -->
    <section class="space-y-3">
      <h2 class="text-sm font-medium text-muted-foreground uppercase tracking-wide">Models</h2>
      <div class="flex items-baseline py-2 gap-4">
        <span class="text-sm text-muted-foreground w-32 shrink-0">Default</span>
        <span class="text-sm font-mono">{config?.default_model ?? '—'}</span>
      </div>
      {#if config?.models.length}
        <div class="space-y-1">
          {#each config.models as model}
            <div class="flex items-baseline py-1.5 gap-4">
              <span class="text-sm w-32 shrink-0">{model.label}</span>
              <span class="text-sm font-mono text-muted-foreground">{model.id}</span>
            </div>
          {/each}
        </div>
      {/if}
    </section>

    <Separator />

    <!-- Paths -->
    <section class="space-y-3">
      <h2 class="text-sm font-medium text-muted-foreground uppercase tracking-wide">Paths</h2>
      <div class="space-y-1">
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-32 shrink-0">Sessions</span>
          <span class="text-sm font-mono truncate">{config?.sessions_dir ?? '—'}</span>
        </div>
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-32 shrink-0">Repos</span>
          <span class="text-sm font-mono truncate">{config?.repos_dir ?? '—'}</span>
        </div>
        <div class="flex items-baseline py-2 gap-4">
          <span class="text-sm text-muted-foreground w-32 shrink-0">Base prompt</span>
          <span class="text-sm font-mono truncate">{config?.base_prompt_path ?? '—'}</span>
        </div>
      </div>
    </section>
  {/if}
</div>
