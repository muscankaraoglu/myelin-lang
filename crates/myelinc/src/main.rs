//! `myelinc` â€” Command-line compiler & verification driver for Myelin (`.myl`).

use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::process::ExitCode;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use myelin_ast::{BinOp, Expr, FnDecl, GuardAction, Item, Literal, Stmt, UnaryOp};
use myelin_diag::SourceFile;
use myelin_lexer::tokenize;
use myelin_parser::parse_source;

mod audit;
pub mod gui;
pub mod json;

fn print_usage() {
    eprintln!(
        "Myelin Compiler Toolchain (myelinc v0.1.0)\n\
         Usage:\n  \
           myelinc run   <file.myl> [--llm]\n  \
           myelinc check <file.myl> [--llm]\n  \
           myelinc audit <file.myl> [--llm]\n  \
           myelinc parse <file.myl> [--llm]\n  \
           myelinc lex   <file.myl> [--llm]"
    );
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    Unit,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::List(items) => {
                let s: Vec<String> = items.iter().map(|i| i.to_string()).collect();
                write!(f, "[{}]", s.join(", "))
            }
            Value::Map(map) => {
                let mut keys: Vec<&String> = map.keys().collect();
                keys.sort();
                let pairs: Vec<String> = keys
                    .iter()
                    .map(|k| format!("{}: {}", k, map[*k]))
                    .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
            Value::Unit => write!(f, "()"),
        }
    }
}

enum ControlFlow {
    None,
    Return(Value),
    Break,
    Next,
}

struct Interpreter {
    vars: HashMap<String, Value>,
    functions: HashMap<String, FnDecl>,
    database: Arc<Mutex<HashMap<String, String>>>,
    cli_args: Vec<String>,
    gui_windows: HashMap<usize, gui::WindowState>,
}

impl Interpreter {
    fn new(functions: HashMap<String, FnDecl>, cli_args: Vec<String>) -> Self {
        Self {
            vars: HashMap::new(),
            functions,
            database: Arc::new(Mutex::new(HashMap::new())),
            cli_args,
            gui_windows: HashMap::new(),
        }
    }

    fn handle_http_request(
        &mut self,
        mut stream: std::net::TcpStream,
        handler_fn: &FnDecl,
    ) -> Result<(), String> {
        let mut buffer = Vec::new();
        let mut temp = [0u8; 1024];
        let mut header_end = None;
        let mut content_length = 0usize;

        loop {
            let n = stream.read(&mut temp).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            buffer.extend_from_slice(&temp[..n]);

            if header_end.is_none() {
                if let Some(pos) = buffer.windows(4).position(|w| w == b"\r\n\r\n") {
                    header_end = Some(pos + 4);
                    let header_str = String::from_utf8_lossy(&buffer[..pos]);
                    for line in header_str.lines().skip(1) {
                        let mut parts = line.splitn(2, ':');
                        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                            if k.trim().eq_ignore_ascii_case("content-length") {
                                content_length = v.trim().parse::<usize>().unwrap_or(0);
                            }
                        }
                    }
                }
            }

            if let Some(end) = header_end {
                let body_len = buffer.len() - end;
                if body_len >= content_length {
                    break;
                }
            }
        }

        if buffer.is_empty() {
            return Ok(());
        }

        let end = header_end.unwrap_or(buffer.len());
        let header_bytes = &buffer[..end.min(buffer.len())];
        let body_bytes = if end < buffer.len() { &buffer[end..] } else { &[] };

        let header_str = String::from_utf8_lossy(header_bytes);
        let mut lines = header_str.lines();
        let req_line = lines.next().unwrap_or("GET / HTTP/1.1");
        let mut parts = req_line.split_whitespace();
        let method = parts.next().unwrap_or("GET").to_string();
        let full_uri = parts.next().unwrap_or("/").to_string();

        let (path, query) = if let Some(q_pos) = full_uri.find('?') {
            (&full_uri[..q_pos], &full_uri[q_pos + 1..])
        } else {
            (full_uri.as_str(), "")
        };

