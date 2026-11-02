# Myelin Compiler Architecture

Myelin is structured as a modular multi-crate Rust workspace inspired by modern query-driven compilers (`rust-analyzer`, `oxc`, `cranelift`).

## Pipeline Overview

1. **`myelin-diag` (Diagnostics & Source Spans):**
   - Stores source text, byte offsets (`Span`), and line/column lookup tables.
   - Formats compiler errors in two modes: `--llm` (single-line ~15-token machine-actionable repair hints) and human summary mode.

2. **`myelin-lexer` (Single-Pass Indentation Lexer):**
   - Scans UTF-8 bytes in `O(n)` without regexes or heap-heavy backtracking.
   - Tracks bracket nesting (`[...]`, `(...)`) and canonical 2-space indentation levels to synthesize `Indent`, `Dedent`, and `Newline` tokens cleanly.

3. **`myelin-ast` & `myelin-parser` (LL(1) + Pratt Frontend):**
   - Top-level `#` directives and `@` control-flow constructs are parsed via deterministic `LL(1)` recursive descent.
   - Expressions combine command-style calls (`db.get_acc uid`), parenthesized calls (`LowBal(acc.bal)`), Pratt operator precedence, and suffix error propagation (`?`, `? ^Err`).

4. **`myelin-check` *(Phase 2)*:**
   - Bidirectional type inference + exhaustive `@match` verification.
   - **Capability Lattice Checker:** Verifies that every callee's `!cap` set is a subset of the caller's declared capabilities.
   - Emits the **Human Trust & Capability Manifest (`myelinc audit`)**.

5. **`myelin-mir` & `myelin-vm` *(Phase 3)*:**
   - Lowers rebound local variables (`sum = sum + x`) into pure **SSA (Static Single Assignment)** basic blocks.
   - Inserts **Perceus** static drop/reuse instructions for zero-GC deterministic memory.
   - Executes bytecode in a 64-bit Register Virtual Machine for sub-millisecond `#test` execution.

6. **`myelin-codegen` *(Phase 5)*:**
   - Emits WebAssembly (WASM) and Cranelift/LLVM native machine code.
