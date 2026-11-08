# Capability Security & The Human Trust Model

The central problem of AI-generated code is **trust**: If autonomous agents write 10,000 lines of code, humans cannot manually review every pull request.

Myelin solves this with **Deterministic Capability Security**.

---

## The Rule of Zero Ambient Authority
By default, every function in Myelin is **100% pure and deterministic**. It cannot:
- Touch the file system
- Make network calls
- Read the system clock or randomness
- Execute OS commands

If a function needs any side effect, it **must explicitly declare that capability in its signature**:

```text
# Pure function: cannot access network or disk
#fn add [a:i64 b:i64 -> i64]
  = a + b

# Impure function: requires explicit network permission
#fn fetch_data [url:str -> str] !net
  ...
```

---

## Standard Capabilities Matrix

| Tag | Authority Granted |
| :--- | :--- |
| `!io` | Standard console output and input |
| `!net` | Network sockets, HTTP server and client |
| `!db` | Embedded and external database storage |
| `!fs` | File system read and write operations |
| `!proc` | Spawning and managing child processes |
| `!gui` | Creating native desktop windows and canvases |
| `!mobile.*`| Accessing mobile camera, location, and haptics |
| `!unsafe.mem`| Bare-metal raw pointer manipulation (`*T`) |

---

## The Human Trust Manifest (`myelinc audit`)
Instead of reading code, humans run:

```bash
myelinc audit app.myl
```

The compiler proves mathematically which capabilities the module uses, guarantees that undeclared capabilities cannot execute, and verifies inline `#test` proofs.
