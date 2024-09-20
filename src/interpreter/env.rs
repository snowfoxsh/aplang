use std::sync::Arc;
use miette::NamedSource;
use std::rc::Rc;
use std::collections::HashMap;
use crate::parser::ast::Variable;
use crate::interpreter::errors::RuntimeError;
use crate::interpreter::Value;
use crate::interpreter::procedure::{Callable, FunctionMap};
use crate::lexer::token::Token;

#[derive(Clone)]
pub struct Env {
    /// private functions
    pub functions: FunctionMap,

    /// public functions
    pub exports: FunctionMap,

    venv: Vec<Context>,
}

impl Env {
    /// used for creating function (or the base env)
    pub fn initialize_empty_scope(&mut self) {
        self.venv.push(Context::default())
    }

    /// creates a new block scope.
    /// used for something like a For Loop, or an If Stmt
    pub fn create_nested_layer(&mut self) {
        let enclosing = self.activate().clone();
        self.venv.push(enclosing)
    }

    /// replace the previous layer with the current layer.
    pub fn flatten_nested_layer(&mut self) {
        let context = self.scrape();

        *self.activate() = context;
    }

    /// pops off the current layer of the venv
    pub fn scrape(&mut self) -> Context {
        self.venv
            .pop()
            .expect("attempted to remove context but failed")
    }

    /// gets mutable ref to the current layer
    fn activate(&mut self) -> &mut Context {
        let len = self.venv.len();
        &mut self.venv[len - 1]
    }


    /// creates a variable with some value
    pub fn define(&mut self, variable: Arc<Variable>, value: Value) {
        // add the variable into the context
        self.activate()
            .variables
            .insert(variable.ident.clone(), (value, variable));
    }

    /// look up a variable based on the symbol
    pub fn lookup_name(&mut self, var: &str, tok: Token, file_path: String) -> Result<&(Value, Arc<Variable>), RuntimeError> {
        self.activate()
            .variables
            .get(var)
            .ok_or(
                RuntimeError {
                    named_source: NamedSource::new(file_path, tok.source.clone()),
                    span: tok.span,
                    message: "Invalid Variable".to_string(),
                    help: format!("Make sure to create the variable `{var}` before you use it"),
                    label: "Could not find variable".to_string()
                }
            )
    }


    /// looks up the variable by comparing the entire variable object
    pub fn lookup_var(&mut self, var: &Variable, file_path: String) -> Result<&Value, RuntimeError> {
        Ok(&self.lookup_name(var.ident.as_str(), var.token.clone(), file_path)?.0)
    }

    pub fn lookup_function(&self, function_name: String, tok: Token, file_path: String) -> Result<Rc<dyn Callable>, RuntimeError> {
        let (a, _b) = self.functions.get(&function_name).ok_or(
            RuntimeError {
                named_source: NamedSource::new(file_path, tok.source.clone()),
                span: tok.span,
                message: "Invalid PROCEDURE".to_string(),
                help: format!("Make sure to create the PROCEDURE `{function_name}` before you call it"),
                label: "This PROCEDURE doesn't exist".to_string()
            }
        )?.clone();
        Ok(a)
    }

    /// removes a variable
    pub fn remove(&mut self, variable: Arc<Variable>) -> Option<(Value, Arc<Variable>)> {
        self.activate().variables.remove(&variable.ident)
    }

    pub fn contains(&mut self, variable: Arc<Variable>) -> bool {
        self.activate().variables.contains_key(&variable.ident)
    }
}

impl Default for Env {
    fn default() -> Self {
        let mut env = Self { functions: Default::default(), exports: Default::default(), venv: vec![] };
        // push the base context layer into env so we don't panic
        env.initialize_empty_scope();
        env
    }

}

#[derive(Default, Clone, Debug)]
pub struct Context {
    variables: HashMap<String, (Value, Arc<Variable>)>,
    //              |^^^^^   |^^^       ^^^^^^^^|> Source code pointer
    //              |        |> Value of symbol
    //              |> Name of symbol
}

#[derive(Copy, Clone, Default)]
pub struct LoopControl {
    pub(crate) should_break: bool,
    pub(crate) should_continue: bool,
}