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

    let cmd = cmd.arg(format!("-e {src}")).assert();

    println!("=== OUT ===");
    let runtime = String::from_utf8(cmd.get_output().stdout.clone()).unwrap();
    println!("{runtime}");

    // do assert
    cmd.stdout(expected);
}

#[test]
fn test_hello_world() {
    smart_test(
        r#"
DISPLAY("hello world") $hello world
DISPLAY_NOLN("hello world") $hello world\
    "#,
    );
}

#[test]
fn test_arithmetic_expressions() {
    smart_test(
        r#"
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
    "#,
    );
}

#[test]
fn test_string_concatenation() {
    smart_test(
        r#"
    greeting <- "Hello, "
    name <- "World!"
    full_greeting <- greeting + name
    DISPLAY(full_greeting) $Hello, World!
    "#,
    );
}

#[test]
fn test_list_concatenation() {
    smart_test(
        r#"
    list1 <- [1, 2, 3]
    list2 <- [4, 5, 6]
    combined_list <- list1 + list2
    DISPLAY(combined_list) $[1, 2, 3, 4, 5, 6]
    "#,
    );
}

#[test]
fn test_function_call_expression() {
    smart_test(
        r#"
    PROCEDURE square(x) {
        RETURN x * x
    }
    result <- square(4)
    DISPLAY(result) $16

    DISPLAY(square(2 + 8)) $100
    "#,
    );
}

#[test]
fn test_logical_expressions() {
    smart_test(
        r#"
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
    "#,
    );
}

#[test]
fn test_comparison_expressions() {
    smart_test(
        r#"
    x <- 5
    y <- 10

    DISPLAY(x == y) $FALSE
    DISPLAY(x != y) $TRUE
    DISPLAY(x > y) $FALSE
    DISPLAY(x < y) $TRUE
    DISPLAY(x >= y) $FALSE
    DISPLAY(x <= y) $TRUE
    "#,
    );
}

#[test]
fn test_combined_expressions() {
    smart_test(
        r#"
    a <- 2
    b <- 3
    c <- 4

    // Parentheses control the order of operations:
    result <- (a + b) * c
    DISPLAY(result) $20
    "#,
    );
}

#[test]
fn test_var() {
    smart_test(
        r#"
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
    "#,
    );
}

#[test]
fn test_if() {
    smart_test(
        r#"
    x <- 10
    y <- 5
    IF (x > y) {
        DISPLAY("x is greater than y") $x is greater than y
    } ELSE {
        DISPLAY("y is greater than or equal to x")
    }

    IF (x == 10) {
        DISPLAY("x is exactly 10") $x is exactly 10
    }

    IF (y < 0) {
        DISPLAY("y is negative")
    } ELSE {
        DISPLAY("y is non-negative") $y is non-negative
    }
    "#,
    );
}

#[test]
fn test_if_else_false() {
    smart_test(
        r#"
    IF (FALSE) {
        DISPLAY("wrong")
    } ELSE {
        DISPLAY("right") $right
    }
    "#,
    );
}

#[test]
fn test_nested_if() {
    smart_test(
        r#"
    num <- 42
    IF (num > 0) {
        DISPLAY("num is positive") $num is positive
        IF (num MOD 2 == 0) {
            DISPLAY("num is even") $num is even
        } ELSE {
            DISPLAY("num is odd")
        }
    } ELSE {
        DISPLAY("num is non-positive")
    }
    "#,
    );
}

#[test]
fn test_if_no_else_true() {
    smart_test(
        r#"
    flag <- TRUE
    IF (flag) {
        DISPLAY("Flag is true") $Flag is true
    }
    "#,
    );
}

#[test]
fn test_multiple_if() {
    smart_test(
        r#"
    value <- 7
    IF (value < 10) {
        DISPLAY("value is less than 10") $value is less than 10
    }
    IF (value > 5) {
        DISPLAY("value is greater than 5") $value is greater than 5
    }
    IF (value == 7) {
        DISPLAY("value is equal to 7") $value is equal to 7
    }
    "#,
    );
}

#[test]
fn test_if_else_nested() {
    smart_test(
        r#"
    score <- 85
    IF (score >= 90) {
        DISPLAY("Grade A")
    } ELSE {
        IF (score >= 80) {
            DISPLAY("Grade B") $Grade B
        } ELSE {
            DISPLAY("Grade C or lower")
        }
    }
    "#,
    );
}

