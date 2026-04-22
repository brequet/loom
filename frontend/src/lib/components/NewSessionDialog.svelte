<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '$lib/components/ui/dialog/index.js';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select/index.js';
  import { get } from '$lib/api/client';
  import { getJiraIssue } from '$lib/api/jira';
  import { createSession } from '$lib/api/sessions';
  import { createQuery, createMutation, useQueryClient } from '@tanstack/svelte-query';
  import type { AppConfig } from '$shared/AppConfig';
  import type { Session } from '$shared/Session';
  import { push } from 'svelte-spa-router';

  let { open = $bindable(false) }: { open?: boolean } = $props();

  const queryClient = useQueryClient();

  const configQuery = createQuery(() => ({
    queryKey: ['config'],
    queryFn: () => get<AppConfig>('/config'),
    staleTime: Infinity,
    enabled: open,
  }));

  let models = $derived(configQuery.data?.models ?? []);
  let defaultModel = $derived(configQuery.data?.default_model ?? '');
  let jiraConfigured = $derived(configQuery.data?.jira_configured ?? false);
  let gitlabConfigured = $derived(configQuery.data?.gitlab_configured ?? false);

  let selectedModel = $state<string>('');
  let scratchTitle = $state('');
  let gitlabUrl = $state('');
  let jiraInput = $state('');
  let customInstructions = $state('');
  let gitlabError = $state('');
  let jiraError = $state('');
  let scratchError = $state('');
  let activeTab = $state('');

  // Set initial active tab when config loads
  $effect(() => {
    if (!activeTab && configQuery.data) {
      activeTab = jiraConfigured ? 'jira' : gitlabConfigured ? 'gitlab' : 'scratch';
    }
  });

  // Set default model when config loads
  $effect(() => {
    if (defaultModel && !selectedModel) {
      selectedModel = defaultModel;
    }
  });

  // Reset form when dialog closes
  $effect(() => {
    if (!open) {
      scratchTitle = '';
      gitlabUrl = '';
      jiraInput = '';
      customInstructions = '';
      jiraError = '';
      gitlabError = '';
      scratchError = '';
      selectedModel = defaultModel;
    }
  });

  function selectedModelLabel(): string {
    return models.find((m) => m.id === selectedModel)?.label ?? selectedModel;
  }

  function parseJiraKey(input: string): string | null {
    const trimmed = input.trim();
    const keyMatch = trimmed.match(/^([A-Z]+-\d+)$/i);
    if (keyMatch) return keyMatch[1].toUpperCase();

    const urlMatch = trimmed.match(/browse\/([A-Z]+-\d+)/i);
    if (urlMatch) return urlMatch[1].toUpperCase();

    return null;
  }

  function navigateToSession(session: Session) {
    open = false;
    queryClient.invalidateQueries({ queryKey: ['sessions'] });
    push(`/sessions/${session.id}?autoOpen=1`);
  }

  const jiraMutation = createMutation(() => ({
    mutationFn: async () => {
      const key = parseJiraKey(jiraInput);
      if (!key) throw new Error('Enter a valid issue key (e.g. SAM-398) or full Jira URL.');
      const issue = await getJiraIssue(key);
      return createSession({
        source_type: 'jira',
        source_ref: issue.key,
        title: `${issue.key}: ${issue.summary}`,
        model: selectedModel,
        custom_instructions: customInstructions.trim() || undefined,
      });
    },
    onSuccess: (session: Session) => navigateToSession(session),
    onError: (e: Error) => { jiraError = e.message; },
  }));

  const gitlabMutation = createMutation(() => ({
    mutationFn: async () => {
      const url = gitlabUrl.trim();
      const mrMatch = url.match(/merge_requests\/(\d+)/);
      if (!mrMatch) throw new Error('Enter a valid GitLab merge request URL.');
      return createSession({
        source_type: 'gitlab',
        source_ref: url,
        title: `MR !${mrMatch[1]}`,
        model: selectedModel,
        custom_instructions: customInstructions.trim() || undefined,
      });
    },
    onSuccess: (session: Session) => navigateToSession(session),
    onError: (e: Error) => { gitlabError = e.message; },
  }));

  const scratchMutation = createMutation(() => ({
    mutationFn: () => createSession({
      source_type: 'scratch',
      title: scratchTitle.trim() || 'Scratch session',
      model: selectedModel,
      custom_instructions: customInstructions.trim() || undefined,
    }),
    onSuccess: (session: Session) => navigateToSession(session),
    onError: (e: Error) => { scratchError = e.message; },
  }));

  let creating = $derived(jiraMutation.isPending || gitlabMutation.isPending || scratchMutation.isPending);

  function handleCreateFromJira() {
    if (creating) return;
    jiraError = '';
    jiraMutation.mutate();
  }

  function handleCreateFromGitLabUrl() {
    if (creating || !gitlabUrl.trim()) return;
    gitlabError = '';
    gitlabMutation.mutate();
  }

  function handleCreateScratch() {
    if (creating) return;
    scratchError = '';
    scratchMutation.mutate();
  }

  function handleCtrlEnter(e: KeyboardEvent) {
    if (e.key === 'Enter' && (e.ctrlKey || e.metaKey)) {
      e.preventDefault();
      if (activeTab === 'jira') handleCreateFromJira();
      else if (activeTab === 'gitlab') handleCreateFromGitLabUrl();
      else handleCreateScratch();
    }
  }
