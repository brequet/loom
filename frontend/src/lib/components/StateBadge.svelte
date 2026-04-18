<script lang="ts">
  import type { SessionState } from '$shared/SessionState';
  import { Badge } from '$lib/components/ui/badge/index.js';

  let { state }: { state: SessionState } = $props();

  const config: Record<SessionState, { label: string; class: string }> = {
    provisioning: { label: 'Provisioning', class: 'bg-amber-500/15 text-amber-600 border-amber-500/30 hover:bg-amber-500/20' },
    running: { label: 'Running', class: 'bg-green-500/15 text-green-600 border-green-500/30 hover:bg-green-500/20' },
    stopped: { label: 'Stopped', class: 'bg-red-500/15 text-red-600 border-red-500/30 hover:bg-red-500/20' },
    terminated: { label: 'Terminated', class: 'bg-gray-500/15 text-gray-500 border-gray-500/30 hover:bg-gray-500/20' },
  };
</script>

<Badge variant="outline" class={config[state].class}>
  {config[state].label}
</Badge>
