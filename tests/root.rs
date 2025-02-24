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
fn test_arithmetic_expressions() {
    smart_test(r#"
    // Addition
    sum <- 5 + 3
    DISPLAY(sum) $8

    // Subtraction
    difference <- 10 - 4
    DISPLAY(difference) $6

    // Multiplication
    product <- 7 * 2
    DISPLAY(product) $14

    // Division
    quotient <- 8 / 2
    DISPLAY(quotient) $4

    // Modulo
    remainder <- 9 MOD 4
    DISPLAY(remainder) $1
    "#);
}


#[test]
fn test_string_concatenation() {
    smart_test(r#"
    greeting <- "Hello, "
    name <- "World!"
    full_greeting <- greeting + name
    DISPLAY(full_greeting) $Hello, World!
    "#);
}

#[test]
fn test_list_concatenation() {
    smart_test(r#"
    list1 <- [1, 2, 3]
    list2 <- [4, 5, 6]
    combined_list <- list1 + list2
    DISPLAY(combined_list) $[1, 2, 3, 4, 5, 6]
    "#);
}

#[test]
fn test_function_call_expression() {
    smart_test(r#"
    PROCEDURE square(x) {
        RETURN x * x
    }
    result <- square(4)
    DISPLAY(result) $16
    
    DISPLAY(square(2 + 8)) $100
    "#);
}

#[test]
fn test_logical_expressions() {
    smart_test(r#"
    a <- TRUE
    b <- FALSE

    // Logical NOT
    not_a <- NOT a
    DISPLAY(not_a) $FALSE

    // Logical AND
    and_result <- a AND b
    DISPLAY(and_result) $FALSE

    // Logical OR
    or_result <- a OR b
    DISPLAY(or_result) $TRUE
    "#);
}

#[test]
fn test_comparison_expressions() {
    smart_test(r#"
    x <- 5
    y <- 10

    DISPLAY(x == y) $FALSE
    DISPLAY(x != y) $TRUE
    DISPLAY(x > y) $FALSE
    DISPLAY(x < y) $TRUE
    DISPLAY(x >= y) $FALSE
    DISPLAY(x <= y) $TRUE
    "#);
}

#[test]
fn test_combined_expressions() {
    smart_test(r#"
    a <- 2
    b <- 3
    c <- 4

    // Parentheses control the order of operations:
    result <- (a + b) * c
    DISPLAY(result) $20
    "#);
}











#[test]
fn test_var() {
    smart_test(r#"
pi <- 3.14
answer <- 42
greeting <- "hello world"
yes <- TRUE
no <- FALSE
nothing <- NULL


DISPLAY(pi)         $3.14
DISPLAY(answer)     $42
DISPLAY(greeting)   $hello world
DISPLAY(yes)        $TRUE
DISPLAY(no)         $FALSE
DISPLAY(nothing)    $NULL
    "#);
}