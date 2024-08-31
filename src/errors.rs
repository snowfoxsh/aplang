use std::fmt::{Debug, Display};
use miette::{Diagnostic, LabeledSpan, Report, SourceCode, SourceSpan};
use thiserror::Error;
use std::fmt;
use std::sync::Arc;

#[derive(Error, Debug)]
#[error("error{} occurred", if reports.len() > 1 {"s"} else {""})]
pub struct Reports {
    reports : Vec<Report>
}

impl Diagnostic for Reports {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new("failure"))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(format!("See individual error{} for more details", if self.reports.len() > 1 {"s"} else {""} )))
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        None
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item=LabeledSpan> + '_>> {
        None
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item=&'a dyn Diagnostic> + 'a>> {
        Some(Box::new(self.reports.iter().map(|report| report.as_ref())))
    }
}

impl From<Vec<Report>> for Reports {
    fn from(value: Vec<Report>) -> Self {
        Self {reports: value}
    }
}

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
