#![allow(dead_code, unused_variables)]

use std::io;
use std::io::{ErrorKind, Read};
use std::time::Instant;
use clap::Parser;
use miette::{miette, Result};

use crate::aplang::ApLang;
use crate::arguments::{CommandLine, DebugMode};
use interpreter::errors::Reports;

mod interpreter;
mod lexer;
mod parser;
mod standard_library;
mod arguments;
mod aplang;

fn main() -> Result<()> {
    let args = CommandLine::parse();
    
    stacker::maybe_grow(1024 * 1024, args.stack_size, || {run(args) })
}

fn run(args: CommandLine) -> Result<()> {
    let mut debug_buffer = String::new();

    let aplang = if let Some(file_path) = args.file {
        ApLang::new_from_file(file_path.clone()).map_err(|err| {
            match err.kind() {
                ErrorKind::NotFound => miette!(
                    "Could not read file {}", file_path.display(),
                ),
                other_err => miette!(
                    "Critical Failure. Could not read file! {} {err:?}", file_path.display(),
                ),
            }
        })
    } else if let Some(eval) = args.eval {
        Ok(ApLang::new_from_stdin(eval))
    } else if args.eval_stdin {
        let mut buffer = String::new();
        
        io::stdin().read_to_string(&mut buffer).map_err(|err| miette!(
            "failed to read from stdin\n{}", err
        ))?;
        Ok(ApLang::new_from_stdin(buffer))
    } else {
        unreachable!()
    };

    // execute the lexer
    let lexed = aplang?.lex().map_err(Reports::from)?; // todo implement errors here
    
    // if the flag is enabled, capture the debug info
    if matches!(args.debug, DebugMode::All | DebugMode::Lexer) {
        lexed.debug_output(&mut debug_buffer).map_err(|err| {
            miette!("could not write debug info for lexer!\n{}", err)
        })?
    } 
    
    // execute the parser
    let parsed = lexed
        .parse()
        .map_err(Reports::from)?; 
    
    // todo implement errors here
    if matches!(args.debug, DebugMode::All | DebugMode::Parser) {
        parsed.debug_output(&mut debug_buffer).map_err(|err| {
            miette!("could not write debug info for parser!\n{}", err)
        })?
    }
    
    // stop if we're only checking
    if args.check { return Ok(()) }
    
    // execute the interpreter
    let runtime = if matches!(args.debug, DebugMode::All | DebugMode::Interpreter) {
        let start = Instant::now();
        let executed = parsed.execute_with_debug()?;
        let elapsed = start.elapsed();
        executed.debug_output(&mut debug_buffer).map_err( |err| {
            miette!("could not write debug info for parser!\n{}", err)
        })?;
        
        elapsed
    } else {
        let start = Instant::now();
        parsed.execute()?;
        start.elapsed()
    };
    
    // todo: consider adding a flag that will specify a write location for the debug string
    // write out our debug buffer if requested
    if !matches!(args.debug, DebugMode::None) {
        eprintln!("{}", debug_buffer);
        if matches!(args.debug, DebugMode::Time | DebugMode::All) {
            eprintln!("EXECUTION TIME: {:?}", runtime)
        }
    }
    
    Ok(())
}
