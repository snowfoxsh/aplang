use std::sync::Arc;
use ariadne::{Report, Source};
use crate::LReport;

pub fn display_errors(source: Arc<str>, errors: Vec<LReport>) {
    for error in errors {
        error.print(("file.ap", Source::from(source.clone()))).unwrap()
    }
}
