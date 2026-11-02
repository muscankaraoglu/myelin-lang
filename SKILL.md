---
name: myelin
description: Write, verify, and run programs in Myelin (.myl) — the token-minimal, capability-safe, flat-SSA systems language designed for LLM authorship.
---

# Myelin (`.myl`) — Canonical LLM Authoring Spec (v0.1)

Myelin is a post-human systems programming language. Do NOT write curly braces `{ }`, semicolons `;`, `let`, `mut`, `return`, `async`, `await`, `null`, or `try/catch`.

## 1. Layout & Lexical Rules
- **Encoding:** UTF-8. Comments start with `--` to end of line.
- **Blocks:** Canonical **2-space indentation** (`  `). Odd indentation is a compile error.
- **One step per line:** Every indented line inside a block is a single atomic statement.
- **Top-level items** always start at column `0` with `#`.

## 2. Types (All Single-Token Primitives)
- Signed ints: `i8` `i16` `i32` `i64`
- Unsigned ints: `u8` `u16` `u32` `u64` `usz`
- Floats: `f32` `f64`
- Text & bytes: `bool` `str` `bytes`
- Collections: `[]T` (slice/list), `[K:V]` (map)
- Optional (no `null`): `?T` (`Some(v)` or `none`; fallback: `expr ?? default`)
- Fallible (no exceptions): `T!E` (`Ok(v)` or `Err(e)`)
- Raw pointer: `*T` (requires `!unsafe.mem` capability)

## 3. Capabilities (Effect Security)
Functions without `!` tags are **100% pure & deterministic**. Any side effect MUST be declared after `]` in `#fn`:
- Storage/FS/Net: `!fs.r` `!fs.w` `!fs` `!net` `!net.http` `!db.r` `!db.w` `!db.rw`
- Host/OS: `!io` `!time` `!rand` `!env` `!proc`
- Bare-metal: `!unsafe.mem` `!unsafe.ffi`
- **Colorless I/O:** Never write `async` or `await`. The compiler schedules suspension automatically from `!` capabilities.

## 4. Top-Level Declarations
```text
#mod app.billing
#use std.db

#rec Account [id:u64 active:bool bal:i64]
#sum TxErr   [BadAmt Inactive LowBal(i64) Db(str)]

#ext "c" os_tick_ns [-> u64] !time

#fn withdraw [uid:u64 amt:i64 -> i64!TxErr] !db.rw
  req amt > 0
  ens ret >= 0
  acc = db.get_acc uid ? ^TxErr.Db
  ? acc.active         | ^TxErr.Inactive
  ? acc.bal >= amt     | ^TxErr.LowBal(acc.bal)
  nxt = acc.bal - amt
  _   = db.set_bal uid nxt ? ^TxErr.Db
  = nxt
```
- Unit-returning function omits `->`: `#fn log_msg [s:str] !io`

## 5. Statements (2-Space Indented)
- **Contract:** `req <bool_expr>` (precondition), `ens <bool_expr>` (postcondition; `ret` refers to return value). Must appear first in `#fn`.
- **Bind / Rebind:** `x = <expr>` (Compiler lowers rebinding to SSA automatically; `_ = <expr>` discards result).
- **Normal Return:** `= <expr>` (auto-wraps `Ok(v)` when return type is `T!E`; bare `=` returns unit).
- **Error / Early Return:** `^ <expr>` (returns `Err(e)`).
- **Guard / Assert:**
  - `? <cond> | ^<err_expr>` — if `<cond>` is false, early-return error.
  - `? <cond> | = <expr>` — if `<cond>` is false, early-return value.
  - `? <cond> | @next` or `| @brk` — continue or break enclosing loop if false.
  - `? <cond>` — assertion (used in `#test`).
- **Error Propagation Suffix:**
  - `v = call arg ?` — unwraps `Ok(v)` or propagates `Err(e)` to caller.
  - `v = call arg ? ^MyErr.Wrap` — unwraps `Ok(v)` or maps `Err` and returns `MyErr.Wrap`.
- **Conditionals:**
  ```text
  @if x > 10
    y = x * 2
  @elif x > 0
    y = x + 1
  @el
    y = 0
  ```
- **Loops:**
  ```text
  @for item : items
    ? item.active | @next
    total = total + item.bal

  @loop
    ? idx < 10 | @brk
    idx = idx + 1
  ```
- **Exhaustive Pattern Match:**
  ```text
  @match res
    Ok(v)            => = v
    Err(TxErr.BadAmt)=> = 0
    _                => = -1
  ```

## 6. Calls & Expressions
- Space-separated command call: `db.set_bal uid nxt` or parenthesized `TxErr.LowBal(acc.bal)` (use `(...)` for nested call arguments: `math.max a (b + 1)`).
- Operators (high to low): `.field` `[idx]` -> unary `-` `!` -> `*` `/` `%` -> `+` `-` -> `<` `<=` `>` `>=` -> `==` `!=` -> `&&` -> `||` -> `??` -> `?`.

## 7. Compiler Toolchain Loop
- Check with LLM repair diagnostics: `myelinc check <file.myl> --llm`
- Run inline `#test` blocks: `myelinc test <file.myl>`
- Generate human trust manifest: `myelinc audit <file.myl>`
