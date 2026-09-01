//! CLI: run a file, type-check a file, or start a small REPL. No dependencies.
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        None => repl(),
        Some("--check") => match args.get(2) {
            Some(path) => check_file(path),
            None => {
                eprintln!("usage: esperanto --check <file.esp>");
                std::process::exit(2);
            }
        },
        Some("-h") | Some("--help") => {
            println!("esperanto <file.esp>     run a program");
            println!("esperanto --check <file> type-check without running");
            println!("esperanto                start the REPL");
        }
        Some(path) => run_file(path),
    }
}

fn run_file(path: &str) {
    let src = read_or_exit(path);
    match esperanto::run(&src) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn check_file(path: &str) {
    let src = read_or_exit(path);
    match esperanto::typecheck(&src) {
        Ok(()) => println!("ok: {path} type-checks"),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}

fn read_or_exit(path: &str) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        std::process::exit(2);
    })
}

fn repl() {
    println!(
        "esperanto REPL. each line is a full program (statements end with ';'). ctrl-d to quit."
    );
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().ok();
        let mut line = String::new();
        match stdin.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        match esperanto::run(line) {
            Ok(v) => println!("{v}"),
            Err(e) => println!("{e}"),
        }
    }
}
