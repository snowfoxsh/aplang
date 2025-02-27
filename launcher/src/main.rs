//! handles launching aplang in info mode
//! made for the MSIX package

use std::env;
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let launcher_path = env::current_exe().unwrap();

    let launcher_dir = launcher_path.parent().unwrap();

    let aplang_path: PathBuf = launcher_dir.join("aplang.exe");

    Command::new(aplang_path)
        .raw_arg(" --info")
        .spawn()
        .unwrap();
}
