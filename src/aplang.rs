use crate::interpreter::errors::RuntimeError;
use crate::interpreter::FunctionMap;
use crate::interpreter::Interpreter;
use crate::interpreter::Value;
use crate::lexer::token::Token;
use crate::lexer::Lexer;
use crate::parser::ast::pretty::TreePrinter;
use crate::parser::ast::Ast;
use crate::parser::Parser2;
use miette::Report;
use std::fmt::Write;
use std::marker::PhantomData;
use std::path::PathBuf;
use std::sync::Arc;
use std::{fmt, fs, io};

pub struct Initialized;
pub struct Lexed;
pub struct Parsed;
pub struct Executed;
pub struct ExecutedWithDebug;

pub struct ApLang<State = Initialized> {
    source_code: Arc<str>,
    file_path: Option<PathBuf>,

    tokens: Option<Vec<Token>>, // generated with the lexer
    ast: Option<Ast>,           // generated with the parser
    values: Option<Vec<Value>>,

    _state: PhantomData<State>,
}

impl ApLang {
    pub fn new_from_file(file_path: PathBuf) -> io::Result<Self> {
        // check if the file exists
        let source_code: Arc<str> = fs::read_to_string(file_path.clone())?.into();

        Ok(Self {
            source_code,
            file_path: Some(file_path),

            tokens: None,
            ast: None,
            values: None,

            _state: PhantomData,
        })
    }

    pub fn new_from_stdin(source_code: impl Into<Arc<str>>) -> Self {
        ApLang {
            source_code: source_code.into(),
            file_path: None,
            tokens: None,
            ast: None,
            values: None,

            _state: PhantomData,
        }
    }

    // dont use this
    pub fn new(source_code: impl Into<Arc<str>>, file_path: Option<PathBuf>) -> Self {
        Self {
            source_code: source_code.into(),
            file_path,

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
        let file_name = self
            .file_path
            .clone()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let tokens = Lexer::scan(self.source_code.clone(), file_name)?;

        // move the data into the next state struct
        Ok(ApLang {
            source_code: self.source_code,
            file_path: self.file_path,
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

        let file_name = self
            .file_path
            .clone()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let mut parser = Parser2::new(tokens, Arc::clone(&self.source_code), file_name.as_str());

        let ast = parser.parse()?;

        Ok(ApLang {
            source_code: self.source_code,
            file_path: self.file_path,
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
    pub fn execute_as_module(self) -> Result<FunctionMap, RuntimeError> {
        Interpreter::new(unsafe { self.ast.unwrap_unchecked() }, self.file_path).interpret_module()
    }

    pub fn execute(self) -> Result<ApLang<Executed>, Report> {
        Interpreter::new(
            unsafe { self.ast.unwrap_unchecked() },
            self.file_path.clone(),
        )
        .interpret()
        .map_err(|err| {
            let named_source = err.named_source.clone();
            Report::from(err).with_source_code(named_source)
        })?;

        Ok(ApLang {
            source_code: self.source_code,
            file_path: self.file_path,
            tokens: None,
            ast: None,
            values: None,

            _state: PhantomData,
        })
    }

    pub fn execute_with_debug(self) -> Result<ApLang<ExecutedWithDebug>, Report> {
        let ast = unsafe { self.ast.unwrap_unchecked() };
        let mut interpreter = Interpreter::new(ast, self.file_path.clone());
        let values = interpreter.interpret_debug().map_err(|err| {
            let named_source = err.named_source.clone();
            Report::from(err).with_source_code(named_source)
        })?;

        Ok(ApLang {
            source_code: self.source_code,
            file_path: self.file_path,
            tokens: None,
            ast: None,
            values: Some(values),

            _state: PhantomData,
        })
    }

    pub fn debug_output<Writer: Write>(&self, buf: &mut Writer) -> fmt::Result {
        let ast = unsafe { self.ast.as_ref().unwrap_unchecked() };

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
