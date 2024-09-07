use std::{fmt};
use std::fmt::Write;
use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use miette::{NamedSource, Report};
use crate::ast::{Ast};
use crate::ast::pretty::TreePrinter;
use crate::errors::RuntimeError;
use crate::interpreter::{FunctionMap, Interpreter, Value};
use crate::lexer::Lexer;
use crate::parser::Parser2;
use crate::token::Token;


pub struct Initialized;
pub struct Lexed;
pub struct Parsed;
pub struct Executed;
pub struct ExecutedWithDebug;
pub struct Module;

pub struct ApLang<State = Initialized> {
    source_code: Arc<str>,
    file_name: String,

    tokens: Option<Vec<Token>>, // generated with the lexer
    ast: Option<Ast>, // generated with the parser
    values: Option<Vec<Value>>,

    _state: PhantomData<State>,
}

impl ApLang {
    pub fn new(source_code: impl Into<Arc<str>>, file_name: String) -> Self {
        ApLang {
            source_code: source_code.into(),
            file_name,
            tokens: None,
            ast: None,
            values: None,

            _state: PhantomData,
        }
    }
    pub fn get_source_code(&self) -> Arc<str> {
        self.source_code.clone()
    }
}

impl ApLang<Initialized> {

    /// executes the lexer to convert source code into tokens
    pub fn lex(self) -> Result<ApLang<Lexed>, Vec<Report>> {
        let tokens = Lexer::scan(self.source_code.clone(), self.file_name.clone())?;

        // move the data into the next state struct
        Ok(ApLang {
            source_code: self.source_code,
            file_name: self.file_name,
            tokens: Some(tokens), // tokens now exist
            ast: None,
            values: None,

            _state: PhantomData,
        })
    }
}

impl ApLang<Lexed> {
    pub fn parse(self) -> Result<ApLang<Parsed>, Vec<Report>> {
        // we know that tokens exist
        let tokens = unsafe { self.tokens.unwrap_unchecked() };

        let mut parser = Parser2::new(
            tokens,
            Arc::clone(&self.source_code),
            self.file_name.as_str()
        );

        let ast = parser.parse()?;

        Ok(ApLang {
            source_code: self.source_code,
            file_name: self.file_name,
            tokens: None,
            ast: Some(ast),
            values: None,

            _state: PhantomData,
        })
    }

    pub fn debug_output<Writer: Write>(&self, buf: &mut Writer) -> fmt::Result {
        for token in unsafe { self.tokens.as_ref().unwrap_unchecked() } {
            if token.is_soft_semi() {
                write!(buf, " ;")?;
            }

            write!(buf, "{}", token)?;
        }

        Ok(())
    }
}

impl ApLang<Parsed> {
    pub fn execute_as_module(self, file_name: String, source_code: Arc<str>) -> Result<FunctionMap, RuntimeError> {
        let named_source = NamedSource::new(self.file_name.clone(), self.source_code.clone());
        println!("Module Named Source: {:?}", named_source);
        Interpreter::new(unsafe { self.ast.unwrap_unchecked() }, "".into())
            .interpret_module()
    }

    pub fn execute(self) -> Result<ApLang<Executed>, Report> {
        let named_source = NamedSource::new(self.file_name.clone(), self.source_code.clone());
        Interpreter::new(unsafe { self.ast.unwrap_unchecked() }, "".into())
            .interpret()
            .map_err(|err| {
                let named_source = err.named_source.clone();
                Report::from(err).with_source_code(named_source)
            })?;

        Ok(ApLang {
            source_code: self.source_code,
            file_name: self.file_name,
            tokens: None,
            ast: None,
            values: None,

            _state: PhantomData,
        })
    }

    pub fn execute_with_debug(self) -> Result<ApLang<ExecutedWithDebug>, Report> {
        let ast = unsafe { self.ast.unwrap_unchecked() };
        let mut interpreter = Interpreter::new(ast, "".into());
        let values = interpreter
            .interpret_debug()
            .map_err(|err| {
                let named_source = err.named_source.clone();
                Report::from(err).with_source_code(named_source)
            })?;

        Ok(ApLang {
            source_code: self.source_code,
            file_name: self.file_name,
            tokens: None,
            ast: None,
            values: Some(values),

            _state: PhantomData,
        })
    }

    pub fn debug_output<Writer: Write>(&self, buf: &mut Writer) -> fmt::Result {
        let ast = unsafe { self.ast.as_ref().unwrap_unchecked()};

        write!(buf, "{}", ast.print_tree())
    }
}

impl ApLang<ExecutedWithDebug> {
    pub fn debug_output<Writer: Write>(&self, buf: &mut Writer) -> fmt::Result {
        let values = unsafe { self.values.as_ref().unwrap_unchecked() };
        
        for value in values {
            writeln!(buf, "EXPR OUTPUT: {}", value)?;
        }
        
        Ok(())
    }
}
