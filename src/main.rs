#![allow(dead_code, unused_variables)]

use std::{fs, io};
use std::io::Read;
use std::sync::Arc;
use std::time::Instant;
use clap::Parser;
use miette::{miette, Result};

use crate::aplang::ApLang;
use crate::arguments::{CommandLine, DebugMode};
use crate::errors::Reports;

mod ast;
mod errors;
mod interpreter;
mod lexer;
mod parser;
mod token;
mod aplang_std;
mod arguments;
mod aplang;

fn main() -> Result<()> {
    let args = CommandLine::parse();
    
    stacker::maybe_grow(1024 * 1024, args.stack_size, || {run(args) })
}

fn run(args: CommandLine) -> Result<()> {
    let mut debug_buffer = String::new();

    let mut file_name = "".to_string();

    // note: consider adding debug logs here
    // load the source code
    let source_code: Arc<str> = if let Some(file_path) = &args.file {
        if args.enforce_file_extension {
            let ext = file_path.extension()
                .ok_or(miette!("could not read file extension from file"))
                .map(|os_str| os_str.to_string_lossy().into_owned())?;

            if ext.eq_ignore_ascii_case(".ap") {
                return Err(miette!("file extension is not '.ap' instead found .{}", ext))
            }
        }

        file_name = file_path.file_name()
            .map(|os_str| os_str.to_string_lossy().into_owned())
            .ok_or(miette!("failed to read file name from file"))?;

        fs::read_to_string(file_path).map_err(|error| miette!(
           "failed to open file {:?}\n{}", file_path.as_path(), error
        ))?.into()
    } else if let Some(eval) = args.eval {
        eval
    } else if args.eval_stdin {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer).map_err(|error| miette!(
            "failed to read from stdin\n{}", error
        ))?;
        buffer.into()
    } else {
        unreachable!()
    };

    let aplang = ApLang::new(source_code, file_name);
    
    // execute the lexer
    let lexed = aplang.lex().map_err(Reports::from)?; // todo implement errors here
    
    // if the flag is enabled capture the debug info
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
    
    // stop if we are only checking
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
