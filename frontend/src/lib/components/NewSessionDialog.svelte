<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '$lib/components/ui/dialog/index.js';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import SourceSearch from './SourceSearch.svelte';
  import { searchJiraIssues, getJiraIssue, type JiraIssue } from '$lib/api/jira';
  import { createSession } from '$lib/api/sessions';
  import { push } from 'svelte-spa-router';

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let scratchTitle = $state('');
  let gitlabUrl = $state('');
  let creating = $state(false);
  let gitlabError = $state('');

  async function handleCreateFromJira(issue: JiraIssue) {
    if (creating) return;
    creating = true;
    try {
      const session = await createSession({
        source_type: 'jira',
        source_ref: issue.key,
        title: `${issue.key}: ${issue.summary}`,
      });
      open = false;
      push(`/sessions/${session.id}`);
    } finally {
      creating = false;
    }
  }

  async function handleCreateFromGitLabUrl() {
    if (creating) return;
    const url = gitlabUrl.trim();
    if (!url) return;

    // Extract MR info from URL: https://gitlab.com/group/project/-/merge_requests/42
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
      push(`/sessions/${session.id}`);
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
      push(`/sessions/${session.id}`);
    } finally {
      creating = false;
    }
  }

  async function handleJiraSearch(q: string): Promise<JiraIssue[]> {
    const keyMatch = q.match(/^([A-Z]+-\d+)$/);
    if (keyMatch) {
      try {
        const issue = await getJiraIssue(keyMatch[1]);
        return [issue];
      } catch {
        return [];
      }
    }
    const urlMatch = q.match(/browse\/([A-Z]+-\d+)/);
    if (urlMatch) {
      try {
        const issue = await getJiraIssue(urlMatch[1]);
        return [issue];
      } catch {
        return [];
      }
    }
    return searchJiraIssues(q);
  }
</script>

<Dialog bind:open>
  <DialogContent class="sm:max-w-lg">
    <DialogHeader>
      <DialogTitle>New Session</DialogTitle>
      <DialogDescription>Create a session from a Jira issue, GitLab MR, or start from scratch.</DialogDescription>
    </DialogHeader>
    {#snippet jiraItem(issue: JiraIssue)}
      <div>
        <span class="font-mono font-medium">{issue.key}</span>
        <span class="ml-2 text-muted-foreground">{issue.summary}</span>
      </div>
    {/snippet}
    <Tabs value="jira">
      <TabsList class="w-full">
        <TabsTrigger value="jira" class="flex-1">Jira</TabsTrigger>
        <TabsTrigger value="gitlab" class="flex-1">GitLab</TabsTrigger>
        <TabsTrigger value="scratch" class="flex-1">Scratch</TabsTrigger>
      </TabsList>
      <TabsContent value="jira" class="mt-4 space-y-2">
        <p class="text-sm text-muted-foreground">Search for a Jira issue or paste a key/URL.</p>
        <SourceSearch
          placeholder="Search issues (e.g. PROJ-123)..."
          onSearch={handleJiraSearch}
          onSelect={handleCreateFromJira}
          renderItem={jiraItem}
        />
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
