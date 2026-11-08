# Control Flow & Guards

Control flow in Myelin is designed to eliminate nesting pyramids and bracket clutter. Every control flow keyword begins with `@`.

## If / Elif / El Statements

Branching in Myelin uses `@if`, `@elif`, and `@el`:

```myelin
#fn evaluate_grade [score:i64 -> str]
  @if score >= 90
    = "A"
  @elif score >= 75
    = "B"
  @elif score >= 60
    = "C"
  @el
    = "F"
```

Notice:
- No parentheses around conditions (`score >= 90`, not `(score >= 90)`).
- No curly braces. The branch body is determined by 2-space indentation.
- Return statements use `= <value>`.

---

## Guard Statements (`?`)

Guards allow you to assert conditions or exit early without wrapping your code in deep `@if` hierarchies.

Syntax:
```myelin
? <condition> | <action>
```

Action can be:
- `^ <Error>`: Early return with error (raise)
- `= <Value>`: Early return with value
- `@brk`: Break out of enclosing loop
- `@next`: Continue to next loop iteration

### Example: Validation Guard

```myelin
#fn transfer_funds [balance:i64 amount:i64 -> i64!str]
  -- Ensure amount is positive, otherwise return error
  ? amount > 0 | ^"amount must be positive"

  -- Ensure sufficient balance
  ? balance >= amount | ^"insufficient funds"

  remaining = balance - amount
  = remaining
```

If the condition evaluates to `true`, execution continues to the next line. If `false`, the pipe action executes immediately.

---

## Loops

Myelin provides two loop constructs: `@for` and `@loop`.

### Iteration Loop (`@for`)

```myelin
#fn print_numbers [] !io
  @for i : [1, 2, 3, 4, 5]
    io.println i
```

### Unbounded Loop (`@loop`)

```myelin
#fn count_down [start:i64] !io
  n = start
  @loop
    ? n > 0 | @brk
    io.println n
    n = n - 1
```

Use `@brk` to break and `@next` to jump to the next iteration.

---

## Pattern Matching (`@match`)

Pattern matching is exhaustive and deterministic:

```myelin
#fn status_message [status:PaymentStatus -> str]
  @match status
    PaymentStatus.Pending => "Awaiting confirmation"
    PaymentStatus.Processing => "Bank transaction in progress"
    PaymentStatus.Success => "Completed successfully"
    PaymentStatus.Failed(reason) => str.fmt "Failed: {}" reason
```
