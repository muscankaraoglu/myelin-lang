# Built-in Modules Reference

Myelin ships with a "batteries-included" runtime. You do not need package managers to write Web APIs or access data storage.

---

## 1. `io` Module (Requires `!io`)
- `io.println args...` — Prints formatted arguments to standard output.

```text
#fn main [] !io
  io.println "User ID:" 101 "Name:" "Can"
```

---

## 2. `str` Module (Pure, Zero Capability Needed)
- `str.fmt template args...` — Replaces `{}` placeholders with arguments.

```text
greeting = str.fmt "Hello, {}! You have {} messages." "Can" 5
```

---

## 3. `db` Module (Requires `!db`)
High-performance zero-dependency embedded key-value and document engine:
- `db.set key val` — Sets a key to a value.
- `db.get key` — Gets a key (returns `"none"` if missing).
- `db.has key` — Checks if key exists (returns `true`/`false`).

```text
#fn save_user [id:str name:str] !db
  _ = db.set id name
```

---

## 4. `http` Module (Requires `!net`)
Native, high-concurrency bare-metal HTTP web server:
- `http.serve port handler_fn_name` — Listens on `127.0.0.1:<port>` and routes all requests to the named handler.

```text
#fn router [path:str -> str]
  @if path == "/status"
    = "{\"status\":\"healthy\"}"
  @el
    = "Welcome to Myelin Web API!"

#fn main [] !net !io
  io.println "Starting server on port 8080..."
  _ = http.serve 8080 "router"
```
