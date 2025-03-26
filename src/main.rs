#![allow(dead_code, unused_variables, clippy::module_inception)]
#[cfg(any(feature = "wasm", target_arch = "wasm32"))]
compile_error!(r#"
HALT! It seems like you are attempting to compile aplang into a binary with the "wasm" feature enabled. \
Do not do this. 
This features is ment for calling aplang from wasm ONLY. Please reinstall or recompile aplang without the wasm feature.
"#);


use clap::Parser;
use miette::{miette, Result};
use std::io;
use std::io::{ErrorKind, Read};
use std::time::Instant;
use cfg_if::cfg_if;
use crate::aplang::ApLang;
use crate::arguments::{CommandLine, DebugMode};
use interpreter::errors::Reports;

mod aplang;
mod arguments;
mod interpreter;
mod lexer;
mod parser;
mod standard_library;
mod output;

// this cannot compile, but it is here for clippy
#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "splash")]
mod splash;


fn main() -> Result<()> {
    let args = CommandLine::parse();
    
    cfg_if! {
        if #[cfg(feature = "portable")] {
            run(args)
        } else {
            stacker::maybe_grow(1024 * 1024, args.stack_size, || run(args))
        }
    }
}

fn run(args: CommandLine) -> Result<()> {
    let mut debug_buffer = String::new();

    #[cfg(feature = "splash")]
    if args.info {
        splash::show_splash().unwrap();
        return Ok(());
    }

    let aplang = if let Some(file_path) = args.file {
        ApLang::new_from_file(file_path.clone()).map_err(|err| match err.kind() {
            ErrorKind::NotFound => miette!("Could not read file {}", file_path.display()),
            other_err => miette!("Critical Failure. Could not read file! {} {err:?}", file_path.display()),
        })
    } else if let Some(eval) = args.eval {
        Ok(ApLang::new_from_stdin(eval))
    } else if args.eval_stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|err| miette!("failed to read from stdin\n{}", err))?;
        Ok(ApLang::new_from_stdin(buffer))
    } else {
        unreachable!()
    }?;

    // --- Lexer Stage ---
    let lex_result = aplang.lex();
    if matches!(args.debug, DebugMode::All | DebugMode::Lexer) {
        match &lex_result {
            Ok(lexed_val) => {
                if let Err(err) = lexed_val.debug_output(&mut debug_buffer) {
                    eprintln!("Error printing lexer debug info: {}", err);
                } else {
                    eprintln!("Lexer Debug Output:\n{}", debug_buffer);
                }
                debug_buffer.clear();
            }
            Err(e) => {
                eprintln!("Lexer stage encountered an error: {:?}", e);
                if !debug_buffer.is_empty() {
                    eprintln!("Lexer Debug Output (partial):\n{}", debug_buffer);
                }
            }
        }
    }
    let lexed = lex_result.map_err(Reports::from)?;

    // --- Parser Stage ---
    let parse_result = lexed.parse();
    if matches!(args.debug, DebugMode::All | DebugMode::Parser) {
        match &parse_result {
            Ok(parsed_val) => {
                if let Err(err) = parsed_val.debug_output(&mut debug_buffer) {
                    eprintln!("Error printing parser debug info: {}", err);
                } else {
                    eprintln!("Parser Debug Output:\n{}", debug_buffer);
                }
                debug_buffer.clear();
            }
            Err(e) => {
                eprintln!("Parser stage encountered an error: {:?}", e);
                if !debug_buffer.is_empty() {
                    eprintln!("Parser Debug Output (partial):\n{}", debug_buffer);
                }
            }
        }
    }
    let parsed = parse_result.map_err(Reports::from)?;

    // Stop if we're only checking
    if args.check {
        return Ok(());
    }

    // --- Interpreter Stage ---
    let runtime = if matches!(args.debug, DebugMode::All | DebugMode::Interpreter) {
        let start = Instant::now();
        let exec_result = parsed.execute_with_debug();
        if let Err(e) = &exec_result {
            eprintln!("Interpreter stage encountered an error: {}", e);
            if !debug_buffer.is_empty() {
                eprintln!("Interpreter Debug Output (partial):\n{}", debug_buffer);
            }
            // Propagate the error after printing debug info
            return exec_result.map(|_| ()).map_err(|e| e.into());
        }
        let executed = exec_result?;
        let elapsed = start.elapsed();
        if let Err(err) = executed.debug_output(&mut debug_buffer) {
            eprintln!("Error printing interpreter debug info: {}", err);
        } else {
            eprintln!("Interpreter Debug Output:\n{}", debug_buffer);
        }
        debug_buffer.clear();
        elapsed
    } else {
        let start = Instant::now();
        parsed.execute()?;
        start.elapsed()
    };

    if matches!(args.debug, DebugMode::Time | DebugMode::All) {
        eprintln!("EXECUTION TIME: {:?}", runtime);
    }

    Ok(())
}
