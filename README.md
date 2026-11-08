# Myelin (`.myl`)

[Documentation Wiki (wiki/)](./wiki/README.md) | [Türkçe Dokümantasyon (README.tr.md)](./README.tr.md) | [LLM Authoring Skill (SKILL.md)](./SKILL.md) | [Language Spec (docs/SPEC.md)](./docs/SPEC.md) | [Compiler Architecture (docs/ARCHITECTURE.md)](./docs/ARCHITECTURE.md)

**Myelin** is a capability-safe, flat-SSA, zero-GC systems programming language designed from first principles **to be written by Large Language Models, executed at bare-metal speed by machines, and audited by humans without reading a single line of code.**

In neurobiology, *myelin* wraps neural axons to prevent signal leakage and accelerate impulse transmission by 100x. In software engineering, **Myelin** eliminates decades of human-ergonomic ceremony between neural reasoning and machine instructions.

---

## Why Myelin Exists

Mainstream languages (C#, TypeScript, Python, Rust) were engineered around human biological constraints: 60-WPM keyboards, visual brace matching, `async/await` coloring, and prose compiler errors.

When autonomous coding agents write software, those human affordances become liabilities:
1. **55–65% Output Token Waste:** Boilerplate keywords, nested closing braces, and repetitive type/effect wrappers inflate autoregressive latency and exhaust context windows.
2. **Implicit Side-Effect Bugs:** `null`, unchecked exceptions, and ambient OS authority allow subtle bugs and unsafe behavior to hide inside thousands of lines of generated code.
3. **The Human Review Bottleneck:** If humans stop writing code, how do humans *trust* running it?

**Myelin flips the contract:**
- **LLMs read [`SKILL.md`](./SKILL.md)** (~900 tokens) and emit canonical, brace-free **Flat Line-SSA** (`.myl`) with single-line compiler repair feedback (`myelinc check --llm`).
- **Machines compile `.myl`** in microseconds into a 64-bit Register VM (for sub-millisecond test loops) or native/WASM code using **Perceus compile-time reference counting & regional arenas** (zero GC pauses, zero Rust lifetime battles).
- **Humans run `myelinc audit`** and inspect a mathematically verified **Trust & Capability Manifest** (`!fs`, `!net`, `!db`, `req`/`ens` contracts, inline `#test` proofs) instead of reading source code.

---

## A Taste of Myelin

```text
#mod bank.ledger
#use std.db

#rec Account [id:u64 active:bool bal:i64]
#sum TxErr   [BadAmt Inactive LowBal(i64) Db(str)]

#fn withdraw [uid:u64 amt:i64 -> i64!TxErr] !db.rw
  req amt > 0
  ens ret >= 0
  acc = db.get_acc uid ? ^TxErr.Db
  ? acc.active         | ^TxErr.Inactive
  ? acc.bal >= amt     | ^TxErr.LowBal(acc.bal)
  nxt = acc.bal - amt
  _   = db.set_bal uid nxt ? ^TxErr.Db
  = nxt

#test "withdraw updates balance" !mock.db
  db.seed_acc 1 true 500
  rem = withdraw 1 200 ?
  ? rem == 300
```

---

## Workspace Crates

| Crate | Responsibility |
| :--- | :--- |
| `crates/myelin-diag` | Source spans, error codes, and the single-line `--llm` repair diagnostic protocol |
| `crates/myelin-ast` | Typed Abstract Syntax Tree definitions for `.myl` modules |
| `crates/myelin-lexer` | Single-pass zero-copy lexer with canonical 2-space indentation tracking |
| `crates/myelin-parser` | Deterministic LL(1) line-oriented + Pratt expression parser |
| `crates/myelinc` | Unified CLI toolchain (`lex`, `parse`, `check`, `audit`, `test`, `run`) |

---

## Quick Start

```bash
# Run the full compiler test suite
cargo test

# Parse and inspect a Myelin module in LLM diagnostic mode
cargo run -p myelinc -- check examples/ledger.myl --llm

# Inspect the parsed AST structure
cargo run -p myelinc -- parse examples/ledger.myl
```

---

## Contributing

Myelin is built openly in Rust. Read [`CONTRIBUTING.md`](./CONTRIBUTING.md) and [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) to jump in!

## License

MIT License — see [`LICENSE`](./LICENSE).
