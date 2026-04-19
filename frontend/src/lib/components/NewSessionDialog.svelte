<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '$lib/components/ui/dialog/index.js';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import { getJiraIssue } from '$lib/api/jira';
  import { createSession } from '$lib/api/sessions';
  import { push } from 'svelte-spa-router';

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let scratchTitle = $state('');
  let gitlabUrl = $state('');
  let jiraInput = $state('');
  let creating = $state(false);
  let gitlabError = $state('');
  let jiraError = $state('');

  /**
   * Parse a Jira issue key from user input.
   * Accepts either a bare key (e.g. "SAM-398") or a full Jira URL
   * (e.g. "https://team.atlassian.net/browse/SAM-398").
   */
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
      jiraError = 'Enter a Jira issue key (e.g. SAM-398) or full URL (e.g. https://team.atlassian.net/browse/SAM-398)';
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
      gitlabError = 'Invalid GitLab MR URL. Expected format: https://gitlab.com/.../merge_requests/42';
      return;
    }

    gitlabError = '';
    creating = true;
    try {
      const session = await createSession({
        source_type: 'gitlab',
        source_ref: url,
        title: `MR !${mrMatch[1]}`,
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
      <DialogDescription>Create a session from a Jira issue, GitLab MR, or start from scratch.</DialogDescription>
    </DialogHeader>
    <Tabs value="jira">
      <TabsList class="w-full">
        <TabsTrigger value="jira" class="flex-1">Jira</TabsTrigger>
        <TabsTrigger value="gitlab" class="flex-1">GitLab</TabsTrigger>
        <TabsTrigger value="scratch" class="flex-1">Scratch</TabsTrigger>
      </TabsList>
      <TabsContent value="jira" class="mt-4 space-y-4">
        <p class="text-sm text-muted-foreground">Enter a Jira issue key or paste a full URL.</p>
        <Input
          placeholder="SAM-398 or https://team.atlassian.net/browse/SAM-398"
          bind:value={jiraInput}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') handleCreateFromJira(); }}
        />
        {#if jiraError}
          <p class="text-sm text-destructive">{jiraError}</p>
        {/if}
        <Button onclick={handleCreateFromJira} disabled={creating || !jiraInput.trim()} class="w-full">
          {creating ? 'Creating...' : 'Create Session'}
        </Button>
      </TabsContent>
      <TabsContent value="gitlab" class="mt-4 space-y-4">
        <p class="text-sm text-muted-foreground">Paste a GitLab merge request URL.</p>
        <Input
          placeholder="https://gitlab.com/.../merge_requests/42"
          bind:value={gitlabUrl}
          onkeydown={(e: KeyboardEvent) => { if (e.key === 'Enter') handleCreateFromGitLabUrl(); }}
        />
        {#if gitlabError}
          <p class="text-sm text-destructive">{gitlabError}</p>
        {/if}
        <Button onclick={handleCreateFromGitLabUrl} disabled={creating || !gitlabUrl.trim()} class="w-full">
          {creating ? 'Creating...' : 'Create Session'}
        </Button>
      </TabsContent>
      <TabsContent value="scratch" class="mt-4 space-y-4">
        <p class="text-sm text-muted-foreground">Create a scratch session with an optional title.</p>
        <Input placeholder="Session title (optional)" bind:value={scratchTitle} />
        <Button onclick={handleCreateScratch} disabled={creating} class="w-full">
          {creating ? 'Creating...' : 'Create Session'}
        </Button>
      </TabsContent>
    </Tabs>
  </DialogContent>
</Dialog>
