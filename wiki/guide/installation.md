# Installation & Toolchain

The Myelin compiler toolchain is distributed as a single unified executable: `myelinc`.

## Prerequisites

- [Rust & Cargo](https://rustup.rs/) (version 1.80 or later recommended)
- Windows, macOS, or Linux

---

## Building from Source

Clone the repository and compile `myelinc` in release mode:

```bash
git clone https://github.com/can/ailang.git
cd ailang

# Run all unit tests
cargo test

# Build the optimized compiler binary
cargo build --release -p myelinc
```

The compiled binary will be placed at:
- **Windows:** `target/release/myelinc.exe`
- **Linux / macOS:** `target/release/myelinc`

You can move `myelinc` to any directory in your system `$PATH` (e.g. `C:\Windows\System32` or `/usr/local/bin`).

---

## CLI Overview

`myelinc` provides commands tailored for both human developers and autonomous AI agents:

```bash
myelinc <COMMAND> <file.myl> [--llm]
```

| Command | Description |
| :--- | :--- |
| `myelinc run <file.myl>` | Executes the Myelin source file using the built-in driverless runtime. |
| `myelinc check <file.myl>` | Runs the lexer, parser, and semantic verifier without executing. |
| `myelinc parse <file.myl>` | Prints the Abstract Syntax Tree (AST) representation. |
| `myelinc lex <file.myl>` | Prints the raw token stream with line & column spans. |
| `--llm` flag | Formats all diagnostics into single-line machine repair hints. |

---

## Running Your First Program

Create a file named `hello.myl`:

```myelin
#mod main

#fn main [] !io
  io.println "Hello from Myelin!"
  io.println "The LLM-Native Systems Programming Language"
```

Run it immediately:

```bash
myelinc run hello.myl
```

Output:
```text
Hello from Myelin!
The LLM-Native Systems Programming Language
Execution completed in 412 µs
```

---

## Autonomous Agent Mode (`--llm`)

When invoking `myelinc` from an AI agent or automated script, always add `--llm`:

```bash
myelinc check hello.myl --llm
```

If the file contains syntax or semantic issues, `myelinc` responds with a high-density, single-line format:

```text
ERR E002 hello.myl:4:3 | odd_indent spaces=3 | fix: align statement to 2 or 4 spaces
```

Agents can parse this output with a simple regex and make instant edits without wasting context tokens.
