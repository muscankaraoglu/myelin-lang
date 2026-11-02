//! `myelinc` — Command-line compiler & verification driver for Myelin (`.myl`).

use std::env;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use myelin_ast::Item;
use myelin_diag::SourceFile;
use myelin_lexer::tokenize;
use myelin_parser::parse_source;

fn print_usage() {
    eprintln!(
        "Myelin Compiler Toolchain (myelinc v0.1.0)\n\
         Usage:\n  \
           myelinc check <file.myl> [--llm]\n  \
           myelinc parse <file.myl> [--llm]\n  \
           myelinc lex   <file.myl> [--llm]"
    );
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
                println!("ERR E000 {path}:1:1 | io_error {err} | fix: verify file path exists");
            } else {
                eprintln!("error: failed to read '{path}': {err}");
            }
            return ExitCode::FAILURE;
        }
    };

    let sf = SourceFile::new(path, raw);

    match cmd {
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
                    "OK: '{}' parsed cleanly ({} items, {} functions, {} inline tests) in {} µs.",
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
            eprintln!("unknown subcommand '{other}'");
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
