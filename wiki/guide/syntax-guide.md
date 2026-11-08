# Syntax & Indentation

Myelin syntax is engineered for maximum token efficiency and unambiguous LL(1) parsing.

## The Directives (`#` at Column 0)

Every top-level declaration in Myelin **must begin at column 0** with a `#` symbol. This allows both LLMs and parsers to immediately synchronize their state.

### `#mod <path>`
Declares the module namespace:
```myelin
#mod finance.ledger
```

### `#use <path>`
Imports symbols from another module:
```myelin
#use std.io
#use std.db
```

### `#rec <Ident> [field:Type ...]`
Declares a flat record type (struct):
```myelin
#rec User [id:u64 active:bool username:str score:i64]
```

### `#sum <Ident> [Variant ...]`
Declares an algebraic sum type (tagged enum):
```myelin
#sum PaymentStatus [Pending Processing Success Failed(str)]
```

### `#fn <name> [params -> ReturnType] !capabilities`
Declares a function. Function bodies are indented by 2 spaces:
```myelin
#fn calculate_tax [income:i64 rate:f64 -> f64]
  tax = income * rate
  = tax
```

### `#test "<name>" !capabilities`
Defines an inline unit test:
```myelin
#test "tax calculation is accurate"
  result = calculate_tax 1000 0.18
  ? result == 180.0
```

---

## 2-Space Indentation Rule

Myelin enforces a strict indentation discipline:
- Indentation **must always be multiples of 2 spaces** (2, 4, 6, 8...).
- Odd indentation (e.g. 1 space, 3 spaces) produces compiler error `E002`.
- Tabs (`\t`) are strictly prohibited and produce error `E001`.
- Blank lines and comments do not affect block scope.

```myelin
#fn example [] !io
  -- Correct (2 spaces)
  a = 10
  b = 20
  @if a < b
    -- Correct (4 spaces)
    io.println "a is smaller"
```

---

## Comments

Single-line comments begin with `--` and continue to the end of the line:

```myelin
-- This is a comment in Myelin
a = 42 -- Inline comment
```

---

## Literals & Primitive Types

| Type | Examples | Description |
| :--- | :--- | :--- |
| `i8` .. `i64` | `42`, `-100` | Signed fixed-width integers |
| `u8` .. `u64` | `100`, `0` | Unsigned fixed-width integers |
| `f32`, `f64` | `3.14159`, `-0.5` | IEEE-754 floating point numbers |
| `bool` | `true`, `false` | Boolean values |
| `str` | `"hello world"` | UTF-8 string values |
| `?T` | `none`, `42` | Optional value |
| `T!E` | `value ? ^Error` | Result / error union |