#[test]
fn test_else_if() {
    smart_test(
        r#"
    num <- 15
    IF (num < 10) {
        DISPLAY("num is less than 10")
    } ELSE IF (num < 20) {
        DISPLAY("num is between 10 and 20") $num is between 10 and 20
    } ELSE {
        DISPLAY("num is 20 or more")
    }
    "#,
    );
}

#[test]
fn test_else_if_grade_evaluation() {
    smart_test(
        r#"
    score <- 85
    IF (score >= 90) {
        DISPLAY("Grade A")
    } ELSE IF (score >= 80) {
        DISPLAY("Grade B") $Grade B
    } ELSE IF (score >= 70) {
        DISPLAY("Grade C")
    } ELSE {
        DISPLAY("Grade D or F")
    }
    "#,
    );
}

#[test]
fn test_else_if_boundary_conditions() {
    smart_test(
        r#"
    temperature <- 0
    IF (temperature < 0) {
        DISPLAY("Below Freezing")
    } ELSE IF (temperature == 0) {
        DISPLAY("At Freezing Point") $At Freezing Point
    } ELSE IF (temperature > 0) {
        DISPLAY("Above Freezing")
    }
    "#,
    );
}

#[test]
fn test_else_if_boolean() {
    smart_test(
        r#"
    is_active <- FALSE
    IF (is_active) {
        DISPLAY("Active")
    } ELSE IF (NOT is_active) {
        DISPLAY("Inactive") $Inactive
    }
    "#,
    );
}

#[test]
fn test_procedure_add() {
    smart_test(
        r#"
    PROCEDURE add(a, b) {
        sum <- a + b
        RETURN sum
    }
    result <- add(3, 4)
    DISPLAY(result) $7
    "#,
    );
}

#[test]
fn test_nested_procedures() {
    smart_test(
        r#"
    PROCEDURE multiply(a, b) {
        product <- a * b
        RETURN product
    }

    PROCEDURE calculate_area(length, width) {
        area <- multiply(length, width)
        RETURN area
    }

    area <- calculate_area(5, 10)
    DISPLAY(area) $50
    "#,
    );
}

#[test]
fn test_pass_by_value() {
    smart_test(
        r#"
    PROCEDURE modify_number(n) {
        n <- n + 10
        DISPLAY(n) $15
    }

    num <- 5
    modify_number(num)
    DISPLAY(num) $5
    "#,
    );
}

#[test]
fn test_pass_by_reference() {
    smart_test(
        r#"
    PROCEDURE add_to_list(lst) {
        APPEND(lst, 4)
        DISPLAY(lst) $[1, 2, 3, 4]
    }

    my_list <- [1, 2, 3]
    add_to_list(my_list)
    DISPLAY(my_list) $[1, 2, 3, 4]
    "#,
    );
}

#[test]
fn test_procedure_no_return() {
    smart_test(
        r#"
    PROCEDURE greet(name) {
        DISPLAY("Hello, " + name + "!")
    }

    greet("Alice") $Hello, Alice!
    "#,
    );
}

#[test]
fn test_repeat_times_basic() {
    smart_test(
        r#"
    REPEAT 3 TIMES {
        DISPLAY("a")
    }
    $a
    $a
    $a
    "#,
    );
}

#[test]
fn test_repeat_times_counter() {
    smart_test(
        r#"
    counter <- 0
    REPEAT 5 TIMES {
        counter <- counter + 1
        DISPLAY(counter)
    }
    $1
    $2
    $3
    $4
    $5
    "#,
    );
}

#[test]
fn test_repeat_until_counter() {
    smart_test(
        r#"
    count <- 0
    REPEAT UNTIL (count == 5) {
        count <- count + 1
        DISPLAY(count)
    }
    $1
    $2
    $3
    $4
    $5
    "#,
    );
}

#[test]
fn test_repeat_until_balance() {
    smart_test(
        r#"
    balance <- 100
    goal <- 400
    REPEAT UNTIL (balance >= goal) {
        balance <- balance * 2
        DISPLAY(balance)
    }
    $200
    $400
    "#,
    );
}
