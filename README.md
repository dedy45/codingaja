# CodingAja — Claw Code Windows Fork

> **Modified Rust fork** of [instructkr/claw-code](https://github.com/instructkr/claw-code) with full Windows compatibility, multi-provider support (OpenAI, xAI, Ollama, custom), and UV-managed Python workspace.

---

## What's Different from Upstream

This is a **heavily modified fork** of the original Claw Code Rust port. The upstream project was written for Linux/macOS — this fork adds:

### 🪟 Windows Compatibility (20+ fixes)
- `bash` tool → uses `cmd /C` on Windows (was hardcoded `sh -lc`)
- `PowerShell` tool → fixed executable detection via `where` command
- OAuth → replaced `/dev/urandom` with cross-platform `getrandom` crate
- Config paths → fallback to `%USERPROFILE%` when `HOME` not set
- Browser open → fixed URL quoting for `cmd /C start`
- Skill/command discovery → searches `%USERPROFILE%\.claw\` and `%USERPROFILE%\.codex\`
- REPL runtime → added `cmd` and `powershell` language support
- Test suite → all Unix-only code (`chmod`, `PermissionsExt`) properly `#[cfg(unix)]`-gated
- `StructuredOutput` tool schema → fixed for OpenAI-compatible API validation

### 🤖 Multi-Provider Support
| Provider | Env Vars | Models |
|----------|----------|--------|
| **OpenAI** | `OPENAI_API_KEY` | gpt-4o, o1, o3, etc. |
| **xAI (Grok)** | `XAI_API_KEY` | grok-3, grok-mini |
| **Ollama (local)** | `OPENAI_API_KEY` + `OPENAI_BASE_URL` | llama3, qwen, etc. |
| **OpenRouter** | `OPENAI_API_KEY` + `OPENAI_BASE_URL` | any model |
| **LM Studio** | `OPENAI_API_KEY` + `OPENAI_BASE_URL` | local models |
| **Custom** | `OPENAI_API_KEY` + `OPENAI_BASE_URL` | any OpenAI-compatible |

Auto-detects provider based on model name and available env vars.

### 🐍 UV-Managed Python Workspace
- `pyproject.toml` with dev dependencies (pytest, ruff, mypy)
- All 22 tests passing
- Zero lint errors (ruff), zero type errors (mypy)
- 36/36 Python modules importable

---

## Quickstart

### 1. Build Rust Binary

```powershell
# Double-click or run:
.\build.bat

# Or manually:
cd rust
cargo build --release
```

Output: `rust\target\release\claw.exe`

### 2. Make `claw` Available Globally

The `claw.cmd` wrapper is already at `%USERPROFILE%\.local\bin\claw.cmd` (in PATH).

### 3. Configure

Edit `C:\Users\dedy\.claw\settings.json`:

```json
{
  "model": "gpt-4o",
  "permissions": {
    "defaultMode": "workspace-write"
  }
}
```

### 4. Set API Key

**Via GUI (permanent):**
1. Win+R → `sysdm.cpl` → Advanced → Environment Variables
2. User variables → New → Name: `OPENAI_API_KEY`, Value: `sk-xxx`

**Via PowerShell profile (per-session):**
```powershell
notepad $PROFILE
# Add: $env:OPENAI_API_KEY = "sk-xxx"
```

### 5. Run

```powershell
claw
# or
claw "hello world"
```

---

## Python Workspace (UV)

```powershell
# Install deps
uv sync --all-extras

# Run tests
uv run python -m pytest tests/ -v

# Lint
uv run ruff check src/ tests/

# Type check
uv run mypy src/ --ignore-missing-imports

# Run Python CLI
uv run python -m src.main summary
```

---

## Configuration

See [`KONFIGURASI.md`](KONFIGURASI.md) for complete config reference.

## Parity Work

If you want to continue the TypeScript-to-Rust parity work from `leak/src`, use these docs first:

- [`docs/PARITY_STATUS.md`](docs/PARITY_STATUS.md) — current parity truth for this workspace
- [`docs/PARITY_TOOL_MATRIX.md`](docs/PARITY_TOOL_MATRIX.md) — generated tool-by-tool implementation matrix
- [`docs/PARITY_IMPLEMENTATION_PATTERN.md`](docs/PARITY_IMPLEMENTATION_PATTERN.md) — exact TS -> Rust implementation pattern
- [`docs/PARITY_ROADMAP.md`](docs/PARITY_ROADMAP.md) — recommended lane order for completing parity

The generator for the tool matrix lives at:

- `docs/parity/generate_tool_parity_matrix.py`

Config file locations (priority low → high):
1. `%USERPROFILE%\.claw.json` — global user (legacy)
2. `%USERPROFILE%\.claw\settings.json` — global user
3. `<project>/.claw.json` — per-project
4. `<project>/.claw/settings.json` — per-project alt
5. `<project>/.claw/settings.local.json` — local override (gitignored)

---

## Repository Layout

```
.
├── src/                    # Python porting workspace (UV-managed)
├── rust/                   # Rust port (main binary)
│   ├── crates/api/         # API client + multi-provider
│   ├── crates/runtime/     # Session, tools, MCP, config
│   ├── crates/claw-cli/    # Interactive CLI binary
│   ├── crates/plugins/     # Plugin system
│   ├── crates/commands/    # Slash commands
│   ├── crates/server/      # HTTP/SSE server
│   ├── crates/lsp/         # LSP client
│   └── crates/tools/       # Tool specs
├── tests/                  # Python tests (22 passing)
├── pyproject.toml          # UV project config
├── build.bat               # Windows build script
├── KONFIGURASI.md          # Config reference (Indonesian)
└── PANDUAN.md              # Usage guide (Indonesian)
```

---

## Upstream

- Original: [instructkr/claw-code](https://github.com/instructkr/claw-code)
- This fork: [dedy45/codingaja](https://github.com/dedy45/codingaja)

---

## License

Same as upstream. This repository does not claim ownership of the original Claw Code source material.
