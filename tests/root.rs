use assert_cmd::Command;
use std::fmt::Write;

fn smart_test(smart_src: impl Into<String>) {
    let smart_src: String = smart_src.into();

    let mut expected = String::new();
    let mut src = String::new();
    for line in smart_src.lines() {
        let mut split = line.splitn(2, "$").collect::<Vec<_>>().into_iter();

        let src_line = split.next().unwrap();
        writeln!(&mut src, "{src_line}").unwrap();
        if let Some(out_line) = split.next() {
            // write!(&mut out, "{out_line}").unwrap();
            // writeln!(&mut src).unwrap();
            if out_line.ends_with(r"\") {
                let out_line = out_line.strip_suffix(r"\").unwrap();
                write!(&mut expected, "{out_line}").unwrap(); // newline not removed
            } else {
                writeln!(&mut expected, "{out_line}").unwrap(); // newline was removed
            }
        }
    }
    println!("=== SRC ===");
    println!("{src}");
    println!("=== EXP ===");
    println!("{expected}");

    let mut cmd = Command::cargo_bin("aplang").unwrap();

    let cmd = cmd.arg(format!("-e {src}"))
        .assert();

    println!("=== OUT ===");
    let runtime = String::from_utf8(cmd.get_output().stdout.clone()).unwrap();
    println!("{runtime}");

    // do assert
    cmd.stdout(expected);
}

#[test]
fn hello_world() {
    smart_test(r#"
DISPLAY("hello world") $hello world
DISPLAY_NOLN("hello world") $hello world\
    "#);
}

#[test]
fn test_var() {
    smart_test(r#"
pi <- 3.14
answer <- 42
greeting <- "hello world"


DISPLAY(pi)         $3.14
DISPLAY(answer)     $42
DISPLAY(greeting)   $hello world
    "#);
}