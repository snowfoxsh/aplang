use thiserror::Error;
use miette::{Diagnostic, SourceSpan, SourceCode, LabeledSpan, NamedSource};
use std::fmt::Display;
use std::sync::Arc;

#[derive(Debug, Error, Diagnostic)]
#[error("message {src}")]
#[diagnostic()]
#[diagnostic(code(aplang::runtime))]
pub struct RuntimeError {
    #[source_code] pub src: Arc<str>,
    #[label("{label}")] pub span: SourceSpan,
    pub message: String,
    pub help: String,
    pub label: String,
}