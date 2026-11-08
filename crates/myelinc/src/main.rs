//! `myelinc` — Command-line compiler & verification driver for Myelin (`.myl`).

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

fn print_usage() {
    eprintln!(
        "Myelin Compiler Toolchain (myelinc v0.1.0)\n\
         Usage:\n  \
           myelinc run   <file.myl> [--llm]\n  \
           myelinc check <file.myl> [--llm]\n  \
           myelinc parse <file.myl> [--llm]\n  \
           myelinc lex   <file.myl> [--llm]"
    );
}

#[derive(Debug, Clone, PartialEq)]
enum Value {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Unit,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(i) => write!(f, "{i}"),
            Value::Float(fl) => write!(f, "{fl}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
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
}

impl Interpreter {
    fn new(functions: HashMap<String, FnDecl>) -> Self {
        Self {
            vars: HashMap::new(),
            functions,
            database: Arc::new(Mutex::new(HashMap::new())),
        }
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

                            println!("⚡ Myelin HTTP server running at http://{addr}");
                            println!("⚡ Test it in browser or: curl http://{addr}/status");
                            println!("⚡ Press Ctrl+C to stop.");

                            for stream in listener.incoming() {
                                if let Ok(mut stream) = stream {
                                    let mut buffer = [0; 2048];
                                    let _ = stream.read(&mut buffer);
                                    let req_str = String::from_utf8_lossy(&buffer);
                                    let path = req_str
                                        .lines()
                                        .next()
                                        .and_then(|line| line.split_whitespace().nth(1))
                                        .unwrap_or("/");

                                    // Call Myelin handler function with requested path
                                    let previous_vars = self.vars.clone();
                                    self.vars.clear();
                                    if let Some(p) = handler_fn.sig.params.first() {
                                        self.vars.insert(p.name.clone(), Value::Str(path.to_string()));
                                    }

                                    let resp_val = self.run_fn(&handler_fn.body)?;
                                    self.vars = previous_vars;

                                    let body = resp_val.to_string();
                                    let response = format!(
                                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                        body.len(),
                                        body
                                    );
                                    let _ = stream.write_all(response.as_bytes());
                                }
                            }
                            return Ok(Value::Unit);
                        }
                    }
                }

                // User-defined function calls
                if let Expr::Ident { name, .. } = &**callee {
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
            Expr::Try { expr, .. } => self.eval_expr(expr),
            _ => Err("unsupported expression".into()),
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

            let mut interp = Interpreter::new(functions);
            if let Err(e) = interp.run_fn(&main_fn.body) {
                eprintln!("runtime error: {e}");
                return ExitCode::FAILURE;
            }

            let elapsed = t0.elapsed();
            if !llm_mode {
                eprintln!("-- finished in {} µs", elapsed.as_micros());
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
                "-- lexed {} tokens in {} µs",
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
                "-- parsed {} top-level items in {} µs",
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
                    "OK: '{}' checked successfully ({} items, {} functions, {} tests) in {} µs.",
                    sf.name,
                    out.module.items.len(),
                    fn_count,
                    test_count,
                    elapsed.as_micros()
                );
            }
            ExitCode::SUCCESS
        }
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
