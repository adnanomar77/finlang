use finlang_core::{
    bytecode::{compile as compile_bytecode, Vm},
    compiler::{canonical_debug, compile_source},
    state::FinancialState,
};
use std::{env, fs, process};

fn usage() {
    eprintln!("Usage: finlang <check|run|compile|format|test> <file.fin>");
}
fn source(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {}: {}", path, e);
        process::exit(2)
    })
}
fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_default();
    if command == "test" {
        let status = process::Command::new("cargo")
            .arg("test")
            .status()
            .expect("failed to invoke cargo");
        process::exit(status.code().unwrap_or(1));
    }
    let path = match args.next() {
        Some(p) => p,
        None => {
            usage();
            process::exit(2);
        }
    };
    let text = source(&path);
    match command.as_str() {
        "check" => match compile_source(&text) {
            Ok(c) => println!("OK: {} ({})", path, c.source_version),
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        },
        "compile" => match compile_source(&text) {
            Ok(c) => println!("{}", canonical_debug(&c)),
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        },
        "format" => {
            println!("{}", text.trim());
        }
        "run" => match compile_source(&text) {
            Ok(c) => {
                let mut state = FinancialState::new();
                state.set_balance("pool", 1_000_000);
                state.set_balance("alice", 0);
                state.set_balance("liquidation_pool", 0);
                state.set_balance("surplus_pool", 0);
                let mut runtime = Vm::new();
                runtime.set_oracle_value(finlang_core::ast::SourceId::FeedA, 150);
                runtime.set_oracle_value(finlang_core::ast::SourceId::FeedB, 200);
                let bytecode = compile_bytecode(&c.typed_ast);
                match runtime.execute(&bytecode, &mut state) {
                    Ok(v) => println!("result={:?}\nstate={:?}", v, state),
                    Err(e) => {
                        eprintln!("runtime: {}", e);
                        process::exit(1);
                    }
                }
            }
            Err(e) => {
                eprintln!("{}", e);
                process::exit(1);
            }
        },
        _ => {
            usage();
            process::exit(2);
        }
    }
}
