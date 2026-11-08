# Standard Built-in Modules

Myelin ships with a **driverless runtime** built directly into `myelinc`. It provides essential building blocks for microservices, data persistence, and system utilities with **zero external dependencies**.

---

## 1. `io` — Console & Debugging

The `io` module provides console output facilities (requires `!io` capability).

### `io.println [arg1, arg2, ...]`
Prints values to standard output separated by spaces, followed by a newline:

```myelin
#fn main [] !io
  io.println "Counter initialized to:" 42
```

---

## 2. `str` — String Formatting

The `str` module provides fast string templating and manipulation.

### `str.fmt [template, arg1, arg2, ...]`
Substitutes occurrences of `{}` with stringified arguments:

```myelin
#fn format_greeting [name:str role:str -> str]
  msg = str.fmt "Hello {}, your role is: {}" name role
  = msg
```

---

## 3. `db` — Embedded Key-Value Storage

Myelin features a zero-dependency, thread-safe embedded key-value database right inside the runtime (requires `!db` capability).

### Available Methods:
- `db.set [key:str, val:str]`: Inserts or updates a key-value pair.
- `db.get [key:str -> str]`: Retrieves a string value for the key (returns `"none"` if absent).
- `db.has [key:str -> bool]`: Returns `true` if the key exists in the database.

### Example:

```myelin
#mod main

#fn main [] !db !io
  -- Store entries
  _ = db.set "user:101:name" "Alice"
  _ = db.set "user:101:role" "Site Reliability Engineer"

  -- Query entries
  user_name = db.get "user:101:name"
  user_role = db.get "user:101:role"

  report = str.fmt "User found: {} [{}]" user_name user_role
  io.println report

  -- Check existence
  is_present = db.has "user:101:name"
  io.println "Is user present in DB:" is_present
```

---

## 4. `http` — Native Microservice Server

Myelin includes a high-performance HTTP server capable of serving web APIs, status endpoints, and agent hooks directly without Express, Nginx, or external runtimes (requires `!net` capability).

### `http.serve [port:i64, handler_name:str]`
Listens on the given port (e.g. `127.0.0.1:8080`) and forwards incoming requests to the specified Myelin handler function.

The handler function receives `path:str` and returns a response body `str`.

### Complete Example:

```myelin
#mod main

#fn handle_request [path:str -> str]
  @if path == "/"
    = "Welcome to Myelin Web API! Server running with 0 external dependencies."
  @elif path == "/status"
    = "{\"status\":\"healthy\",\"engine\":\"myelinc-v0.1.0\",\"runtime\":\"driverless\"}"
  @el
    = str.fmt "404 Not Found: Path '{}' is not registered" path

#fn main [] !net !io
  io.println "Starting Myelin HTTP Server on port 8080..."
  _ = http.serve 8080 "handle_request"
```

Run this file with `myelinc run web_api.myl` and test it with curl or in your browser:
```bash
curl http://127.0.0.1:8080/status
```
Response:
```json
{"status":"healthy","engine":"myelinc-v0.1.0","runtime":"driverless"}
```

---

## 5. `term` — Terminal Styling & Colors

The `term` module provides ANSI terminal color and text decoration utilities for building rich, developer-grade CLI experiences with zero runtime overhead (requires `!term` capability).

### Available Methods:
- `term.green [text:str -> str]`: Wraps text in ANSI green (`\x1b[32m ... \x1b[0m`).
- `term.red [text:str -> str]`: Wraps text in ANSI red (`\x1b[31m ... \x1b[0m`).
- `term.yellow [text:str -> str]`: Wraps text in ANSI yellow (`\x1b[33m ... \x1b[0m`).
- `term.blue [text:str -> str]`: Wraps text in ANSI blue (`\x1b[34m ... \x1b[0m`).
- `term.cyan [text:str -> str]`: Wraps text in ANSI cyan (`\x1b[36m ... \x1b[0m`).
- `term.magenta [text:str -> str]`: Wraps text in ANSI magenta (`\x1b[35m ... \x1b[0m`).
- `term.gray [text:str -> str]` / `term.dim [text:str -> str]`: Wraps text in ANSI dim gray (`\x1b[90m ... \x1b[0m`).
- `term.bold [text:str -> str]`: Wraps text in ANSI bold (`\x1b[1m ... \x1b[0m`).
- `term.reset`: Emits standard ANSI reset sequence (`\x1b[0m`).

Styles can be nested cleanly (e.g. `term.bold (term.green "PASSED")`).

### Example:

```myelin
#mod main

#fn main [] !io !term
  banner = term.bold (term.cyan "=== MYELIN DIAGNOSTICS REPORT ===")
  io.println banner
  io.println (term.green "  [PASS] Lexical invariant check (0 tab characters)")
  io.println (term.yellow "  [WARN] Indentation depth equals 4 spaces")
  io.println (term.red   "  [FAIL] 0 violations detected")
```

---

## 6. `cli` — Command-Line Arguments & Flags

The `cli` module parses command-line arguments and option flags passed to the program after the `.myl` script name (requires `!cli` capability).

### Available Methods:
- `cli.args`: Returns the list of positional and flag arguments as `[]str`.
- `cli.args [index:i64]`: Retrieves the argument at `index` (or raises an out-of-bounds error).
- `cli.count` / `cli.arg_count`: Returns the total number of arguments as `i64`.
- `cli.flag_bool [flag:str -> bool]` / `cli.has_flag [flag:str -> bool]`: Returns `true` if the flag exists in arguments.
- `cli.flag_str [flag:str, default:str -> str]`: Retrieves the string value immediately following `flag`, or returns `default`.
- `cli.flag_int [flag:str, default:i64 -> i64]`: Retrieves the integer value immediately following `flag`, or returns `default`.

### Complete CLI Application Example:

```myelin
#mod main

#fn main [] !io !cli !term
  io.println (term.bold (term.cyan "=== Myelin Micro-Service Builder ==="))

  args = cli.args
  arg_count = args.len
  io.println (str.fmt "Received {} command-line arguments" arg_count)

  -- Parse flags
  is_release = cli.flag_bool "--release"
  target     = cli.flag_str "--target" "x86_64"
  workers    = cli.flag_int "--jobs" 4

  io.println (str.fmt "Target Architecture: {}" target)
  io.println (str.fmt "Worker Threads:      {}" workers)

  @if is_release
    io.println (term.green "Profile: RELEASE (Optimized, LTO active)")
  @el
    io.println (term.yellow "Profile: DEV (Assertions and capability traces active)")
```

Execute this script with arguments:

```bash
myelinc run build_service.myl --release --target arm64 --jobs 8
```

Output:
```text
=== Myelin Micro-Service Builder ===
Received 5 command-line arguments
Target Architecture: arm64
Worker Threads:      8
Profile: RELEASE (Optimized, LTO active)
-- finished in 329 µs
```
