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

---

## 6. `json` — Pure & Deterministic JSON Engine

The `json` module is a pure, zero-allocation runtime parser and serializer. Because JSON manipulation is deterministic and has no side effects, it requires **zero ambient capabilities** (100% pure).

### Available Methods:
- `json.parse [json_str -> doc]`: Parses JSON into Myelin records/maps.
- `json.stringify [val -> str]`: Serializes any Myelin value to JSON.
- `json.get [doc, path_str -> val]`: Dotted property lookup (e.g. `json.get user "address.city"`).
- `json.set [doc, path_str, new_val -> doc]`: Immutable update for record fields.
- `json.has [doc, key:str -> bool]`: Checks if a key exists in a JSON object.

### Example:

```myelin
#mod main

#fn main [] !io
  raw = "{\"user\":\"Can\",\"roles\":[\"architect\",\"creator\"],\"active\":true}"
  doc = json.parse raw

  user_name = json.get doc "user"
  first_role = json.get doc "roles.0"
  io.println "User:" user_name "Primary Role:" first_role

  updated = json.set doc "version" "v1.0"
  io.println "Reserialized:" (json.stringify updated)
```

---

## 7. `gui` — Native Lightweight Desktop GUI

Myelin provides a zero-electron native desktop windowing engine (requires `!gui` capability). On Windows, it binds directly to native OS controls (Win32 GDI/controls) with 0ms startup time and zero external DLLs.

### Available Methods:
- `gui.alert [title:str, msg:str]`: Displays a native OS alert dialog.
- `gui.confirm [title:str, msg:str -> bool]`: Displays a native OK/Cancel dialog.
- `gui.window [title:str, width:i64, height:i64 -> win_id]`: Creates a native window.
- `gui.text [win, text:str, x:i64, y:i64, w:i64, h:i64]`: Adds a native text label.
- `gui.button [win, text:str, x:i64, y:i64, w:i64, h:i64, on_click_fn:str]`: Adds a native button control with an event callback.
- `gui.show [win]`: Opens the window and starts the native OS event loop.

### Example:

```myelin
#mod main

#fn on_save [] !gui !db
  _ = db.set "state" "active"
  gui.alert "Myelin App" "Record saved to database!"

#fn main [] !gui !db !io
  io.println "Launching desktop interface..."
  win = gui.window "Myelin Native Desktop" 450 260
  gui.text win "Welcome to Myelin Zero-Electron Desktop" 25 25 400 24
  gui.button win "Save State" 25 80 130 36 "on_save"
  gui.show win
```
