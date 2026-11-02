# Contributing to Myelin

Welcome! Myelin is building the execution substrate for the autonomous software era.

## Development Workflow

1. Install Rust (`stable` toolchain via [rustup.rs](https://rustup.rs)).
2. Clone the repository and verify tests pass:
   ```bash
   cargo test --workspace
   cargo clippy --workspace -- -D warnings
   cargo fmt --check
   ```
3. Keep `SKILL.md` in sync with any grammar or stdlib change — **if an LLM cannot learn a feature from `SKILL.md` in a few lines, the feature is too complex.**

## Core Design Tenets for Contributors
- **Zero Human Syntactic Sugar:** Never add aliases or two ways to write the same construct. Canonical syntax improves LLM accuracy and git diffs.
- **LLM-First Diagnostics:** Every compiler error MUST include a concise `fix:` hint in `myelin-diag`.
- **Zero Ambient Authority:** Every host or OS interaction requires an explicit `!capability`.