</script>

<Dialog bind:open>
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <DialogContent class="sm:max-w-xl" onkeydown={handleCtrlEnter}>
    <DialogHeader>
      <DialogTitle>New Session</DialogTitle>
      <DialogDescription>Create a coding session from an issue tracker or start fresh.</DialogDescription>
    </DialogHeader>

    <div class="space-y-2">
      <Label for="model-select">Model</Label>
      <Select type="single" bind:value={selectedModel}>
        <SelectTrigger id="model-select">{selectedModelLabel()}</SelectTrigger>
        <SelectContent>
          {#each models as model}
            <SelectItem value={model.id}>{model.label}</SelectItem>
          {/each}
        </SelectContent>
      </Select>
    </div>

    <div class="space-y-2">
      <Label for="custom-instructions">Custom instructions</Label>
      <textarea
        id="custom-instructions"
        bind:value={customInstructions}
        placeholder="Additional instructions for the agent (optional)"
        rows="3"
        class="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 resize-none overflow-hidden"
        style="min-height: 80px; max-height: 200px;"
        oninput={(e: Event) => {
          const el = e.target as HTMLTextAreaElement;
          el.style.height = 'auto';
          el.style.height = Math.min(el.scrollHeight, 200) + 'px';
          el.style.overflow = el.scrollHeight > 200 ? 'auto' : 'hidden';
        }}
      ></textarea>
    </div>

    <Tabs value={activeTab} onValueChange={(v: string) => { activeTab = v; }}>
      <TabsList class="w-full">
        <TabsTrigger value="jira" class="flex-1" disabled={!jiraConfigured}>Jira{#if !jiraConfigured}<span class="ml-1 text-xs text-muted-foreground">(N/A)</span>{/if}</TabsTrigger>
        <TabsTrigger value="gitlab" class="flex-1" disabled={!gitlabConfigured}>GitLab{#if !gitlabConfigured}<span class="ml-1 text-xs text-muted-foreground">(N/A)</span>{/if}</TabsTrigger>
        <TabsTrigger value="scratch" class="flex-1">Scratch</TabsTrigger>
      </TabsList>

      <TabsContent value="jira" class="mt-4">
        <form
          class="space-y-4"
          onsubmit={(e: Event) => { e.preventDefault(); handleCreateFromJira(); }}
        >
          <div class="space-y-2">
            <Label for="jira-input">Issue key or URL</Label>
            <Input
              id="jira-input"
              placeholder="SAM-398 or https://team.atlassian.net/browse/SAM-398"
              bind:value={jiraInput}
              aria-invalid={!!jiraError}
              aria-describedby={jiraError ? 'jira-error' : undefined}
            />
            {#if jiraError}
              <p id="jira-error" class="text-sm text-destructive">{jiraError}</p>
            {/if}
          </div>
          <Button type="submit" disabled={creating || !jiraInput.trim()} class="w-full">
            {creating ? 'Creating...' : 'Create Session'}
          </Button>
        </form>
      </TabsContent>

      <TabsContent value="gitlab" class="mt-4">
        <form
          class="space-y-4"
          onsubmit={(e: Event) => { e.preventDefault(); handleCreateFromGitLabUrl(); }}
        >
          <div class="space-y-2">
            <Label for="gitlab-input">Merge request URL</Label>
            <Input
              id="gitlab-input"
              placeholder="https://gitlab.com/.../merge_requests/42"
              bind:value={gitlabUrl}
              aria-invalid={!!gitlabError}
              aria-describedby={gitlabError ? 'gitlab-error' : undefined}
            />
            {#if gitlabError}
              <p id="gitlab-error" class="text-sm text-destructive">{gitlabError}</p>
            {/if}
          </div>
          <Button type="submit" disabled={creating || !gitlabUrl.trim()} class="w-full">
            {creating ? 'Creating...' : 'Create Session'}
          </Button>
        </form>
      </TabsContent>

      <TabsContent value="scratch" class="mt-4">
        <form
          class="space-y-4"
          onsubmit={(e: Event) => { e.preventDefault(); handleCreateScratch(); }}
        >
          <div class="space-y-2">
            <Label for="scratch-title">Session title</Label>
            <Input
              id="scratch-title"
              placeholder="Optional -- defaults to 'Scratch session'"
              bind:value={scratchTitle}
            />
            {#if scratchError}
              <p class="text-sm text-destructive">{scratchError}</p>
            {/if}
          </div>
          <Button type="submit" disabled={creating} class="w-full">
            {creating ? 'Creating...' : 'Create Session'}
          </Button>
        </form>
      </TabsContent>
    </Tabs>
  </DialogContent>
</Dialog>
