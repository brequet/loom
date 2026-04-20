<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '$lib/components/ui/dialog/index.js';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { Label } from '$lib/components/ui/label/index.js';
  import { Select, SelectContent, SelectItem, SelectTrigger } from '$lib/components/ui/select/index.js';
  import { getJiraIssue } from '$lib/api/jira';
  import { createSession } from '$lib/api/sessions';
  import { push } from 'svelte-spa-router';

  let { open = $bindable(false) }: { open?: boolean } = $props();

  const MODELS = [
    { value: 'github-copilot/gpt-5-mini', label: 'GPT-5 Mini' },
    { value: 'github-copilot/claude-haiku-4.5', label: 'Claude Haiku 4.5' },
    { value: 'github-copilot/claude-sonnet-4.6', label: 'Claude Sonnet 4.6' },
    { value: 'github-copilot/claude-opus-4.6', label: 'Claude Opus 4.6' },
  ] as const;

  let scratchTitle = $state('');
  let gitlabUrl = $state('');
  let jiraInput = $state('');
  let selectedModel = $state<string>(MODELS[0].value);
  let customInstructions = $state('');
  let creating = $state(false);
  let gitlabError = $state('');
  let jiraError = $state('');

  function selectedModelLabel(): string {
    return MODELS.find((m) => m.value === selectedModel)?.label ?? selectedModel;
  }

  function parseJiraKey(input: string): string | null {
    const trimmed = input.trim();
    const keyMatch = trimmed.match(/^([A-Z]+-\d+)$/i);
    if (keyMatch) return keyMatch[1].toUpperCase();

    const urlMatch = trimmed.match(/browse\/([A-Z]+-\d+)/i);
    if (urlMatch) return urlMatch[1].toUpperCase();

    return null;
  }

  async function handleCreateFromJira() {
    if (creating) return;
    const key = parseJiraKey(jiraInput);
    if (!key) {
      jiraError = 'Enter a valid issue key (e.g. SAM-398) or full Jira URL.';
      return;
    }

    jiraError = '';
    creating = true;
    try {
      const issue = await getJiraIssue(key);
      const session = await createSession({
        source_type: 'jira',
        source_ref: issue.key,
        title: `${issue.key}: ${issue.summary}`,
        model: selectedModel,
        custom_instructions: customInstructions.trim() || undefined,
      });
      open = false;
      jiraInput = '';
      push(`/sessions/${session.id}?autoOpen=1`);
    } catch (e) {
      jiraError = e instanceof Error ? e.message : 'Failed to fetch Jira issue';
    } finally {
      creating = false;
    }
  }

  async function handleCreateFromGitLabUrl() {
    if (creating) return;
    const url = gitlabUrl.trim();
    if (!url) return;

    const mrMatch = url.match(/merge_requests\/(\d+)/);
    if (!mrMatch) {
      gitlabError = 'Enter a valid GitLab merge request URL.';
      return;
    }

    gitlabError = '';
    creating = true;
    try {
      const session = await createSession({
        source_type: 'gitlab',
        source_ref: url,
        title: `MR !${mrMatch[1]}`,
        model: selectedModel,
        custom_instructions: customInstructions.trim() || undefined,
      });
      open = false;
      gitlabUrl = '';
      push(`/sessions/${session.id}?autoOpen=1`);
    } finally {
      creating = false;
    }
  }

  async function handleCreateScratch() {
    if (creating) return;
    creating = true;
    try {
      const session = await createSession({
        source_type: 'scratch',
        title: scratchTitle.trim() || 'Scratch session',
        model: selectedModel,
        custom_instructions: customInstructions.trim() || undefined,
      });
      open = false;
      scratchTitle = '';
      push(`/sessions/${session.id}?autoOpen=1`);
    } finally {
      creating = false;
    }
  }
</script>

<Dialog bind:open>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>New Session</DialogTitle>
      <DialogDescription>Create a coding session from an issue tracker or start fresh.</DialogDescription>
    </DialogHeader>

    <div class="space-y-2">
      <Label for="model-select">Model</Label>
      <Select type="single" bind:value={selectedModel}>
        <SelectTrigger id="model-select">{selectedModelLabel()}</SelectTrigger>
        <SelectContent>
          {#each MODELS as model}
            <SelectItem value={model.value}>{model.label}</SelectItem>
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
        class="flex w-full rounded-md border border-input bg-background px-3 py-2 text-sm ring-offset-background placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-50 resize-y"
      ></textarea>
    </div>

    <Tabs value="jira">
      <TabsList class="w-full">
        <TabsTrigger value="jira" class="flex-1">Jira</TabsTrigger>
        <TabsTrigger value="gitlab" class="flex-1">GitLab</TabsTrigger>
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
          </div>
          <Button type="submit" disabled={creating} class="w-full">
            {creating ? 'Creating...' : 'Create Session'}
          </Button>
        </form>
      </TabsContent>
    </Tabs>
  </DialogContent>
</Dialog>
