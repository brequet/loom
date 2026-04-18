<script lang="ts" generics="T">
  import type { Snippet } from 'svelte';
  import { Input } from '$lib/components/ui/input/index.js';

  let {
    placeholder = 'Search...',
    onSearch,
    onSelect,
    renderItem,
  }: {
    placeholder?: string;
    onSearch: (query: string) => Promise<T[]>;
    onSelect: (item: T) => void;
    renderItem: Snippet<[T]>;
  } = $props();

  let query = $state('');
  let results = $state<T[]>([]);
  let loading = $state(false);
  let showResults = $state(false);
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;

  function handleInput(e: Event) {
    const value = (e.target as HTMLInputElement).value;
    query = value;
    clearTimeout(debounceTimer);

    if (!value.trim()) {
      results = [];
      showResults = false;
      return;
    }

    loading = true;
    debounceTimer = setTimeout(async () => {
      try {
        results = await onSearch(value.trim());
        showResults = results.length > 0;
      } catch {
        results = [];
        showResults = false;
      } finally {
        loading = false;
      }
    }, 300);
  }

  function select(item: T) {
    showResults = false;
    query = '';
    results = [];
    onSelect(item);
  }
</script>

<div class="relative">
  <Input {placeholder} value={query} oninput={handleInput} onfocus={() => { if (results.length) showResults = true; }} />
  {#if loading}
    <div class="absolute right-3 top-1/2 -translate-y-1/2 text-xs text-muted-foreground">Loading...</div>
  {/if}
  {#if showResults}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="absolute z-50 mt-1 w-full rounded-md border bg-popover shadow-md"
      onmousedown={(e: MouseEvent) => e.preventDefault()}
    >
      {#each results as item}
        <button
          class="w-full px-3 py-2 text-left text-sm hover:bg-accent transition-colors"
          onclick={() => select(item)}
        >
          {@render renderItem(item)}
        </button>
      {/each}
    </div>
  {/if}
</div>
