# Claude Code Entrypoint

Read and follow `AGENTS.md` — it is the canonical project documentation.

## Claude-Specific Notes

- Use `architecture/ARCHITECTURE.md` and `architecture/API_SURFACES.md` as the authoritative API reference. When they conflict with other docs, the architecture docs win.
- Prefer Edit over Write for existing files. Prefer Grep/Glob over Bash for search.
- Do not create documentation files unless explicitly asked. Knowledge belongs in `kb/`.
- When learning something about the project, update the relevant `kb/` page rather than saving it as memory.
- Run `just lint` before considering work done.
- Never push to main without confirmation.
