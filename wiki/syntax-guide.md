# Myelin Syntax Guide (for C#, Rust & JS Developers)

If you know C#, TypeScript, or Rust, Myelin's concepts are identical, but the syntax eliminates all boilerplate ceremony.

---

## Cheat Sheet Comparison

| Feature | C# / TypeScript / Rust | Myelin (`.myl`) |
| :--- | :--- | :--- |
| **Blocks** | `{ ... }` | 2-space indentation |
| **Top-Level** | `namespace`, `class`, `fn` | `#mod`, `#rec`, `#fn` |
| **Binding** | `let mut x = 10;` or `var x = 10;` | `x = 10` |
| **Function Return** | `return value;` | `= value` |
| **Error Return** | `return Result.Err(e);` | `^ e` |
| **Conditionals** | `if (x) { ... } else { ... }` | `@if x ... @el ...` |
| **Loops** | `while (true) { ... }` | `@loop ...` |
| **Break Loop** | `break;` | `@brk` |
| **Continue Loop**| `continue;` | `@next` |
| **Early Guard** | `if (!cond) return err;` | `? cond \| ^err` |
| **Async / Await**| `async Task ... await fn()` | *None! Colorless concurrency based on `!net` / `!io`* |

---

## Canonical Example: Variables & Functions

```text
#mod math.utils

#fn add [a:i64 b:i64 -> i64]
  = a + b

#fn main [] !io
  val = add 15 25
  msg = str.fmt "15 + 25 = {}" val
  io.println msg
```
