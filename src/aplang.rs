use std::{fmt};
use std::collections::HashMap;
use std::fmt::Write;
use std::marker::PhantomData;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use miette::Report;
use crate::ast::{Ast, ProcDeclaration};
use crate::ast::pretty::TreePrinter;
use crate::interpreter::{Callable, Interpreter, Value};
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

        // move the data into next state struct
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
    pub fn execute_as_module(self) -> Result<HashMap<String, (Rc<dyn Callable>, Option<Arc<ProcDeclaration>>)>, Report> {
        Interpreter::new(unsafe { self.ast.unwrap_unchecked() })
            .interpret_module()
            .map_err(|err|
                Report::from(err).with_source_code(self.source_code.clone())
            )
    }

    pub fn execute(self) -> Result<ApLang<Executed>, Report> {
        Interpreter::new(unsafe { self.ast.unwrap_unchecked() })
            .interpret()
            .map_err(|err|
                Report::from(err).with_source_code(self.source_code.clone())
            )?;

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
        let mut interpreter = Interpreter::new(ast);
        let values = interpreter
            .interpret_debug()
            .map_err(|err|
                Report::from(err).with_source_code(self.source_code.clone())
            )?;

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
