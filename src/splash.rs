use std::io;
use std::io::{Read};
use owo_colors::OwoColorize;

pub fn show_splash() -> io::Result<()> {
println!(r#"             _
  __ _ _ __ | | __ _ _ __   __ _
 / _` | '_ \| |/ _` | '_ \ / _` |
| (_| | |_) | | (_| | | | | (_| |
 \__,_| .__/|_|\__,_|_| |_|\__, |
      |_|                  |___/
"#);

println!(
r#"{welcome}

{info}
    Aplang is a command line (CLI) tool - it must be run from a terminal.

    For more information on how use aplang please see the {block_docs} page.

{tldr}
    To view the command line arguments run the command:
>_      {help_cmd}

    To execute a .ap file run the command:
>_      {run_cmd}

{links}
    - {block_home_page}        https://aplang.org
    - {block_docs}             https://aplang.org/docs
    - {block_github}           https://github.com/snowfoxsh/aplang
    - {block_vscode_ext} https://marketplace.visualstudio.com/items?itemName=aplang.aplang

{exit}"#,
    welcome="A programming language made to aid students who are taking AP Computer Science Principles".bold(),
    info="INFO".bold().cyan(),
    tldr="TLDR".bold().cyan(),
    links="LINKS".bold().cyan(),
    help_cmd="aplang -h".italic(),
    run_cmd="aplang myfile.ap".italic(),
    block_home_page="[home page]".bold(),
    block_docs="[docs]".bold(),
    block_github="[github]".bold(),
    block_vscode_ext="[vscode extension]".bold(),
    exit="(press enter to exit)".dimmed(),
);
    io::stdin().read_exact(&mut [0])?;
    Ok(())
}