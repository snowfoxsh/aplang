use rustyline::{DefaultEditor, Result};

const PROMPT: &str = ">> ";

pub fn run_repl(eval: fn(String)) -> Result<()> {
    let mut rl = DefaultEditor::new()?;

    loop {
        let line = rl.readline(PROMPT)?;
        eval(line);
    }
}
