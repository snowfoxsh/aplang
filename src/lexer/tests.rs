use crate::lexer::lexer::Lexer;
use crate::lexer::token::LiteralValue;
use crate::lexer::token::TokenType::*;

#[test]
fn handle_one_char_tokens() {
    let source = "(( )) }{ []";
    let mut scanner = Lexer::new(source, String::default());
    scanner.scan_tokens().unwrap();

    assert_eq!(scanner.tokens.len(), 9);
    assert_eq!(scanner.tokens[0].token_type, LeftParen);
    assert_eq!(scanner.tokens[1].token_type, LeftParen);
    assert_eq!(scanner.tokens[2].token_type, RightParen);
    assert_eq!(scanner.tokens[3].token_type, RightParen);
    assert_eq!(scanner.tokens[4].token_type, RightBrace);
    assert_eq!(scanner.tokens[5].token_type, LeftBrace);
    assert_eq!(scanner.tokens[6].token_type, LeftBracket);
    assert_eq!(scanner.tokens[7].token_type, RightBracket);
    assert_eq!(scanner.tokens[8].token_type, Eof);
}

#[test]
fn handle_two_char_tokens() {
    let source = "<- != == >=";
    let mut scanner = Lexer::new(source, String::default());
    scanner.scan_tokens().unwrap();

    assert_eq!(scanner.tokens.len(), 5);
    assert_eq!(scanner.tokens[0].token_type, Arrow);
    assert_eq!(scanner.tokens[1].token_type, BangEqual);
    assert_eq!(scanner.tokens[2].token_type, EqualEqual);
    assert_eq!(scanner.tokens[3].token_type, GreaterEqual);
    assert_eq!(scanner.tokens[4].token_type, Eof);
}

#[test]
fn handle_string_lit() {
    let source = r#""ABC""#;
    let mut scanner = Lexer::new(source, String::default());
    scanner.scan_tokens().unwrap();
    assert_eq!(scanner.tokens.len(), 2);
    assert_eq!(scanner.tokens[0].token_type, StringLiteral);
    match scanner.tokens[0].literal.as_ref().unwrap() {
        LiteralValue::String(val) => assert_eq!(val, "ABC"),
        _ => panic!("Incorrect literal type"),
    }
}

#[test]
fn handle_string_lit_unterminated() {
    let source = r#""ABC"#;
    let mut scanner = Lexer::new(source, "".to_string());
    let result = scanner.scan_tokens();
    match result {
        Err(_) => (),
        _ => panic!("Should have failed"),
    }
}

#[test]
fn handle_string_lit_multiline() {
    let source = "\"ABC\ndef\"";
    let mut scanner = Lexer::new(source, String::default());
    scanner.scan_tokens().unwrap();
    assert_eq!(scanner.tokens.len(), 2);
    assert_eq!(scanner.tokens[0].token_type, StringLiteral);
    match scanner.tokens[0].literal.as_ref().unwrap() {
        LiteralValue::String(val) => assert_eq!(val, "ABC\ndef"),
        _ => panic!("Incorrect literal type"),
    }
}

#[test]
fn handle_number_literals() {
    let source = "123.123\n321.0\n5";
    let mut scanner = Lexer::new(source, String::default());
    scanner.scan_tokens().unwrap();

    assert_eq!(scanner.tokens.len(), 6);

    match scanner.tokens[0].literal {
        Some(LiteralValue::Number(val)) => assert_eq!(val, 123.123),
        _ => panic!("Incorrect literal type"),
    }
    assert_eq!(scanner.tokens[1].token_type, SoftSemi);
    match scanner.tokens[2].literal {
        Some(LiteralValue::Number(val)) => assert_eq!(val, 321.0),
        _ => panic!("Incorrect literal type"),
    }
    assert_eq!(scanner.tokens[3].token_type, SoftSemi);
    match scanner.tokens[4].literal {
        Some(LiteralValue::Number(val)) => assert_eq!(val, 5.0),
        _ => panic!("Incorrect literal type"),
    }
    assert_eq!(scanner.tokens[5].token_type, Eof)
}

#[test]
fn handle_keywords() {
    let keywords = vec![
        ("mod", Mod),
        ("if", If),
        ("else", Else),
        ("repeat", Repeat),
        ("times", Times),
        ("until", Until),
        ("for", For),
        ("each", Each),
        ("continue", Continue),
        ("break", Break),
        ("in", In),
        ("procedure", Procedure),
        ("return", Return),
        ("not", Not),
        ("and", And),
        ("or", Or),
        ("true", True),
        ("false", False),
        ("null", Null),
    ];

    for (keyword, token_type) in keywords {
        // Test lowercase version
        let mut scanner = Lexer::new(keyword, String::default());
        let result = scanner.scan_tokens().expect("Scanner failed on lowercase");
        assert_eq!(result.len(), 2, "Failed on keyword length: {}", keyword); // Expecting keyword token and EOF token
        assert_eq!(
            result[0].token_type, token_type,
            "Failed on lowercase keyword: {}",
            keyword
        );

        // Test uppercase version
        let upper_keyword = keyword.to_uppercase();
        let mut scanner_upper = Lexer::new(upper_keyword.to_owned(), String::default());
        let result_upper = scanner_upper
            .scan_tokens()
            .expect("Scanner failed on uppercase");
        assert_eq!(
            result_upper.len(),
            2,
            "Failed on keyword length: {}",
            upper_keyword
        ); // Expecting keyword token and EOF token
        assert_eq!(
            result_upper[0].token_type, token_type,
            "Failed on uppercase keyword: {}",
            upper_keyword
        );
    }
}

#[test]
fn handle_identifier() {
    let source = "this_is_a_3_var <- 12;";
    let mut scanner = Lexer::new(source, String::default());
    scanner.scan_tokens().unwrap();

    assert_eq!(scanner.tokens.len(), 5);

    assert_eq!(scanner.tokens[0].token_type, Identifier);
    assert_eq!(scanner.tokens[1].token_type, Arrow);
    assert_eq!(scanner.tokens[2].token_type, Number);
    assert_eq!(scanner.tokens[3].token_type, SoftSemi);
    assert_eq!(scanner.tokens[4].token_type, Eof);
}

#[test]
fn handle_implicit_semicolon() {
    let test_cases = vec![
        ("varName\n", true),    // Identifier ends with newline
        ("123\n", true),        // Number ends with newline
        ("\"string\"\n", true), // StringLiteral ends with newline
        (")\n", true),          // RightParen ends with newline
        ("]\n", true),          // RightBracket ends with newline
        ("}\n", true),          // RightBrace ends with newline
        ("+\n", false),         // Plus doesn't end a statement
        ("varName", false),     // No newline, no implicit semicolon
    ];

    for (source, should_have_semicolon) in test_cases {
        let mut scanner = Lexer::new(source, String::new());
        let result = scanner.scan_tokens().unwrap();

        let has_semicolon = result.iter().any(|token| token.token_type == SoftSemi);
        assert_eq!(
            has_semicolon, should_have_semicolon,
            "Failed on source: {}",
            source
        );
    }
}

#[test]
fn test_spans() {
    let input = "IF (a == 3) {\
            a <- a + 1\
            }";
}