# Loom

A local web app that manages multiple isolated OpenCode sessions, each tied to a Jira issue, GitLab merge request, or created from scratch. Runs on a Hyper-V Windows VM, accessed via browser.

## Language

**Session**:
An isolated working environment tied to a specific task. Has a lifecycle: provisioning, running, stopped, terminated. Contains a git worktree and an exposed OpenCode web instance.
_Avoid_: workspace (overloaded), environment

**Session states**:
- **provisioning** — worktree being created, OpenCode starting up
- **running** — OpenCode process is live, user can work
- **stopped** — process not running (crash, reboot, or intentional stop), session still exists and can be resumed
- **terminated** — user explicitly ended the session, cleanup can proceed

**Source**:
The origin that triggers a Session: a Jira issue, a GitLab merge request, or "scratch" (manual/ad-hoc).
_Avoid_: trigger, origin

**Project**:
A git repository that Loom knows about. Has a cached bare clone used to create worktrees.
_Avoid_: repo (reserved for the raw git concept)

**Workspace**:
The Session's working directory — a git worktree checkout from the Project's bare clone.
_Avoid_: folder, directory

**Dashboard**:
The main UI. Lists active Sessions, allows creating new ones from Sources.

## Relationships

- A **Session** belongs to exactly one **Project**
- A **Session** is created from exactly one **Source** (Jira issue, GitLab MR, or scratch)
- A **Session** has exactly one **Workspace** (git worktree)
- A **Project** has one bare clone cache and zero or more active **Sessions**

## Example dialogue

> **Dev:** "I want to work on PROJ-123. What happens?"
> **Domain expert:** "Loom creates a **Session** from that Jira **Source**. It fetches the **Project**'s bare clone, creates a **Workspace** via git worktree, and exposes an OpenCode web instance."

> **Dev:** "Can I have two Sessions on the same Project?"
> **Domain expert:** "Yes — each Session has its own **Workspace** (worktree), so they're fully isolated."

## Flagged ambiguities

- None yet.
