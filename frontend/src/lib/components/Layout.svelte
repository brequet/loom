<script lang="ts">
  import type { Snippet } from 'svelte';
  import { link } from 'svelte-spa-router';

  let { children }: { children?: Snippet } = $props();

  // Track current hash route for active nav highlighting
  let currentPath = $state(window.location.hash.slice(1) || '/');

  function onHashChange() {
    currentPath = window.location.hash.slice(1) || '/';
  }

  function isActive(path: string): boolean {
    if (path === '/') return currentPath === '/' || currentPath === '';
    return currentPath.startsWith(path);
  }
</script>

<svelte:window onhashchange={onHashChange} />

<div class="min-h-screen bg-background text-foreground">
  <header class="border-b">
    <nav class="container mx-auto flex h-14 items-center gap-6 px-4">
      <a href="/" use:link class="text-lg font-bold tracking-tight">Loom</a>
      <div class="flex items-center gap-4 text-sm">
        <a
          href="/"
          use:link
          class="transition-colors hover:text-foreground {isActive('/') ? 'text-foreground font-medium' : 'text-muted-foreground'}"
        >Dashboard</a>
        <a
          href="/settings"
          use:link
          class="transition-colors hover:text-foreground {isActive('/settings') ? 'text-foreground font-medium' : 'text-muted-foreground'}"
        >Settings</a>
      </div>
    </nav>
  </header>
  <main class="container mx-auto px-4 py-6">
    {#if children}
      {@render children()}
    {/if}
  </main>
</div>
