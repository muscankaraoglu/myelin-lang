# Myelin Lang: Complete Context & Memory State Transfer

> **Purpose:** This file preserves the entire conversational history, architectural decisions, and operational state from the initial bootstrap conversation to enable seamless continuation in **Antigravity Remote Project** on GitHub (`muscankaraoglu/myelin-lang`).

---

## 1. Project Genesis & Creator's Vision
- **Creator:** Mustafa Can Karaoğlu (`muscankaraoglu`).
- **Core Vision:** Existing programming languages (C#, TypeScript, Rust, Python) were engineered for human biology (60-WPM fingers, visual brace matching, verbose keywords). They waste 55–65% output tokens in LLM generation and leak runtime authority through implicit state, nulls, and ambient OS calls.
- **Myelin Objective:** The first universal systems language designed from first principles:
  - **Written by LLMs:** Canonical, brace-free **Flat Line-SSA** with strict 2-space indentation and single-line `--llm` repair hints.
  - **Executed by Machines at Bare-Metal Speed:** 64-bit Register Virtual Machine and Perceus compile-time reference counting (zero GC pauses, zero Rust lifetime complexity).
  - **Audited by Humans Without Reading Code:** Deterministic Capability Security (`!net`, `!db`, `!fs`, `!io`, `!term`, `!cli`, `!unsafe.*`) and cryptographic SHA-256 Human Trust Manifests (`myelinc audit`).

---

## 2. Key Architecture Decisions & Breakthroughs
1. **Name & Identity:**
   - Name: **Myelin** (`.myl`) — named after the myelin sheath that wraps nerve axons to accelerate electrical impulses by 100x and prevent signal leakage.
   - CLI Toolchain: `myelinc.exe` (or `myelinc`), self-contained standalone binary with **zero external dependencies**.
2. **Dual-License Model:**
   - **MIT OR Apache-2.0** (Rust standard). Ensures zero friction for community adoption and complete enterprise/patent safety when major tech companies fund or contribute.
3. **Batteries-Included Standard Modules (Tested & Working):**
   - `io.println`: Formatted console printing (`!io`).
   - `str.fmt`: Zero-cost template formatting.
   - `db.set`, `db.get`, `db.has`: Built-in zero-dependency embedded key-value and document storage (`!db`).
   - `http.serve`: Native high-concurrency bare-metal HTTP web server (`!net`).
   - `term.*`: ANSI escape styling (`term.green`, `term.bold`, `term.cyan`, etc.) (`!term`).
   - `cli.*`: CLI flag and argument inspection (`cli.args`, `cli.flag_str`, `cli.flag_int`) (`!cli`).
4. **Human Trust Engine (`myelinc audit`):**
   - Emits NIST FIPS 180-4 standard SHA-256 module fingerprint.
   - Mathematically proves capability boundaries and detects undeclared ambient leaks with 0 violations.

---

## 3. Implemented Workspace Crates & Assets
```text
myelin-lang/
├── Cargo.toml                 # Root workspace manifest
├── myelinc.exe                # 1.4 MB Standalone Native Executable
├── SKILL.md                   # Canonical ~900-token LLM authoring specification
├── assets/
│   └── myelin-quickstart.svg  # Claude Code CLI interactive animated banner
├── crates/
│   ├── myelin-diag/           # Spans, error codes, and single-line --llm repair protocol
│   ├── myelin-ast/            # Abstract Syntax Tree data structures
│   ├── myelin-lexer/          # Zero-copy single-pass indentation lexer
│   ├── myelin-parser/         # Deterministic LL(1) + Pratt expression parser
│   └── myelinc/               # CLI driver (run, audit, check, parse, lex) + audit.rs
├── editors/
│   └── vscode/                # Official VS Code / Cursor IDE Extension
│       ├── package.json
│       ├── syntaxes/myelin.tmLanguage.json
│       ├── snippets/myelin.json
│       └── language-configuration.json
├── wiki/                      # Bilingual (EN / TR) VitePress Documentation Hub
│   ├── .vitepress/config.mts  # Shiki TextMate highlighter + i18n routing
│   ├── guide/*.md             # English documentation suite (9 chapters)
│   └── tr/guide/*.md          # Turkish documentation suite (9 chapters)
└── .agent-skills/             # Specialized Agent Swarm Skills
```

---

## 4. Installed Subagent Swarm Skills
All 12 specialized skill definitions are installed in `.agent-skills/` and `.gemini/antigravity/skills/myelin-orchestrator/`:
1. `core-compiler.md`: Lexer, Parser, SSA lowering, and Type/Capability Lattice.
2. `stdlib-web.md`: Native HTTP REST server, embedded database, and JSON runtime.
3. `stdlib-desktop.md`: Native windowing, canvas, and desktop application bridge.
4. `stdlib-mobile.md`: Android NDK (.so) and iOS FFI shared logic bridge.
5. `stdlib-cli.md`: Terminal ANSI styling, CLI flags, and interactive TUI utilities.
6. `stdlib-ai.md`: On-device AI model loading (GGUF), tensor execution, and inference.
7. `stdlib-embedded.md`: Bare-metal microcontroller execution (ARM Cortex-M, RISC-V).
8. `stdlib-cloud.md`: Cloudflare Workers & AWS Lambda micro-WASM artifacts.
9. `interop-ffi.md`: Zero-cost C ABI (#ext "c"), Rust, C#, and Python interop.
10. `benchmark-audit.md`: Formal verification, token savings analysis, and audit reports.
11. `community-devrel.md`: Ecosystem growth, release logs, and developer documentation.
12. `syntax-tooling-sync.md`: Real-time synchronization of VS Code grammars and Shiki highlighter.
13. `wiki-doc-sync.md`: Bilingual (EN/TR) VitePress documentation synchronization.

---

## 5. Next Immediate Roadmap Milestones
When continuing in Remote Project, prompt the agent with any of these tracks:
- **Milestone A (Web/REST):** Implement `json.parse` and request body parsing in `http.serve` for dynamic CRUD REST APIs.
- **Milestone B (Native GUI):** Prototype `std.gui` native lightweight windowing for Windows/macOS.
- **Milestone C (SSA / VM):** Lower rebound variables to pure Single Static Assignment (`myelin-mir`).
- **Milestone D (AI Inference):** Prototype local GGUF/ONNX model execution via `std.ai`.
