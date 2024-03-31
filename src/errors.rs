use miette::Report;

pub fn display_errors(errors: Vec<Report>, pretty: bool) {
    if pretty {
        for error in errors {
            println!("{:?}\n", error)
        }
    } else {
        for error in errors {
            println!("{}\n", error)
        }
    }
}
