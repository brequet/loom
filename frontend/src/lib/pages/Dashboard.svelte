<script lang="ts">
    import type { Session } from "$shared/Session";
    import type { SessionState } from "$shared/SessionState";
    import { listSessions, stopSession, resumeSession, terminateSession } from "$lib/api/sessions";
    import { createQuery, createMutation, useQueryClient } from "@tanstack/svelte-query";
    import { Button } from "$lib/components/ui/button/index.js";
    import {
        Table,
        TableBody,
        TableCell,
        TableHead,
        TableHeader,
        TableRow,
    } from "$lib/components/ui/table/index.js";
    import {
        DropdownMenu,
        DropdownMenuContent,
        DropdownMenuItem,
        DropdownMenuSeparator,
        DropdownMenuTrigger,
    } from "$lib/components/ui/dropdown-menu/index.js";
    import StateBadge from "$lib/components/StateBadge.svelte";
    import NewSessionDialog from "$lib/components/NewSessionDialog.svelte";
    import { push } from "svelte-spa-router";

    const queryClient = useQueryClient();

    let dialogOpen = $state(false);
    let showTerminated = $state(false);

    const sessionsQuery = createQuery(() => ({
        queryKey: ['sessions', { showTerminated }],
        queryFn: async () => {
            const data = await listSessions(showTerminated);
            return data.sessions;
        },
        refetchInterval: 5_000,
    }));

    const stateOrder: Record<SessionState, number> = {
        running: 0,
        provisioning: 1,
        stopped: 2,
        terminated: 3,
    };

    let sorted = $derived(
        [...(sessionsQuery.data ?? [])].sort((a, b) => stateOrder[a.state] - stateOrder[b.state]),
    );

    function formatDate(iso: string): string {
        return new Date(iso).toLocaleString(undefined, {
            month: "short",
            day: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    }

    function getOpenCodeUrl(s: Session): string | null {
        if (!s.opencode_port) return null;
        const base = `http://${window.location.hostname}:${s.opencode_port}`;
        if (s.opencode_path_prefix && s.opencode_session_id) {
            return `${base}/${s.opencode_path_prefix}/session/${s.opencode_session_id}`;
        }
        if (s.opencode_session_id) {
            return `${base}/session/${s.opencode_session_id}`;
        }
        return base;
    }

    const stopMutation = createMutation(() => ({
        mutationFn: (id: string) => stopSession(id),
        onSettled: () => queryClient.invalidateQueries({ queryKey: ['sessions'] }),
    }));

    const resumeMutation = createMutation(() => ({
        mutationFn: (id: string) => resumeSession(id),
        onSettled: () => queryClient.invalidateQueries({ queryKey: ['sessions'] }),
    }));

    const terminateMutation = createMutation(() => ({
        mutationFn: (id: string) => terminateSession(id),
        onSettled: () => queryClient.invalidateQueries({ queryKey: ['sessions'] }),
    }));

    function handleTerminate(id: string) {
        if (!confirm('Terminate this session? The workspace will be permanently deleted.')) return;
        terminateMutation.mutate(id);
    }
</script>

<svelte:head>
    <title>Loom</title>
</svelte:head>

<div class="space-y-6">
    <div class="flex items-center justify-between">
        <h1 class="text-2xl font-bold tracking-tight">Sessions</h1>
        <div class="flex items-center gap-2">
            <Button variant="ghost" size="sm" onclick={() => (showTerminated = !showTerminated)} class="text-muted-foreground">
                {showTerminated ? 'Hide terminated' : 'Show terminated'}
            </Button>
            <Button onclick={() => (dialogOpen = true)}>New Session</Button>
        </div>
    </div>

    {#if sessionsQuery.isPending}
        <p class="text-muted-foreground">Loading sessions...</p>
    {:else if sessionsQuery.isError}
        <p class="text-destructive">{sessionsQuery.error.message}</p>
    {:else if sorted.length === 0}
        <div
            class="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed p-12 text-center"
        >
            <p class="text-muted-foreground">No sessions yet.</p>
            <Button variant="outline" onclick={() => (dialogOpen = true)}
                >Create your first session</Button
            >
        </div>
    {:else}
        <div class="rounded-md border">
            <Table>
                <TableHeader>
                    <TableRow>
                        <TableHead>Title</TableHead>
                        <TableHead>Source</TableHead>
                        <TableHead>State</TableHead>
                        <TableHead>Created</TableHead>
                        <TableHead class="w-15"></TableHead>
                    </TableRow>
                </TableHeader>
                <TableBody>
                    {#each sorted as session (session.id)}
                        <TableRow
                            class="cursor-pointer"
                            onclick={() => push(`/sessions/${session.id}`)}
                        >
                            <TableCell class="font-medium max-w-80 truncate"
                                >{session.title}</TableCell
                            >
                            <TableCell class="capitalize text-muted-foreground"
                                >{session.source_type}</TableCell
                            >
                            <TableCell
                                ><StateBadge state={session.state} /></TableCell
                            >
                            <TableCell
                                class="text-muted-foreground text-xs whitespace-nowrap"
                                >{formatDate(session.created_at)}</TableCell
                            >
                            <TableCell>
                                <!-- svelte-ignore a11y_click_events_have_key_events -->
                                <div
                                    role="button"
                                    tabindex="-1"
                                    onclick={(e) => e.stopPropagation()}
                                    onkeydown={(e) => {
                                        if (e.key === "Enter")
                                            e.stopPropagation();
                                    }}
                                >
                                    <DropdownMenu>
                                        <DropdownMenuTrigger>
                                            <Button
                                                variant="ghost"
                                                size="icon"
                                                class="h-8 w-8"
                                            >
                                                <svg
                                                    xmlns="http://www.w3.org/2000/svg"
                                                    width="16"
                                                    height="16"
                                                    viewBox="0 0 24 24"
                                                    fill="none"
                                                    stroke="currentColor"
                                                    stroke-width="2"
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    ><circle
                                                        cx="12"
                                                        cy="12"
                                                        r="1"
                                                    /><circle
                                                        cx="12"
                                                        cy="5"
                                                        r="1"
                                                    /><circle
                                                        cx="12"
                                                        cy="19"
                                                        r="1"
                                                    /></svg
                                                >
                                            </Button>
                                        </DropdownMenuTrigger>
                                        <DropdownMenuContent align="end">
                                            <DropdownMenuItem
                                                onclick={() =>
                                                    push(
                                                        `/sessions/${session.id}`,
                                                    )}
                                            >
                                                View details
                                            </DropdownMenuItem>
                                            {#if session.state === "running" && getOpenCodeUrl(session)}
                                                <DropdownMenuItem
                                                    onclick={() =>
                                                        window.open(
                                                            getOpenCodeUrl(
                                                                session,
                                                            )!,
                                                            "_blank",
                                                        )}
                                                >
                                                    Open OpenCode ↗
                                                </DropdownMenuItem>
                                            {/if}
                                            {#if session.state === 'running' || session.state === 'stopped'}
                                                <DropdownMenuSeparator />
                                            {/if}
                                            {#if session.state === 'running'}
                                                <DropdownMenuItem onclick={() => stopMutation.mutate(session.id)}>
                                                    Stop
                                                </DropdownMenuItem>
                                            {:else if session.state === 'stopped'}
                                                <DropdownMenuItem onclick={() => resumeMutation.mutate(session.id)}>
                                                    Resume
                                                </DropdownMenuItem>
                                                <DropdownMenuItem class="text-destructive" onclick={() => handleTerminate(session.id)}>
                                                    Terminate
                                                </DropdownMenuItem>
                                            {/if}
                                        </DropdownMenuContent>
                                    </DropdownMenu>
                                </div>
                            </TableCell>
                        </TableRow>
                    {/each}
                </TableBody>
            </Table>
        </div>
    {/if}
</div>

<NewSessionDialog bind:open={dialogOpen} />
