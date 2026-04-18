<script lang="ts">
  import { Card, CardHeader, CardTitle, CardContent } from '$lib/components/ui/card/index.js';
  import { Separator } from '$lib/components/ui/separator/index.js';
  import { get } from '$lib/api/client';
  import type { HealthResponse } from '$shared/HealthResponse';

  let health = $state<HealthResponse | null>(null);

  async function fetchHealth() {
    try {
      health = await get<HealthResponse>('/health');
    } catch {
      // ignore
    }
  }
  fetchHealth();

  const envVars = [
    { name: 'JIRA_BASE_URL', desc: 'Jira instance URL' },
    { name: 'JIRA_EMAIL', desc: 'Jira authentication email' },
    { name: 'JIRA_API_TOKEN', desc: 'Jira API token' },
    { name: 'GITLAB_URL', desc: 'GitLab instance URL' },
    { name: 'GITLAB_TOKEN', desc: 'GitLab personal access token' },
    { name: 'OPENCODE_PATH', desc: 'Path to OpenCode binary' },
  ];
</script>

<div class="space-y-6">
  <h1 class="text-2xl font-bold">Settings</h1>

  <Card>
    <CardHeader>
      <CardTitle>Server Status</CardTitle>
    </CardHeader>
    <CardContent class="text-sm">
      {#if health}
        <p>Status: <span class="font-medium text-green-600">{health.status}</span></p>
        <p>Version: <span class="font-mono">{health.version}</span></p>
      {:else}
        <p class="text-muted-foreground">Unable to reach server.</p>
      {/if}
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Environment Configuration</CardTitle>
    </CardHeader>
    <CardContent>
      <p class="mb-4 text-sm text-muted-foreground">
        These variables are configured on the server. Status is not available from the frontend.
      </p>
      <div class="space-y-3">
        {#each envVars as v}
          <div class="flex items-center justify-between text-sm">
            <div>
              <p class="font-mono font-medium">{v.name}</p>
              <p class="text-muted-foreground">{v.desc}</p>
            </div>
          </div>
          {#if v !== envVars[envVars.length - 1]}
            <Separator />
          {/if}
        {/each}
      </div>
    </CardContent>
  </Card>

  <Card>
    <CardHeader>
      <CardTitle>Paths</CardTitle>
    </CardHeader>
    <CardContent class="space-y-2 text-sm">
      <div>
        <p class="font-medium">Bare clone cache</p>
        <p class="font-mono text-muted-foreground">~/.loom/cache/</p>
      </div>
      <Separator />
      <div>
        <p class="font-medium">Worktrees</p>
        <p class="font-mono text-muted-foreground">~/.loom/worktrees/</p>
      </div>
      <Separator />
      <div>
        <p class="font-medium">Base prompt file</p>
        <p class="font-mono text-muted-foreground">~/.loom/base-prompt.md</p>
      </div>
    </CardContent>
  </Card>
</div>
