mod interpreter;
mod value;
mod procedure;
mod env;
pub mod errors;

// used by ApLang
pub use interpreter::Interpreter;

// used by modules
pub use value::Value;
pub use procedure::FunctionMap;
#[allow(unused_imports)] // this is actually used in a macro
pub use procedure::NativeProcedure;