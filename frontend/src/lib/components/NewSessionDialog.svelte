<script lang="ts">
  import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogDescription } from '$lib/components/ui/dialog/index.js';
  import { Tabs, TabsContent, TabsList, TabsTrigger } from '$lib/components/ui/tabs/index.js';
  import { Button } from '$lib/components/ui/button/index.js';
  import { Input } from '$lib/components/ui/input/index.js';
  import SourceSearch from './SourceSearch.svelte';
  import { searchJiraIssues, getJiraIssue, type JiraIssue } from '$lib/api/jira';
  import { searchGitLabMRs, type GitLabMR } from '$lib/api/gitlab';
  import { createSession } from '$lib/api/sessions';
  import { push } from 'svelte-spa-router';

  let { open = $bindable(false) }: { open?: boolean } = $props();

  let scratchTitle = $state('');
  let creating = $state(false);

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

  async function handleCreateFromGitLab(mr: GitLabMR) {
    if (creating) return;
    creating = true;
    try {
      const session = await createSession({
        source_type: 'gitlab',
        source_ref: `!${mr.iid}`,
        title: mr.title,
      });
      open = false;
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
    // Detect JIRA key pattern or URL
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

  async function handleGitLabSearch(q: string): Promise<GitLabMR[]> {
    // Detect !42 pattern
    const mrMatch = q.match(/^!(\d+)$/);
    if (mrMatch) {
      return searchGitLabMRs(mrMatch[1]);
    }
    // Detect GitLab MR URL
    const urlMatch = q.match(/merge_requests\/(\d+)/);
    if (urlMatch) {
      return searchGitLabMRs(urlMatch[1]);
    }
    return searchGitLabMRs(q);
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
    {#snippet gitlabItem(mr: GitLabMR)}
      <div>
        <span class="font-mono font-medium">!{mr.iid}</span>
        <span class="ml-2 text-muted-foreground">{mr.title}</span>
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
      <TabsContent value="gitlab" class="mt-4 space-y-2">
        <p class="text-sm text-muted-foreground">Search for a GitLab merge request or paste !ID/URL.</p>
        <SourceSearch
          placeholder="Search MRs (e.g. !42)..."
          onSearch={handleGitLabSearch}
          onSelect={handleCreateFromGitLab}
          renderItem={gitlabItem}
        />
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
