# LLM & Autonomous Agent Integration

Myelin is built specifically to bridge Large Language Models with systems-level execution.

## The LLM Diagnostic Protocol (`--llm`)

Traditional compilers emit multi-line ASCII art and lengthy explanations designed for human eyes:

```text
error[E0308]: mismatched types
  --> src/main.rs:12:18
   |
12 |     let x: i32 = "hello";
   |            ---   ^^^^^^^ expected `i32`, found `&str`
   |            |
   |            expected due to this
```

While human-friendly, feeding this back to an LLM:
1. Burns dozens of precious context tokens.
2. Increases autoregressive latency.
3. Requires the LLM to deduce the exact column and repair step.

### Myelin Single-Line Wire Format

In Myelin, running with `--llm` formats every error, warning, and note into a deterministic **single-line wire format**:

```text
<SEVERITY> <CODE> <FILE>:<LINE>:<COL> | <KIND> <DETAIL> | fix: <REPAIR_HINT>
```

### Real Compiler Output Examples

```text
ERR E001 math.myl:5:1 | tab_indent | fix: replace tabs with 2-space multiples
ERR E002 ledger.myl:14:3 | odd_indent spaces=3 | fix: align statement to 2 or 4 spaces
ERR E008 server.myl:18:7 | missing_capability cap=!net | fix: add !net to '#fn main' signature
```

An autonomous agent can parse this with a single regex:
```regex
^(?P<sev>ERR|WARN) (?P<code>E\d+) (?P<file>[^:]+):(?P<line>\d+):(?P<col>\d+) \| (?P<detail>[^|]+) \| fix: (?P<fix>.*)$
```

The agent applies the fix in its next code revision with 0 wasted reasoning steps.

---

## The 900-Token LLM Skill File (`SKILL.md`)

Rather than requiring an agent to read a 500-page language specification, Myelin provides [`SKILL.md`](https://github.com/can/ailang/blob/main/SKILL.md) in the root repository.

### How to Use with AI Agents

When instructing Claude, ChatGPT, or Gemini to write Myelin code:
1. Provide the content of `SKILL.md` as system instructions or prompt context (~900 tokens).
2. The model will reliably emit canonical, valid `.myl` files.
3. Hook compiler verification into the agent execution loop:
   ```bash
   myelinc check generated.myl --llm
   ```
4. If output is non-empty, provide the single-line error directly back to the agent for instantaneous repair.

---

## Token Efficiency Benchmark

Writing equivalent logic across different programming languages demonstrates Myelin's dramatic token savings:

| Language | Typical Token Count (o200k) | Boilerplate / Ceremony |
| :--- | :--- | :--- |
| **Java** | 240 tokens | Classes, packages, types, braces, exceptions |
| **TypeScript** | 185 tokens | Type annotations, import/export, braces, async/await |
| **Rust** | 170 tokens | Lifetime parameters, Result match, braces, use |
| **Python** | 130 tokens | Indentation, def, colons, import, type hints |
| **Myelin (`.myl`)** | **68 tokens** | **55% - 70% reduction in tokens emitted!** |
