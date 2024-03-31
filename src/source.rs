// struct Source {
//     src: String,
//     semis: Vec<usize>,
// }
//
// impl Source {
//     pub fn new(input: impl IntoString) -> Self {
//         let src: String = input.into();
//
//         Source {
//             src: input,
//             semis: vec![]
//         }
//     }
//
//     pub fn add_semi(&mut self) {
//         self.src.
//     }
// }

use std::rc::Rc;
use std::sync::Arc;
use miette::{MietteError, MietteSpanContents, SourceCode, SourceSpan, SpanContents};
use crate::scanner::Token;

// todo: make pretty display
#[derive(Debug)]
pub struct Source {
    pub tokens: Vec<Token>,
    raw: Arc<str>,
}

impl Source {
    pub fn new(tokens: Vec<Token>, raw: Arc<str>) -> Self {
        Self {
            tokens,
            raw
        }
    }
}

impl SourceCode for Source {
    fn read_span<'a>(&'a self, span: &SourceSpan, context_lines_before: usize, context_lines_after: usize) -> Result<Box<dyn SpanContents<'a> + 'a>, MietteError> {
        self.raw.read_span(span, context_lines_before, context_lines_after)
    }
}
