use std::sync::Arc;


struct Ast {
    source: Arc<str>,
    
}

fn t() {
    let s: Arc<String> = "hello".to_string().into();
}