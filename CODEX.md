# Codex Entrypoint

Read and follow `AGENTS.md` — it is the canonical project documentation.

## Codex-Specific Notes

- Deployment context and verified facts live in `kb/Building/Deployment.md`.
- Wolfgang (`~/dev/work/wolfgang`) is the reference implementation for Firebase auth, Cloud Run, and Terraform patterns. When making deployment decisions, align with Wolfgang's conventions.
- Runtime feature defaults for deployment scripts: `gcp,auth,firebase`.
- Run `just lint` before considering work done.
