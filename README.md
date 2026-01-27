# jean-pierre-code

Agentic AI CLI code assistant (no TUI). Uses a local lightweight planner (feature-gated Candle integration planned) to gather context and propose actions, while delegating heavyweight reasoning to a remote model endpoint you control. Classic CLI UX.

Current status: early scaffold (MVP). Compiles without any model weights or remote endpoint. You can:
- Initialize config: `jpc init`
- Inspect project context: `jpc context`
- Produce a simple heuristic plan: `jpc plan "update the README with setup steps"`
- Apply a plan JSON: `jpc apply plan.json`
- Chat using a remote endpoint (or echo fallback): `jpc chat "How to refactor main?"`
- Run helpers: `jpc run build`, `jpc run test`, `jpc run <cmd>`

Install
1. Ensure Rust toolchain installed (stable).
2. Build:
   - Default (no local LLM weights needed):
     ```bash
     cargo build --release
     ```
   - With experimental local LLM feature (Candle stub; you must add actual model code later):
     ```bash
     cargo build --release --features local-llm
     ```

Usage
```bash
jean-pierre-code init [--force]
jean-pierre-code context [--max-files N] [--max-bytes BYTES]
jean-pierre-code plan <query> [--max-tokens N]
jean-pierre-code apply <plan_file> [--dry-run]
jean-pierre-code chat <prompt>
jean-pierre-code run <build|test|PROGRAM>
```

Configuration
- Files searched in order (later overrides earlier):
  - XDG config dir (e.g., `~/.config/jean-pierre-code/config.toml`)
  - Local project: `./.jpc/config.toml`
- Env vars override config file values:
  - `JPC_REMOTE_ENDPOINT` – HTTP endpoint for remote model
  - `JPC_API_KEY` – Bearer token for remote HTTP
  - `JPC_MODEL` – Model name sent to the remote
  - `JPC_PROJECT_ROOT` – Project root path

Remote model API
The tool sends a POST request to `JPC_REMOTE_ENDPOINT` with JSON body:
```json
{ "model": "<string>", "prompt": "<string>", "max_tokens": 2048 }
```
Expected response is either:
- `{ "output": "<string>" }`, or
- a plain text body with the output.

Local planner
- Default: a heuristic planner that makes simple plans (e.g., editing README).
- Future: enable `local-llm` feature to integrate Candle (HuggingFace candle) for local context summarization and planning.

Context gathering
- Gathers `Cargo.toml`, `Cargo.lock`, `src/` (and `tests/` if present), with size limits.
- Git info (branch/status) included if available.

Safety
- Applying a plan that writes to an existing file creates a timestamped backup under `./.jpc/backups/`.

Roadmap
- Add Candle-backed tiny LLM for summarization/planning behind `local-llm` feature.
- Expand action set (patch/hunk edits, multi-file diffs, run sequences).
- Streaming remote responses.
- Better prompting & context windows.