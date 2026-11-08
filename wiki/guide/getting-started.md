# Introduction & Philosophy

## What is Myelin?

**Myelin (`.myl`)** is a capability-safe, flat-SSA, zero-garbage-collection systems programming language designed from first principles:
- **Written by Large Language Models (LLMs)** with minimal token overhead and deterministic grammar.
- **Executed at bare-metal speed** by microsecond compilers, zero-dependency runtimes, and register VMs.
- **Audited by humans** without reading thousands of lines of generated code.

In neurobiology, *myelin* forms an insulating sheath around nerve fibers, eliminating signal leakage and accelerating neural impulse transmission by up to 100x. In software engineering, **Myelin** eliminates decades of human-ergonomic ceremony between neural reasoning and silicon execution.

---

## The Human-Language Bottleneck

Mainstream programming languages (C#, TypeScript, Python, Rust, Go) were built for human biological constraints:
- 60-WPM mechanical keyboards
- Visual curly brace matching (`{ ... }`)
- Complex nesting and prose compiler errors
- Implicit runtime authority (functions can secretly access the network, file system, or system clock)

When autonomous coding agents write code in these languages, those human conveniences become critical flaws:

| Problem in Classic Languages | How Myelin Solves It |
| :--- | :--- |
| **55–65% Token Waste:** Closing braces, boilerplate keywords (`function`, `return`, `public static void`), verbose error handling. | **Flat Line-SSA & Token Alignment:** Every symbol is optimized for modern LLM BPE tokenizers. No trailing closing braces. |
| **Hidden Side-Effects:** Functions secretly making network calls, deleting files, or leaking memory. | **Zero Ambient Authority:** Pure by default. All side effects require explicit capability flags (`!net`, `!db`, `!fs`). |
| **Context Window Exhaustion on Compiler Errors:** 20-line Rust/C++ error messages pollute LLM context. | **Single-Line Diagnostic Wire (`--llm`):** Precise single-line repair instructions that agents can parse in 1 token step. |
| **Human Review Burnout:** Humans cannot safely review 10,000 lines of generated code every hour. | **Trust & Capability Manifest (`myelinc audit`):** Humans inspect verified capabilities and contracts, not lines of code. |

---

## Core Guarantees

1. **Deterministic LL(1) Parsing:** Every top-level item begins with `#` at column 0. Every control flow statement begins with `@`. There is never an ambiguous parsing state.
2. **2-Space Indentation Grammar:** Tabs are strictly forbidden (`E001`). Indentation must always be multiples of 2 spaces.
3. **Zero Null & Zero Unchecked Exceptions:** Optionality (`?T`) and failure (`T!E`) are first-class value types with single-character operators (`?`, `^`).
4. **Driverless Embedded Runtime:** Includes a native embedded key-value database and HTTP server without pulling in gigabytes of `node_modules` or complex C libraries.

---

## Next Steps

- Proceed to [Installation & Toolchain](/guide/installation) to install `myelinc` and run your first Myelin program.
- Explore the [Syntax Guide](/guide/syntax-guide) to understand the core directives and statement forms.
