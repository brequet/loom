# OpenCode integration: one server per session, controlled via REST API

Each Loom Session spawns its own `opencode web` process bound to the session's Workspace directory on a unique port. Loom's Rust backend uses OpenCode's REST API (via reqwest) to create sessions and inject initial prompts with task context. The user opens the OpenCode web UI directly at the session's port.

**Considered alternatives:**
- Single shared OpenCode instance for all sessions — rejected because `opencode web` binds to a single working directory. Multiple sessions need isolated file systems.
- Frontend talking directly to OpenCode instances — rejected due to CORS complexity and loss of centralized control. Loom backend proxies all OpenCode API calls.
- Pre-configuring git repos before OpenCode starts — rejected. OpenCode can start in an empty directory and handle git setup itself via system prompt instructions, which is simpler and more flexible.
