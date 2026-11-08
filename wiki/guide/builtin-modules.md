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
