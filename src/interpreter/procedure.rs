use crate::interpreter::errors::RuntimeError;
use crate::interpreter::{Interpreter, Value};
use crate::parser::ast::{ProcDeclaration, Stmt, Variable};
use miette::SourceSpan;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

pub type FunctionMap = HashMap<String, (Rc<dyn Callable>, Option<Arc<ProcDeclaration>>)>;
/*                            |^^^^^^  |^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^^^^^^^^^^|> Maybe pointer to function def
                              |        |                                                  If None: it is native function
                              |        |> Pointer to the function
                              |> Function name (symbol)
*/

pub trait Callable {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[Value],
        args_tokens: &[SourceSpan],
        source: Arc<str>,
    ) -> Result<Value, RuntimeError>;
    fn arity(&self) -> u8;
}

pub struct Procedure {
    pub name: String,
    pub params: Vec<Variable>,
    pub body: Stmt,
}

impl Callable for Procedure {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[Value],
        args_tokens: &[SourceSpan],
        source: Arc<str>,
    ) -> Result<Value, RuntimeError> {
        // save the return value
        let cached_return_value = interpreter.return_value.clone();

        // todo: consider allowing variables to be taken into context
        // ignore the global env
        interpreter.venv.initialize_empty_scope();

        // copy in the arguments
        // assigns them to their appropriate name parameter
        self.params
            .iter()
            .zip(args.iter().cloned())
            .for_each(|(param, arg)| interpreter.venv.define(Arc::new(param.clone()), arg));

        // execute the function
        interpreter.stmt(&self.body)?;

        let return_value = interpreter.return_value.clone();
        // todo implement backtrace
        interpreter.return_value = cached_return_value;

        // restore the previous env
        interpreter.venv.scrape();

        match return_value {
            None => Ok(Value::Null),
            Some(value) => Ok(value),
        }
    }

    fn arity(&self) -> u8 {
        self.params.len().try_into().unwrap()
    }
}

pub struct NativeProcedure {
    pub name: String,
    pub arity: u8,
    pub callable: fn(
        &mut Interpreter,
        &[Value],
        args_tokens: &[SourceSpan],
        source: Arc<str>,
    ) -> Result<Value, RuntimeError>,
}

impl Callable for NativeProcedure {
    fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &[Value],
        args_tokens: &[SourceSpan],
        source: Arc<str>,
    ) -> Result<Value, RuntimeError> {
        (self.callable)(interpreter, args, args_tokens, source)
    }

    fn arity(&self) -> u8 {
        self.arity
    }
}
