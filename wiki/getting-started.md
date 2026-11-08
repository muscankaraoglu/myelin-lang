# Getting Started with Myelin (`.myl`)

Welcome to Myelin! This guide will get you up and running in less than 2 minutes.

---

## 1. What is Myelin?
Myelin is a systems programming language engineered for the AI agent era:
- **Zero Syntax Ceremony:** No curly braces `{ }`, no semicolons `;`, no `let`, no `mut`, no `return`, no `async/await`.
- **Token Optimal:** Generates ~60% fewer LLM tokens than Rust or C#.
- **Capability Security:** Strictly sandboxed. Functions cannot touch files, network, or OS resources without explicit `!tags`.
- **Standalone Binary:** The compiler (`myelinc.exe`) has zero external dependencies.

---

## 2. Running Your First Program

Create a file named `hello.myl`:

```text
#mod main

#fn main [] !io
  io.println "Hello from Myelin!"
```

Run it immediately using the Myelin CLI:

```bash
myelinc run hello.myl
```

Output:
```text
Hello from Myelin!
-- finished in 143 µs
```

---

## 3. Core Commands
- `myelinc run <file.myl>` — Execute the module's `#fn main []`.
- `myelinc check <file.myl> --llm` — Perform static type and syntax verification with single-line LLM repair hints.
- `myelinc parse <file.myl>` — Inspect the parsed Abstract Syntax Tree (AST).
- `myelinc lex <file.myl>` — View zero-copy indentation tokens.
