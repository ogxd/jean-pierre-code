# jean-pierre-code

Agentic AI CLI code assistant (no TUI). Uses a local planner powered by Kalosm (always on) to gather context and propose actions, while optionally delegating heavyweight reasoning to a remote model endpoint you control. Classic CLI UX.

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
   - Default:
     ```bash
     cargo build --release
     ```
   Notes:
   - Local inference uses [Kalosm](https://github.com/floneum/floneum/tree/main/interfaces/kalosm) and will initialize a tiny Llama model on first use. No separate runtime (like Ollama) is required.
   - If you build from a clean environment, Cargo will fetch Kalosm from Git. The `full` feature set may require a recent Rust toolchain. If you encounter a compiler version error, run:
     ```bash
     rustup update stable
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
  - `JPC_MODEL` – Model name (used by the remote endpoint; local Kalosm planner currently uses its default embedded model).
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
- Always-on local LLM via Kalosm (Llama). No external runtime required.
- The planner prompts the model to output a strict JSON plan. If parsing fails or inference errors occur, it gracefully falls back to a simple heuristic plan.

Context gathering
- Gathers `Cargo.toml`, `Cargo.lock`, `src/` (and `tests/` if present), with size limits.
- Git info (branch/status) included if available.

Safety
- Applying a plan that writes to an existing file creates a timestamped backup under `./.jpc/backups/`.

Roadmap
- Optional: add model selection and streaming UX for Kalosm planner.
- Expand action set (patch/hunk edits, multi-file diffs, run sequences).
- Streaming remote responses.
- Better prompting & context windows.