        let mut headers_map = HashMap::new();
        for line in lines {
            let mut kv = line.splitn(2, ':');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                headers_map.insert(k.trim().to_lowercase(), Value::Str(v.trim().to_string()));
            }
        }

        let body_str = String::from_utf8_lossy(body_bytes).to_string();

        let previous_vars = self.vars.clone();
        self.vars.clear();

        let param_count = handler_fn.sig.params.len();
        if param_count == 1 {
            let p = &handler_fn.sig.params[0];
            if p.name == "path" {
                self.vars.insert(p.name.clone(), Value::Str(path.to_string()));
            } else {
                let mut req_map = HashMap::new();
                req_map.insert("method".to_string(), Value::Str(method));
                req_map.insert("path".to_string(), Value::Str(path.to_string()));
                req_map.insert("query".to_string(), Value::Str(query.to_string()));
                req_map.insert("body".to_string(), Value::Str(body_str));
                req_map.insert("headers".to_string(), Value::Map(headers_map));
                self.vars.insert(p.name.clone(), Value::Map(req_map));
            }
        } else if param_count == 2 {
            self.vars.insert(handler_fn.sig.params[0].name.clone(), Value::Str(method));
            self.vars.insert(handler_fn.sig.params[1].name.clone(), Value::Str(path.to_string()));
        } else if param_count >= 3 {
            self.vars.insert(handler_fn.sig.params[0].name.clone(), Value::Str(method));
            self.vars.insert(handler_fn.sig.params[1].name.clone(), Value::Str(path.to_string()));
            self.vars.insert(handler_fn.sig.params[2].name.clone(), Value::Str(body_str));
            if param_count >= 4 {
                self.vars.insert(handler_fn.sig.params[3].name.clone(), Value::Str(query.to_string()));
            }
        }

        let resp_res = self.run_fn(&handler_fn.body);
        self.vars = previous_vars;
        let resp_val = resp_res?;

        let (status_code, status_text, content_type, resp_body) = match &resp_val {
            Value::Map(m) => {
                let status = match m.get("status") {
                    Some(Value::Int(code)) => *code as u16,
                    _ => 200,
                };
                let text = match status {
                    200 => "OK",
                    201 => "Created",
                    202 => "Accepted",
                    204 => "No Content",
                    400 => "Bad Request",
                    401 => "Unauthorized",
                    403 => "Forbidden",
                    404 => "Not Found",
                    405 => "Method Not Allowed",
                    500 => "Internal Server Error",
                    _ => "OK",
                };
                let ct = match m.get("content_type") {
                    Some(Value::Str(s)) => s.clone(),
                    _ => "application/json; charset=utf-8".to_string(),
                };
                let body = match m.get("body") {
                    Some(Value::Str(s)) => s.clone(),
                    Some(v) => json::stringify(v),
                    None => "".to_string(),
                };
                (status, text, ct, body)
            }
            Value::Str(s) => {
                let is_json = s.trim_start().starts_with('{') || s.trim_start().starts_with('[');
                let ct = if is_json {
                    "application/json; charset=utf-8"
                } else {
                    "text/plain; charset=utf-8"
                };
                (200, "OK", ct.to_string(), s.clone())
            }
            other => (200, "OK", "text/plain; charset=utf-8".to_string(), other.to_string()),
        };

        let response = format!(
            "HTTP/1.1 {status_code} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{resp_body}",
            resp_body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        Ok(())
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Literal { lit, .. } => match lit {
                Literal::Int(v) => Ok(Value::Int(*v)),
                Literal::Float(v) => Ok(Value::Float(*v)),
                Literal::Bool(v) => Ok(Value::Bool(*v)),
                Literal::Str(s) => Ok(Value::Str(s.clone())),
                Literal::None => Ok(Value::Unit),
            },
            Expr::Ident { name, .. } => self
                .vars
                .get(name)
                .cloned()
                .ok_or_else(|| format!("undefined identifier '{name}'")),
            Expr::Unary { op, expr, .. } => {
                let v = self.eval_expr(expr)?;
                match (op, v) {
                    (UnaryOp::Neg, Value::Int(i)) => Ok(Value::Int(-i)),
                    (UnaryOp::Neg, Value::Float(f)) => Ok(Value::Float(-f)),
                    (UnaryOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                    _ => Err("unsupported unary operand type".into()),
                }
            }
            Expr::Binary { op, lhs, rhs, .. } => {
                let l = self.eval_expr(lhs)?;
                let r = self.eval_expr(rhs)?;
                match (op, l, r) {
                    (BinOp::Add, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
                    (BinOp::Add, Value::Str(a), Value::Str(b)) => Ok(Value::Str(format!("{a}{b}"))),
                    (BinOp::Add, Value::List(mut a), Value::List(b)) => {
                        a.extend(b);
                        Ok(Value::List(a))
                    }
                    (BinOp::Sub, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
                    (BinOp::Mul, Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
                    (BinOp::Div, Value::Int(a), Value::Int(b)) => {
                        if b == 0 {
                            Err("division by zero".into())
                        } else {
                            Ok(Value::Int(a / b))
                        }
                    }
                    (BinOp::Rem, Value::Int(a), Value::Int(b)) => {
                        if b == 0 {
                            Err("remainder by zero".into())
                        } else {
                            Ok(Value::Int(a % b))
                        }
                    }
                    (BinOp::Eq, Value::List(a), Value::Int(b)) => Ok(Value::Bool((a.len() as i64) == b)),
                    (BinOp::Eq, Value::Int(a), Value::List(b)) => Ok(Value::Bool(a == (b.len() as i64))),
                    (BinOp::Ne, Value::List(a), Value::Int(b)) => Ok(Value::Bool((a.len() as i64) != b)),
                    (BinOp::Ne, Value::Int(a), Value::List(b)) => Ok(Value::Bool(a != (b.len() as i64))),
                    (BinOp::Gt, Value::List(a), Value::Int(b)) => Ok(Value::Bool((a.len() as i64) > b)),
                    (BinOp::Lt, Value::List(a), Value::Int(b)) => Ok(Value::Bool((a.len() as i64) < b)),
                    (BinOp::Ge, Value::List(a), Value::Int(b)) => Ok(Value::Bool((a.len() as i64) >= b)),
                    (BinOp::Le, Value::List(a), Value::Int(b)) => Ok(Value::Bool((a.len() as i64) <= b)),
                    (BinOp::Eq, a, b) => Ok(Value::Bool(a == b)),
                    (BinOp::Ne, a, b) => Ok(Value::Bool(a != b)),
                    (BinOp::Lt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
                    (BinOp::Le, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
                    (BinOp::Gt, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
                    (BinOp::Ge, Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
                    (BinOp::And, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                    (BinOp::Or, Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                    _ => Err("type mismatch for binary operation".into()),
                }
            }
            Expr::Call { callee, args, .. } => {
                // Builtin calls
                if let Expr::Field { base, field, .. } = &**callee {
                    if let Expr::Ident { name, .. } = &**base {
                        // --- io module ---
                        if name == "io" && field == "println" {
                            let mut rendered = Vec::new();
                            for arg in args {
                                rendered.push(self.eval_expr(arg)?.to_string());
                            }
                            println!("{}", rendered.join(" "));
                            return Ok(Value::Unit);
                        }

                        // --- str module ---
                        if name == "str" && field == "fmt" {
                            if let Some(first) = args.first() {
                                let template = self.eval_expr(first)?.to_string();
                                let mut result = template;
                                for arg in args.iter().skip(1) {
                                    let v = self.eval_expr(arg)?.to_string();
                                    result = result.replacen("{}", &v, 1);
                                }
                                return Ok(Value::Str(result));
                            }
                        }

                        // --- db module (Zero-dependency Embedded Storage) ---
                        if name == "db" && field == "set" {
                            if args.len() < 2 {
                                return Err("db.set expects (key, value)".into());
                            }
                            let key = self.eval_expr(&args[0])?.to_string();
                            let val = self.eval_expr(&args[1])?.to_string();
                            let mut db = self.database.lock().unwrap();
                            db.insert(key, val);
                            return Ok(Value::Unit);
                        }

                        if name == "db" && field == "get" {
                            if args.is_empty() {
                                return Err("db.get expects (key)".into());
                            }
                            let key = self.eval_expr(&args[0])?.to_string();
                            let db = self.database.lock().unwrap();
                            let val = db.get(&key).cloned().unwrap_or_else(|| "none".to_string());
                            return Ok(Value::Str(val));
                        }

                        if name == "db" && field == "has" {
                            if args.is_empty() {
                                return Err("db.has expects (key)".into());
                            }
                            let key = self.eval_expr(&args[0])?.to_string();
                            let db = self.database.lock().unwrap();
                            return Ok(Value::Bool(db.contains_key(&key)));
                        }

                        // --- http module (Native Builtin Web Server) ---
                        if name == "http" && field == "serve" {
                            if args.len() < 2 {
                                return Err("http.serve expects (port, handler_fn_name)".into());
                            }
                            let port = match self.eval_expr(&args[0])? {
                                Value::Int(p) => p,
                                _ => return Err("http port must be an integer".into()),
                            };
                            let handler_name = match self.eval_expr(&args[1])? {
                                Value::Str(s) => s,
                                _ => return Err("handler function name must be a string".into()),
                            };

                            let handler_fn = self
                                .functions
                                .get(&handler_name)
                                .cloned()
                                .ok_or_else(|| format!("handler function '{handler_name}' not found"))?;

                            let addr = format!("127.0.0.1:{port}");
                            let listener = TcpListener::bind(&addr)
                                .map_err(|e| format!("failed to bind {addr}: {e}"))?;

                            println!("âš¡ Myelin HTTP server running at http://{addr}");
                            println!("âš¡ Test it in browser or: curl http://{addr}/status");
                            println!("âš¡ Press Ctrl+C to stop.");

                            for stream in listener.incoming() {
                                if let Ok(stream) = stream {
                                    let _ = self.handle_http_request(stream, &handler_fn);
                                }
                            }
                            return Ok(Value::Unit);
                        }

                        // --- json module (Pure & Deterministic JSON Runtime) ---
                        if name == "json" {
                            match field.as_str() {
                                "parse" => {
                                    if args.is_empty() {
                                        return Err("json.parse expects (json_str)".into());
                                    }
                                    let raw = self.eval_expr(&args[0])?.to_string();
                                    return json::parse(&raw).map_err(|e| format!("json.parse error: {e}"));
                                }
                                "stringify" => {
                                    if args.is_empty() {
                                        return Err("json.stringify expects (val)".into());
                                    }
                                    let v = self.eval_expr(&args[0])?;
                                    return Ok(Value::Str(json::stringify(&v)));
                                }
                                "get" => {
                                    if args.len() < 2 {
                                        return Err("json.get expects (doc, key_or_path)".into());
                                    }
                                    let doc = self.eval_expr(&args[0])?;
                                    let path = self.eval_expr(&args[1])?.to_string();
                                    return Ok(json::get_path(&doc, &path).unwrap_or(Value::Unit));
                                }
                                "set" => {
                                    if args.len() < 3 {
                                        return Err("json.set expects (doc, path, new_value)".into());
                                    }
                                    let mut doc = self.eval_expr(&args[0])?;
                                    let path = self.eval_expr(&args[1])?.to_string();
                                    let new_val = self.eval_expr(&args[2])?;
                                    json::set_path(&mut doc, &path, new_val)
                                        .map_err(|e| format!("json.set error: {e}"))?;
                                    return Ok(doc);
                                }
                                "has" => {
                                    if args.len() < 2 {
                                        return Err("json.has expects (doc, key)".into());
                                    }
                                    let doc = self.eval_expr(&args[0])?;
                                    let key = self.eval_expr(&args[1])?.to_string();
                                    let found = match &doc {
                                        Value::Map(m) => m.contains_key(&key),
                                        _ => json::get_path(&doc, &key).is_some(),
                                    };
                                    return Ok(Value::Bool(found));
                                }
                                _ => return Err(format!("unknown json function '{field}'")),
                            }
                        }

                        // --- gui module (Native Desktop Windowing & Dialogs) ---
                        if name == "gui" {
                            match field.as_str() {
                                "alert" => {
                                    let title = if args.len() > 1 {
                                        self.eval_expr(&args[0])?.to_string()
                                    } else {
                                        "Myelin".to_string()
                                    };
                                    let msg = if args.len() > 1 {
                                        self.eval_expr(&args[1])?.to_string()
                                    } else if let Some(first) = args.first() {
                                        self.eval_expr(first)?.to_string()
                                    } else {
                                        "".to_string()
                                    };
                                    gui::alert(&title, &msg);
                                    return Ok(Value::Unit);
                                }
                                "confirm" => {
                                    let title = if args.len() > 1 {
                                        self.eval_expr(&args[0])?.to_string()
                                    } else {
                                        "Myelin".to_string()
                                    };
                                    let msg = if args.len() > 1 {
                                        self.eval_expr(&args[1])?.to_string()
                                    } else if let Some(first) = args.first() {
                                        self.eval_expr(first)?.to_string()
                                    } else {
                                        "".to_string()
                                    };
                                    let result = gui::confirm(&title, &msg);
                                    return Ok(Value::Bool(result));
                                }
                                "window" => {
                                    if args.len() < 3 {
                                        return Err("gui.window expects (title, width, height)".into());
                                    }
                                    let title = self.eval_expr(&args[0])?.to_string();
                                    let width = match self.eval_expr(&args[1])? {
                                        Value::Int(w) => w as i32,
                                        _ => return Err("window width must be an integer".into()),
                                    };
                                    let height = match self.eval_expr(&args[2])? {
                                        Value::Int(h) => h as i32,
                                        _ => return Err("window height must be an integer".into()),
                                    };

                                    let win_id = gui::next_window_id();
                                    let win_state = gui::WindowState {
                                        id: win_id,
                                        title,
                                        width,
                                        height,
                                        buttons: Vec::new(),
                                        texts: Vec::new(),
                                    };
                                    self.gui_windows.insert(win_id, win_state);
                                    return Ok(Value::Int(win_id as i64));
                                }
                                "button" => {
                                    if args.len() < 7 {
                                        return Err("gui.button expects (win, text, x, y, w, h, on_click_fn)".into());
                                    }
                                    let win_id = match self.eval_expr(&args[0])? {
                                        Value::Int(id) => id as usize,
                                        _ => return Err("window handle must be an integer".into()),
                                    };
                                    let text = self.eval_expr(&args[1])?.to_string();
                                    let x = match self.eval_expr(&args[2])? {
                                        Value::Int(v) => v as i32,
                                        _ => 0,
                                    };
                                    let y = match self.eval_expr(&args[3])? {
                                        Value::Int(v) => v as i32,
                                        _ => 0,
                                    };
                                    let w = match self.eval_expr(&args[4])? {
                                        Value::Int(v) => v as i32,
                                        _ => 80,
                                    };
                                    let h = match self.eval_expr(&args[5])? {
                                        Value::Int(v) => v as i32,
                                        _ => 30,
                                    };
                                    let callback = self.eval_expr(&args[6])?.to_string();

                                    let win = self.gui_windows.get_mut(&win_id)
                                        .ok_or_else(|| format!("window id {win_id} not found"))?;
                                    let btn_id = 100 + win.buttons.len();
                                    win.buttons.push(gui::ButtonControl {
                                        id: btn_id,
                                        text,
                                        x,
                                        y,
                                        w,
                                        h,
                                        callback,
                                    });
                                    return Ok(Value::Unit);
                                }
                                "text" => {
                                    if args.len() < 6 {
                                        return Err("gui.text expects (win, content, x, y, w, h)".into());
                                    }
                                    let win_id = match self.eval_expr(&args[0])? {
                                        Value::Int(id) => id as usize,
                                        _ => return Err("window handle must be an integer".into()),
                                    };
                                    let text = self.eval_expr(&args[1])?.to_string();
                                    let x = match self.eval_expr(&args[2])? {
                                        Value::Int(v) => v as i32,
                                        _ => 0,
                                    };
                                    let y = match self.eval_expr(&args[3])? {
                                        Value::Int(v) => v as i32,
                                        _ => 0,
                                    };
                                    let w = match self.eval_expr(&args[4])? {
                                        Value::Int(v) => v as i32,
                                        _ => 100,
                                    };
                                    let h = match self.eval_expr(&args[5])? {
                                        Value::Int(v) => v as i32,
                                        _ => 20,
                                    };

                                    let win = self.gui_windows.get_mut(&win_id)
                                        .ok_or_else(|| format!("window id {win_id} not found"))?;
                                    win.texts.push(gui::TextControl {
                                        text,
                                        x,
                                        y,
                                        w,
                                        h,
                                    });
                                    return Ok(Value::Unit);
                                }
                                "show" => {
                                    if args.is_empty() {
                                        return Err("gui.show expects (win)".into());
                                    }
                                    let win_id = match self.eval_expr(&args[0])? {
                                        Value::Int(id) => id as usize,
                                        _ => return Err("window handle must be an integer".into()),
                                    };
                                    let win = self.gui_windows.get(&win_id)
                                        .cloned()
                                        .ok_or_else(|| format!("window id {win_id} not found"))?;

                                    let fns = self.functions.clone();
                                    let db = self.database.clone();
                                    gui::show_window_loop(&win, |cb_name| {
                                        if let Some(target_fn) = fns.get(cb_name).cloned() {
                                            let mut sub_interp = Interpreter::new(fns.clone(), Vec::new());
                                            sub_interp.database = db.clone();
                                            sub_interp.run_fn(&target_fn.body)?;
                                        }
                                        Ok(())
                                    })?;
                                    return Ok(Value::Unit);
                                }
                                _ => return Err(format!("unknown gui function '{field}'")),
                            }
                        }

                        // --- term module (ANSI Styling & Colors) ---
                        if name == "term" {
                            let mut rendered = Vec::new();
                            for arg in args {
                                rendered.push(self.eval_expr(arg)?.to_string());
                            }
                            let text = rendered.join(" ");
                            let styled = match field.as_str() {
                                "green" => format!("\x1b[32m{text}\x1b[0m"),
                                "red" => format!("\x1b[31m{text}\x1b[0m"),
                                "cyan" => format!("\x1b[36m{text}\x1b[0m"),
                                "bold" => format!("\x1b[1m{text}\x1b[0m"),
                                "yellow" => format!("\x1b[33m{text}\x1b[0m"),
                                "blue" => format!("\x1b[34m{text}\x1b[0m"),
                                "magenta" => format!("\x1b[35m{text}\x1b[0m"),
                                "gray" | "dim" => format!("\x1b[90m{text}\x1b[0m"),
                                "reset" => "\x1b[0m".to_string(),
                                _ => return Err(format!("unknown term function '{field}'")),
                            };
                            return Ok(Value::Str(styled));
                        }

                        // --- cli module (Command-Line Arguments & Flags) ---
                        if name == "cli" {
                            match field.as_str() {
                                "args" => {
                                    if let Some(first) = args.first() {
                                        let v = self.eval_expr(first)?;
                                        match v {
                                            Value::Str(s) if s == "count" || s == "len" => {
                                                return Ok(Value::Int(self.cli_args.len() as i64));
                                            }
                                            Value::Int(idx) => {
                                                let u = idx as usize;
                                                if idx >= 0 && u < self.cli_args.len() {
                                                    return Ok(Value::Str(self.cli_args[u].clone()));
                                                } else {
                                                    return Err(format!(
                                                        "arg index {idx} out of range (total {})",
                                                        self.cli_args.len()
                                                    ));
                                                }
                                            }
                                            _ => {}
                                        }
                                    }
                                    let items = self
                                        .cli_args
                                        .iter()
                                        .map(|s| Value::Str(s.clone()))
                                        .collect();
                                    return Ok(Value::List(items));
                                }
                                "count" | "arg_count" => {
                                    return Ok(Value::Int(self.cli_args.len() as i64));
                                }
                                "flag_bool" | "has_flag" => {
                                    if args.is_empty() {
                                        return Err("cli.flag_bool expects flag name".into());
                                    }
                                    let flag = self.eval_expr(&args[0])?.to_string();
                                    let found = self.cli_args.iter().any(|a| a == &flag);
                                    return Ok(Value::Bool(found));
                                }
                                "flag_str" => {
                                    if args.is_empty() {
                                        return Err("cli.flag_str expects flag name".into());
                                    }
                                    let flag = self.eval_expr(&args[0])?.to_string();
                                    let default = if args.len() > 1 {
                                        self.eval_expr(&args[1])?.to_string()
                                    } else {
                                        String::new()
                                    };
                                    let val = self
                                        .cli_args
                                        .iter()
                                        .position(|a| a == &flag)
                                        .and_then(|idx| self.cli_args.get(idx + 1))
                                        .cloned()
                                        .unwrap_or(default);
                                    return Ok(Value::Str(val));
                                }
                                "flag_int" => {
                                    if args.is_empty() {
                                        return Err("cli.flag_int expects flag name".into());
                                    }
                                    let flag = self.eval_expr(&args[0])?.to_string();
                                    let default = if args.len() > 1 {
                                        match self.eval_expr(&args[1])? {
                                            Value::Int(i) => i,
                                            _ => return Err("default for flag_int must be an integer".into()),
                                        }
                                    } else {
                                        0
                                    };
                                    let val = self
                                        .cli_args
                                        .iter()
                                        .position(|a| a == &flag)
                                        .and_then(|idx| self.cli_args.get(idx + 1))
                                        .and_then(|s| s.parse::<i64>().ok())
                                        .unwrap_or(default);
                                    return Ok(Value::Int(val));
                                }
                                _ => return Err(format!("unknown cli function '{field}'")),
                            }
                        }
                    }
                }

                // User-defined function calls or top-level builtins
                if let Expr::Ident { name, .. } = &**callee {
                    if name == "len" {
                        if let Some(first) = args.first() {
                            let v = self.eval_expr(first)?;
                            match v {
                                Value::List(l) => return Ok(Value::Int(l.len() as i64)),
                                Value::Str(s) => return Ok(Value::Int(s.len() as i64)),
                                _ => return Err("len expects a list or string".into()),
                            }
                        }
                    }
                    if let Some(target_fn) = self.functions.get(name).cloned() {
                        let mut evaluated_args = Vec::new();
                        for a in args {
                            evaluated_args.push(self.eval_expr(a)?);
                        }

                        if evaluated_args.len() != target_fn.sig.params.len() {
                            return Err(format!(
                                "function '{name}' expected {} arguments, found {}",
                                target_fn.sig.params.len(),
                                evaluated_args.len()
                            ));
                        }

                        let previous_vars = self.vars.clone();
                        self.vars.clear();

                        for (param, val) in target_fn.sig.params.iter().zip(evaluated_args) {
                            self.vars.insert(param.name.clone(), val);
                        }

                        let res = self.run_fn(&target_fn.body);
                        self.vars = previous_vars;
                        return res;
                    }
                }

                Err(format!("call to undefined symbol or unsupported method"))
            }
            Expr::Field { base, field, .. } => {
                if let Expr::Ident { name, .. } = &**base {
                    if name == "cli" {
                        match field.as_str() {
                            "args" => {
                                let list = self
                                    .cli_args
                                    .iter()
                                    .map(|s| Value::Str(s.clone()))
                                    .collect();
                                return Ok(Value::List(list));
                            }
                            "count" | "arg_count" => {
                                return Ok(Value::Int(self.cli_args.len() as i64));
                            }
                            _ => {}
                        }
                    }
                    if name == "term" && field == "reset" {
                        return Ok(Value::Str("\x1b[0m".to_string()));
                    }
                }

                let base_val = self.eval_expr(base)?;
                match (base_val, field.as_str()) {
                    (Value::List(items), "len" | "count") => Ok(Value::Int(items.len() as i64)),
                    (Value::Str(s), "len" | "count") => Ok(Value::Int(s.len() as i64)),
                    (Value::Map(map), "len" | "count") => Ok(Value::Int(map.len() as i64)),
                    (Value::Map(map), "keys") => {
                        let mut ks: Vec<Value> = map.keys().map(|k| Value::Str(k.clone())).collect();
                        ks.sort_by(|a, b| a.to_string().cmp(&b.to_string()));
                        Ok(Value::List(ks))
                    }
                    (Value::Map(map), "values") => {
                        let mut ks: Vec<&String> = map.keys().collect();
                        ks.sort();
                        let vs: Vec<Value> = ks.iter().map(|k| map[*k].clone()).collect();
                        Ok(Value::List(vs))
                    }
                    (Value::Map(map), f) => {
                        Ok(map.get(f).cloned().unwrap_or(Value::Unit))
                    }
                    (Value::List(items), "first") => {
                        items.first().cloned().ok_or_else(|| "list is empty".to_string())
                    }
                    (Value::List(items), "last") => {
                        items.last().cloned().ok_or_else(|| "list is empty".to_string())
                    }
                    (val, f) => Err(format!("field access '.{f}' not supported on {val}")),
                }
            }
            Expr::List { items, .. } => {
                let mut evaluated = Vec::new();
                for item in items {
                    evaluated.push(self.eval_expr(item)?);
                }
                Ok(Value::List(evaluated))
            }
            Expr::Index { base, index, .. } => {
                let b = self.eval_expr(base)?;
                let idx = self.eval_expr(index)?;
                match (b, idx) {
                    (Value::List(items), Value::Int(i)) => {
                        if i < 0 || (i as usize) >= items.len() {
                            Err(format!("index {i} out of bounds (len {})", items.len()))
                        } else {
                            Ok(items[i as usize].clone())
                        }
                    }
                    (Value::Str(s), Value::Int(i)) => {
                        if i < 0 || (i as usize) >= s.len() {
                            Err(format!("index {i} out of bounds (len {})", s.len()))
                        } else {
                            Ok(Value::Str(s.chars().nth(i as usize).unwrap().to_string()))
                        }
                    }
                    (Value::Map(map), Value::Str(k)) => {
                        Ok(map.get(&k).cloned().unwrap_or(Value::Unit))
                    }
                    _ => Err("index target must be a list, map, or string".into()),
                }
            }
            Expr::Try { expr, .. } => self.eval_expr(expr),
        }
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, String> {
        match stmt {
            Stmt::Bind { target, value, .. } => {
                let val = self.eval_expr(value)?;
                self.vars.insert(target.clone(), val);
                Ok(ControlFlow::None)
            }
            Stmt::Expr { expr, .. } => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::Return { value, .. } => {
                let val = if let Some(v) = value {
                    self.eval_expr(v)?
                } else {
                    Value::Unit
                };
                Ok(ControlFlow::Return(val))
            }
            Stmt::Guard { cond, otherwise, .. } => {
                let c = self.eval_expr(cond)?;
                if c != Value::Bool(true) {
                    if let Some(act) = otherwise {
                        match act {
                            GuardAction::Break => return Ok(ControlFlow::Break),
                            GuardAction::Next => return Ok(ControlFlow::Next),
                            GuardAction::Return(ret_expr) => {
                                let v = if let Some(re) = ret_expr {
                                    self.eval_expr(re)?
                                } else {
                                    Value::Unit
                                };
                                return Ok(ControlFlow::Return(v));
                            }
                            GuardAction::Fail(err_expr) => {
                                let err_val = self.eval_expr(err_expr)?;
                                return Err(format!("failed with error: {err_val}"));
                            }
                        }
                    } else {
                        return Err("assertion failed (guard evaluated to false)".into());
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Break { .. } => Ok(ControlFlow::Break),
            Stmt::Next { .. } => Ok(ControlFlow::Next),
            Stmt::If {
                branches,
                else_body,
                ..
            } => {
                let mut matched = false;
                for (cond_expr, body) in branches {
                    let cond_val = self.eval_expr(cond_expr)?;
                    if cond_val == Value::Bool(true) {
                        matched = true;
                        for s in body {
                            let flow = self.exec_stmt(s)?;
                            if !matches!(flow, ControlFlow::None) {
                                return Ok(flow);
                            }
                        }
                        break;
                    }
                }
                if !matched {
                    if let Some(el_stmts) = else_body {
                        for s in el_stmts {
                            let flow = self.exec_stmt(s)?;
                            if !matches!(flow, ControlFlow::None) {
                                return Ok(flow);
                            }
                        }
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::For {
                binding,
                iter,
                body,
                ..
            } => {
                let iter_val = self.eval_expr(iter)?;
                let items = match iter_val {
                    Value::List(l) => l,
                    _ => return Err("for loop target must be a list".into()),
                };
                for item in items {
                    self.vars.insert(binding.clone(), item);
                    let mut should_break = false;
                    for s in body {
                        let flow = self.exec_stmt(s)?;
                        match flow {
                            ControlFlow::Break => {
                                should_break = true;
                                break;
                            }
                            ControlFlow::Next => break,
                            ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                            ControlFlow::None => {}
                        }
                    }
                    if should_break {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::Loop { body, .. } => loop {
                let mut should_break = false;
                for s in body {
                    let flow = self.exec_stmt(s)?;
                    match flow {
                        ControlFlow::Break => {
                            should_break = true;
                            break;
                        }
                        ControlFlow::Next => {
                            break;
                        }
                        ControlFlow::Return(v) => return Ok(ControlFlow::Return(v)),
                        ControlFlow::None => {}
                    }
                }
                if should_break {
                    break Ok(ControlFlow::None);
                }
            },
            _ => Err("statement form not yet enabled in runtime".into()),
        }
    }

    fn run_fn(&mut self, stmts: &[Stmt]) -> Result<Value, String> {
        for stmt in stmts {
            let flow = self.exec_stmt(stmt)?;
            if let ControlFlow::Return(val) = flow {
                return Ok(val);
            }
        }
        Ok(Value::Unit)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_usage();
        return ExitCode::SUCCESS;
    }

    let llm_mode = args.iter().any(|a| a == "--llm");
    let positional: Vec<&str> = args
        .iter()
        .map(String::as_str)
        .filter(|a| !a.starts_with('-'))
        .collect();

    if positional.len() < 2 {
        print_usage();
        return ExitCode::from(2);
    }

    let cmd = positional[0];
    let path = positional[1];

    let raw = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            if llm_mode {
                println!("ERR E000 {path}:1:1 | io_error {err} | fix: verify source file path exists");
            } else {
                eprintln!("error: failed to read '{path}': {err}");
            }
            return ExitCode::FAILURE;
        }
    };

    let sf = SourceFile::new(path, raw);

    match cmd {
        "run" => {
            let t0 = Instant::now();
            let out = parse_source(&sf.src);
            if !out.diagnostics.is_empty() {
                emit_diagnostics(&sf, &out.diagnostics, llm_mode);
                return ExitCode::FAILURE;
            }

            let mut functions = HashMap::new();
            for item in &out.module.items {
                if let Item::Fn(f) = item {
                    functions.insert(f.name.clone(), f.clone());
                }
            }

            let Some(main_fn) = functions.get("main").cloned() else {
                eprintln!("error: no '#fn main []' found in module");
                return ExitCode::FAILURE;
            };

            let path_idx = args.iter().position(|a| a == path);
            let cli_user_args: Vec<String> = match path_idx {
                Some(idx) => args[idx + 1..]
                    .iter()
                    .filter(|a| *a != "--llm")
                    .cloned()
                    .collect(),
                None => Vec::new(),
            };
            let mut interp = Interpreter::new(functions, cli_user_args);
            if let Err(e) = interp.run_fn(&main_fn.body) {
                eprintln!("runtime error: {e}");
                return ExitCode::FAILURE;
            }

            let elapsed = t0.elapsed();
            if !llm_mode {
                eprintln!("-- finished in {} Âµs", elapsed.as_micros());
            }
            ExitCode::SUCCESS
        }
        "lex" => {
            let t0 = Instant::now();
            let out = tokenize(&sf.src);
            let elapsed = t0.elapsed();
            if !out.diagnostics.is_empty() {
                emit_diagnostics(&sf, &out.diagnostics, llm_mode);
                return ExitCode::FAILURE;
            }
            for tok in &out.tokens {
                let (line, col) = sf.line_col(tok.span.start);
                println!("{line:>3}:{col:<3} {:?}", tok.kind);
            }
            println!(
                "-- lexed {} tokens in {} Âµs",
                out.tokens.len(),
                elapsed.as_micros()
            );
            ExitCode::SUCCESS
        }
        "parse" => {
            let t0 = Instant::now();
            let out = parse_source(&sf.src);
            let elapsed = t0.elapsed();
            if !out.diagnostics.is_empty() {
                emit_diagnostics(&sf, &out.diagnostics, llm_mode);
                return ExitCode::FAILURE;
            }
            println!("{:#?}", out.module);
            println!(
                "-- parsed {} top-level items in {} Âµs",
                out.module.items.len(),
                elapsed.as_micros()
            );
            ExitCode::SUCCESS
        }
        "check" => {
            let t0 = Instant::now();
            let out = parse_source(&sf.src);
            let elapsed = t0.elapsed();
            if !out.diagnostics.is_empty() {
                emit_diagnostics(&sf, &out.diagnostics, llm_mode);
                return ExitCode::FAILURE;
            }
            let fn_count = out
                .module
                .items
                .iter()
                .filter(|i| matches!(i, Item::Fn(_)))
                .count();
            let test_count = out
                .module
                .items
                .iter()
                .filter(|i| matches!(i, Item::Test(_)))
                .count();
            if llm_mode {
                println!(
                    "OK {} | items={} fns={} tests={} | {}us",
                    sf.name,
                    out.module.items.len(),
                    fn_count,
                    test_count,
                    elapsed.as_micros()
                );
            } else {
                println!(
                    "OK: '{}' checked successfully ({} items, {} functions, {} tests) in {} Âµs.",
                    sf.name,
                    out.module.items.len(),
                    fn_count,
                    test_count,
                    elapsed.as_micros()
                );
            }
            ExitCode::SUCCESS
        }
        "audit" => audit::execute_audit(&sf, llm_mode),
        other => {
            eprintln!("unknown subcommand: '{other}'");
            print_usage();
            ExitCode::from(2)
        }
    }
}

fn emit_diagnostics(sf: &SourceFile, diags: &[myelin_diag::Diagnostic], llm_mode: bool) {
    for d in diags {
        if llm_mode {
            println!("{}", d.format_llm(sf));
        } else {
            eprintln!("{}\n", d.format_human(sf));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_term_and_cli_builtins() {
        let src = r#"
#mod main

#fn main [] !io !cli !term
  g = term.green "ok"
  r = term.red "err"
  c = term.cyan "info"
  b = term.bold "strong"
  args = cli.args
  cnt = args.len
  rel = cli.flag_bool "--release"
  target = cli.flag_str "--target" "native"
  workers = cli.flag_int "--workers" 4
  = cnt
"#;
        let out = parse_source(src);
        assert!(out.diagnostics.is_empty());
        let mut functions = HashMap::new();
        for item in &out.module.items {
            if let Item::Fn(f) = item {
                functions.insert(f.name.clone(), f.clone());
            }
        }
        let main_fn = functions.get("main").cloned().unwrap();
        let mut interp = Interpreter::new(
            functions,
            vec!["--release".to_string(), "--target".to_string(), "wasm".to_string()],
        );
        let ret = interp.run_fn(&main_fn.body).unwrap();
        assert_eq!(ret, Value::Int(3));
        assert_eq!(interp.vars.get("g").unwrap(), &Value::Str("\x1b[32mok\x1b[0m".into()));
        assert_eq!(interp.vars.get("r").unwrap(), &Value::Str("\x1b[31merr\x1b[0m".into()));
        assert_eq!(interp.vars.get("c").unwrap(), &Value::Str("\x1b[36minfo\x1b[0m".into()));
        assert_eq!(interp.vars.get("b").unwrap(), &Value::Str("\x1b[1mstrong\x1b[0m".into()));
        assert_eq!(interp.vars.get("rel").unwrap(), &Value::Bool(true));
        assert_eq!(interp.vars.get("target").unwrap(), &Value::Str("wasm".into()));
        assert_eq!(interp.vars.get("workers").unwrap(), &Value::Int(4));
    }
}

