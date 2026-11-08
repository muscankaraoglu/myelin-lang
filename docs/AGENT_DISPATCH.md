# Myelin Subagent Orchestration & Task Dispatch Guide

Use these prompt templates to launch autonomous subagents for specific roadmap tasks.
Each subagent runs in its own context, reads its specialized skill, and reports back.

---

### Task Brief A: Capability Lattice Checker (`myelin-check`)
**Target Subagent Role:** `Compiler Engineer`  
**Skill:** `.agent-skills/core-compiler.md`  
**Prompt Template:**
```text
Task: Implement the Static Capability Checker in crates/myelin-check.
Instructions:
1. Read .agent-skills/core-compiler.md and docs/SPEC.md.
2. Inspect the AST in crates/myelin-ast/src/lib.rs.
3. For each `#fn`, collect all called functions and builtins.
4. Verify that any callee's required capabilities (e.g., !net, !db, !fs, !io) are a strict subset of the caller's declared capabilities.
5. If an undeclared capability is used, emit error E021 in single-line --llm format with a fix hint.
6. Run `cargo test --workspace` to ensure all tests pass.
```

---

### Task Brief B: JSON Module & Full REST Routing
**Target Subagent Role:** `Runtime & Ecosystem Engineer`  
**Skill:** `.agent-skills/stdlib-web.md`  
**Prompt Template:**
```text
Task: Implement built-in JSON parsing and dynamic REST routing in Myelin.
Instructions:
1. Read .agent-skills/stdlib-web.md.
2. In crates/myelinc/src/main.rs, extend the runtime with `json.parse` and `json.get`.
3. Support HTTP methods (GET, POST) in `http.serve` so handlers can inspect request headers and body.
4. Create examples/rest_api.myl showcasing a complete CRUD endpoint.
5. Verify with `cargo run -p myelinc -- run examples/rest_api.myl`.
```

---

### Task Brief C: `myelinc audit` Human Trust Manifest
**Target Subagent Role:** `Security & Verification Engineer`  
**Skill:** `.agent-skills/benchmark-audit.md`  
**Prompt Template:**
```text
Task: Implement the `myelinc audit <file.myl>` CLI command.
Instructions:
1. Read .agent-skills/benchmark-audit.md.
2. In crates/myelinc/src/main.rs, add the `audit` subcommand.
3. Parse the file, extract all `#fn`, `#rec`, and `#test` declarations.
4. Print the formatted Trust & Capability Manifest showing SHA256, active capabilities, contracts, and test status.
5. Add a test in examples/ledger.myl verifying `myelinc audit examples/ledger.myl` works cleanly.
```
