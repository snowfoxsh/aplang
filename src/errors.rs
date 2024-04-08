use ariadne::Report;

pub fn display_errors(errors: Vec<Report>) {
    for error in errors {
        println!("{:?}\n", error)
    }
}
