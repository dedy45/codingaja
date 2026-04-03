# Production Parity Execution Plan

Last updated: 2026-03-31
Scope: `rust/` runtime + `src/` Python workspace + parity workflow integration.
Policy: **production-only** validation (no mocks, no synthetic fixtures, no demo-only paths).

---

## 1) Objective

Bring this repository to production-grade parity behavior with the parity reference branch while preserving cross-platform stability and deterministic runtime behavior.

Success means:
- core CLI and runtime paths run end-to-end in production mode,
- tools/commands/hooks/plugins are wired through real execution paths,
- structured output and session state are reliable,
- verification gates pass on Windows and Linux.

---

## 2) Non-negotiable quality gates

### Rust gates (from `rust/`)
1. `cargo fmt --all`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test --workspace`

### Python gates (from repo root)
1. `ruff check src tests`
2. `mypy src`
3. `pytest -q`

### Live production gates
1. Interactive REPL path works with real auth/provider.
2. One-shot prompt mode works with real auth/provider.
3. Tool-calling works with real tool execution.
4. Session persistence/load/resume works on real filesystem.
5. Structured output mode emits valid JSON without preamble/noise.

No merge if any gate fails.

---

## 3) Domain matrix (implementation order)

## A. Runtime core (P0)
Target:
- single-turn and multi-turn loop correctness,
- stop reasons and usage accounting correctness,
- stream event correctness,
- no duplicated turn submission side effects.

Implementation files:
- `rust/crates/runtime/src/conversation.rs`
- `rust/crates/claw-cli/src/main.rs`
- `src/query_engine.py`
- `src/runtime.py`

Acceptance checks:
- one logical turn increments transcript once,
- stream event stop usage matches final turn usage,
- max_turn / max_budget stopping behavior is deterministic.

## B. Hooks runtime execution (P0)
Target:
- PreToolUse and PostToolUse execution in live runtime,
- deny/rewrite/post-process behavior.

Implementation files:
- `rust/crates/runtime/src/conversation.rs`
- `rust/crates/runtime/src/config.rs`
- `rust/crates/commands/src/lib.rs` (inspection/report commands)

Acceptance checks:
- hook policy actually changes runtime behavior,
- deny path blocks tool execution with clear message,
- post-hook transforms are reflected in final output.

## C. Plugins lifecycle (P0)
Target:
- install/load/enable/disable/reload/uninstall lifecycle production-ready.

Implementation files:
- `rust/crates/plugins/**`
- `rust/crates/claw-cli/src/main.rs`
- `rust/crates/commands/src/lib.rs`

Acceptance checks:
- plugin state survives restart,
- disabled plugin cannot execute hooks/tools,
- reload is safe and atomic.

## D. Tool parity surface (P0/P1)
Target:
- critical missing tools implemented and integrated in normal runtime path.

Implementation files:
- `rust/crates/tools/src/lib.rs`
- tool-specific runtime integration surfaces

Acceptance checks:
- each tool runs against real environment/resources,
- permission checks enforced,
- failure mode messages are actionable.

## E. Command parity surface (P1)
Target:
- major command families available and usable.

Implementation files:
- `rust/crates/commands/src/lib.rs`
- `rust/crates/claw-cli/src/main.rs`

Acceptance checks:
- commands registered, discoverable, and executable,
- command outputs consistent across REPL and prompt modes.

## F. Services/provider/auth robustness (P1)
Target:
- provider routing by model,
- OAuth and API-key flows hardened,
- stable retries/error mapping.

Implementation files:
- `rust/crates/api/**`
- `rust/crates/runtime/src/oauth.rs`
- `rust/crates/runtime/src/config.rs`

Acceptance checks:
- login/logout/refresh work in live mode,
- provider switch does not break tool-capable flow,
- no credential corruption on missing/invalid files.

## G. Cross-platform behavior (P0)
Target:
- Windows and Linux parity for shell invocation, config dirs, process openers.

Implementation files:
- `rust/crates/runtime/src/bash.rs`
- `rust/crates/runtime/src/config.rs`
- `rust/crates/runtime/src/oauth.rs`
- `rust/crates/tools/src/lib.rs`
- `rust/crates/claw-cli/src/main.rs`

Acceptance checks:
- command execution path works on both OS,
- config/credentials resolve correctly from HOME/USERPROFILE,
- browser open behavior robust with quoted URLs on Windows.

## H. Python workspace integrity (P1)
Target:
- Python support workspace remains internally consistent and testable.

Implementation files:
- `src/main.py`
- `src/runtime.py`
- `src/query_engine.py`
- `src/parity_audit.py`
- `tests/test_porting_workspace.py`

Acceptance checks:
- CLI commands run and return expected outputs,
- session/token accounting and stream metadata are consistent,
- parity audit parsing robust to malformed reference values.

---

## 4) Implementation protocol

For each domain:
1. Read current implementation and parity target.
2. Apply smallest safe change set.
3. Run all relevant gates.
4. Record behavior changes in docs.
5. Move to next domain only when current domain passes all gates.

---

## 5) Release readiness checklist

- [ ] Rust fmt/clippy/tests all pass.
- [ ] Python ruff/mypy/pytest all pass.
- [ ] Live auth flows verified (API key + OAuth).
- [ ] Tools and hooks verified in live path.
- [ ] Plugin lifecycle verified in live path.
- [ ] Structured output validated (strict JSON).
- [ ] Session persistence/resume validated.
- [ ] Windows and Linux production verification completed.
- [ ] Documentation updated to match actual behavior.

---

## 6) Current immediate work in this workspace

1. Fix Python runtime turn duplication path in `src/runtime.py`.
2. Remove Python type-ignore debt in `src/query_engine.py` and `src/parity_audit.py`.
3. Continue domain-by-domain implementation and validation until all gates pass.
