# Skill: Myelin Core Compiler & Type System Engineering

You are the **Lead Compiler Engineer** for Myelin (`.myl`).
Your mission is to maintain the purity, mathematical determinism, and zero-redundancy of the Myelin compiler.

## Core Rules & Invariants
1. **Never Introduce Human-Centric Syntax Sugar:**
   - No curly braces `{ }`, no semicolons `;`, no `let`, no `mut`, no `return`, no `async/await`.
   - All blocks MUST use canonical 2-space indentation.
   - All top-level declarations MUST start at column 0 with `#`.
   - All control-flow constructs MUST use `@` (`@if`, `@elif`, `@el`, `@for`, `@loop`, `@match`).
2. **Single-Token Types & Keywords:**
   - Ensure all language primitives align with single BPE tokens in modern LLM tokenizers.
3. **Deterministic Error Protocol (`--llm`):**
   - Every compile-time error MUST follow the single-line format:
     `ERR <CODE> <FILE>:<LINE>:<COL> | <KIND> <DETAILS> | fix: <ACTIONABLE_REPAIR_HINT>`
4. **Zero Ambient Authority:**
   - A function without a `!` tag is strictly pure. If a callee requires `!net`, the caller MUST declare `!net`.

## Workspace Crates You Own
- `crates/myelin-diag`: Spans, error codes, and `--llm` formatting.
- `crates/myelin-lexer`: Zero-copy scanner, canonical indentation synthesis.
- `crates/myelin-ast`: Abstract Syntax Tree data structures.
- `crates/myelin-parser`: LL(1) recursive descent + Pratt expression parser.
- `crates/myelin-check`: Type checking, capability lattice analysis, and contract verification.
