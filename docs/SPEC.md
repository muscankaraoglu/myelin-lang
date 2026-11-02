# Myelin Language Specification (v0.1 Draft)

## 1. Design Goals
1. **Token Optimality:** Align every keyword and operator with single BPE tokens in modern LLM tokenizers (`o200k_base` / `cl100k_base` / SentencePiece).
2. **LL(1) Determinism:** Every top-level item starts with `#` at column `0`; every control-flow block starts with `@`; every statement form is disambiguated within 1–2 lookahead tokens.
3. **Zero Ambient Authority:** Pure functions cannot touch I/O, clocks, randomness, or raw pointers unless granted explicit `!capability` tags.
4. **Zero Null / Zero Exceptions:** Optionality (`?T`) and failure (`T!E`) are first-class value types with single-character propagation (`?`, `??`, `^`).
5. **Deterministic Memory:** Value semantics backed by Perceus static reference counting with drop/reuse specialization and regional arenas.

---

## 2. Lexical Grammar

- **Comments:** `--` until `\n`.
- **Indentation:** Outside `[...]` and `(...)` brackets, leading spaces at the start of a non-empty line MUST be a multiple of `2`. Increasing indent by `2 * k` emits `k` `INDENT` tokens; decreasing emits `DEDENT` tokens. Tabs (`\t`) are rejected (`E001`).
- **Identifiers:** `[a-zA-Z_][a-zA-Z0-9_]*`.
- **Directives (Column 0):** `#mod`, `#use`, `#rec`, `#sum`, `#ext`, `#fn`, `#test`.
- **Control Keywords:** `@if`, `@elif`, `@el`, `@for`, `@loop`, `@match`, `@next`, `@brk`.
- **Contract Keywords:** `req`, `ens`.
- **Capabilities:** `!` followed by dotted path (e.g., `!db.rw`, `!fs.r`, `!net.http`, `!unsafe.mem`).

---

## 3. Syntactic Grammar (EBNF Summary)

```ebnf
Module      ::= Item* EOF
Item        ::= ModDecl | UseDecl | RecDecl | SumDecl | ExtDecl | FnDecl | TestDecl

ModDecl     ::= "#mod" Path NL
UseDecl     ::= "#use" Path NL
RecDecl     ::= "#rec" Ident "[" Field* "]" NL
Field       ::= Ident ":" Type

SumDecl     ::= "#sum" Ident "[" Variant* "]" NL
Variant     ::= Ident ("(" Type (","? Type)* ")")?

ExtDecl     ::= "#ext" StrLit Ident Signature Cap* NL
FnDecl      ::= "#fn" Ident Signature Cap* NL INDENT Contract* Stmt+ DEDENT
TestDecl    ::= "#test" StrLit Cap* NL INDENT Stmt+ DEDENT

Signature   ::= "[" Param* ("->" Type)? "]"
Param       ::= Ident ":" Type
Cap         ::= "!" Path

Contract    ::= ("req" | "ens") Expr NL

Stmt        ::= Ident "=" Expr NL
              | "=" Expr? NL
              | "^" Expr NL
              | "?" Expr ("|" GuardAct)? NL
              | "@next" NL
              | "@brk" NL
              | IfStmt | ForStmt | LoopStmt | MatchStmt
              | Expr NL

GuardAct    ::= "^" Expr | "=" Expr? | "@next" | "@brk"
IfStmt      ::= "@if" Expr Block ("@elif" Expr Block)* ("@el" Block)?
ForStmt     ::= "@for" Ident ":" Expr Block
LoopStmt    ::= "@loop" Block
MatchStmt   ::= "@match" Expr NL INDENT MatchArm+ DEDENT
MatchArm    ::= Pattern "=>" (Stmt | NL INDENT Stmt+ DEDENT)
Block       ::= NL INDENT Stmt+ DEDENT
```

---

## 4. Type System

| Syntax | Meaning |
| :--- | :--- |
| `i8..i64`, `u8..u64`, `usz` | Fixed-width & pointer-sized integers |
| `f32`, `f64` | IEEE-754 floating point |
| `bool`, `str`, `bytes` | Boolean, UTF-8 string value, byte slice |
| `[]T` | Contiguous slice / vector |
| `[K:V]` | Deterministic hash/B-tree map |
| `?T` | Optional value (`Some(T)` or `none`) |
| `T!E` | Result value (`Ok(T)` or `Err(E)`) |
| `*T` | Raw memory pointer (restricted to `!unsafe.mem`) |

---

## 5. LLM Diagnostic Wire Format (`--llm`)

Every diagnostic fits on a single line for minimal token overhead:

```text
<SEVERITY> <CODE> <FILE>:<LINE>:<COL> | <KIND> <DETAIL> | fix: <REPAIR_HINT>
```

Example:
```text
ERR E002 ledger.myl:14:3 | odd_indent spaces=3 | fix: align statement to 2 or 4 spaces
```
