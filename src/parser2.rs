use crate::ast::{Ast, Expr, Literal, LogicalOp, Stmt};
use crate::lexer::LiteralValue;
use crate::token::TokenType::{Eof, LeftParen, RightParen};
use crate::token::{Token, TokenType};
use miette::{miette, Diagnostic, LabeledSpan, NamedSource, Report, Severity, SourceSpan};
use owo_colors::OwoColorize;
use std::fmt::Display;
use std::sync::Arc;
use thiserror::Error;

// something like
// self.consume(Semicolon, "Expected ';' after expression.")?;
// should have a diagnostic pointing to the before expression
// let previous

// add functionality miette mutilate that will insert x spaces before the error

// #[derive(Error, Diagnostic, Debug)]
// struct ExpectedError {
//     #[source_code]
//     src: NamedSource<Arc<str>>,
//
//
//     found: SourceSpan,
// }

use crate::token::TokenType::*;

pub struct Parser2 {
    tokens: Vec<Token>,
    source: Arc<str>,
    named_source: NamedSource<Arc<str>>,
    current: usize,
    warnings: Vec<Report>,
}

impl Parser2 {
    pub(crate) fn new(tokens: Vec<Token>, source: Arc<str>, file_name: &str) -> Self {
        Self {
            tokens,
            source: source.clone(),
            // named_source: NamedSource::new(file_name, source),
            named_source: NamedSource::new(format!("{}", file_name), source),
            current: 0,
            warnings: vec![],
        }
    }

    pub(crate) fn parse(&mut self) -> Result<Ast, Vec<Report>> {
        let mut statements = vec![];
        let mut reports = vec![];

        while !self.is_at_end() {
            if self.match_token(&SoftSemi) {
                continue;
            }

            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(report) => {
                    let report = report;
                    self.synchronize();
                    reports.push(report)
                }
            }
        }

        if !reports.is_empty() {
            return Err(reports);
        }

        Ok(Ast {
            source: self.source.clone(),
            program: statements,
        })
    }
}

/// parse expression
impl Parser2 {
    fn declaration(&mut self) -> miette::Result<Stmt> {
        if self.match_token(&Procedure) {
            let proc_token = self.previous().clone();

            return self.procedure(proc_token);
        }
        self.statement()
    }

    fn procedure(&mut self, proc_token: Token) -> miette::Result<Stmt> {
        let name_token = self
            .consume(&Identifier, |token| {
                let labels = vec![
                    LabeledSpan::at(proc_token.span(), "this procedure requires a name"),
                    LabeledSpan::at(token.span(), format!("name goes here")),
                ];

                miette!(
                    labels = labels,
                    code = "unnamed_procedure",
                    help = "name the PROCEDURE with an IDENT",
                    "expected `IDENT` found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        // self.ident_warning(&name_token);

        let name = name_token.lexeme.clone();

        let lp_token = self
            .consume(&LeftParen, |token| {
                // miette!(
                //     labels = vec![LabeledSpan::at(token.span, "kill yourself2")],
                //     "expected lp token, found {token}"
                // )
                let labels = vec![
                    LabeledSpan::at(token.span(), "expected a `(`"),
                    LabeledSpan::at(
                        name_token.span(),
                        format!("{} requires `(..)` argument list", name_token.lexeme),
                    ),
                ];

                miette!(
                    labels = labels,
                    code = "missing_lp",
                    help = "a PROCEDURE requires a argument list in `()` after its name",
                    "expected `(` found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        let (params, params_tokens) = if !self.check(&RightParen) {
            // parse shit
            // todo

            (vec![], vec![])
        } else {
            (vec![], vec![])
        };

        let rp_token = self
            .consume(&RightParen, |token| {
                let labels = vec![LabeledSpan::at(token.span(), "expected a `)`")];

                miette!(
                    labels = labels,
                    code = "missing_rp",
                    help = "mismatched `(`, it seems you missed a `)`.",
                    "expected `)`, found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        let body = self.statement()?.into();

        Ok(Stmt::ProcDeclaration {
            name,
            params,
            body,
            proc_token,
            name_token,
            params_tokens,
        })
    }

    fn statement(&mut self) -> miette::Result<Stmt> {
        // IF (condition)
        if self.match_token(&If) {
            let if_token = self.previous().clone();
            return self.if_statement(if_token);
        }

        // REPEAT UNTIL (condition)
        if self.match_token(&Repeat) {
            let repeat_token = self.previous().clone();
            // this is a repeat until block
            if self.check(&Until) {
                return self.repeat_until(repeat_token);
            }

            return self.repeat_times(repeat_token);
        }

        if self.match_token(&For) {
            let for_token = self.previous().clone();
            return self.for_each(for_token);
        }

        // { expr }
        if self.match_token(&LeftBrace) {
            let lb_token = self.previous().clone();

            return self.block(lb_token);
        }

        self.expression_statement()
    }

    fn block(&mut self, lb_token: Token) -> miette::Result<Stmt> {
        let mut statements = vec![];

        while !self.check(&LeftBrace) && !self.is_at_end() {
            // evil code that took 6 hours to figure out
            if self.match_token(&SoftSemi) {
                continue;
            }

            // evil code that took 6 hours to figure out
            if self.check(&RightBrace) {
                break;
            }

            statements.push(self.declaration()?);
        }

        let rb_token = self
            .consume(&RightBrace, |token| {
                let labels = vec![LabeledSpan::at(
                    lb_token.span(),
                    "this delimiter requires a closing `}`",
                )];
                // todo: span the next `}` token

                miette!(
                    labels = labels,
                    code = "missing_rb",
                    help = "mismatched `{`, it seems you missed a `}`",
                    "this block has an unclosed delimiter"
                )
            })?
            .clone();

        Ok(Stmt::Block {
            lb_token,
            statements,
            rb_token,
        })
    }

    fn if_statement(&mut self, if_token: Token) -> miette::Result<Stmt> {
        // todo: improve this report
        let lp_token = self
            .consume(&LeftParen, |token| {
                // miette!("expected lp_token")
                let labels = vec![
                    LabeledSpan::at(token.span(), "expected a `(`"),
                    LabeledSpan::at(if_token.span(), "IF requires `(..)` condition"),
                ];

                miette!(
                    labels = labels,
                    code = "missing_lp",
                    help = "an IF statement requires a condition in `()` after the `IF` keyword",
                    "expected `(` found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        let condition = self.expression()?;

        let rp_token = self
            .consume(&RightParen, |token| {
                // miette!("Expected `)` found {}", token)
                let labels = vec![LabeledSpan::at(token.span(), "expected a `)`")];

                miette!(
                    labels = labels,
                    code = "missing_rp",
                    help = "mismatched `(`, it seems you missed a `)`.",
                    "expected `)`, found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        let then_branch = self.statement()?.into();
        // let (else_branch, else_token) = if self.match_token(&Else) {
        //     let else_token = self.previous().clone();
        //     let else_branch = self.statement()?.into();
        //
        //     (Some(else_branch), Some(else_token))
        // } else { (None, None) };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch: None,
            if_token,
            else_token: None,
        })
    }

    fn repeat_times(&mut self, repeat_token: Token) -> miette::Result<Stmt> {
        // confirm that the repeat token was consumed
        self.confirm(&Repeat)?;

        // expected expression
        let count = self.expression()?;

        let times_token = self.consume(&Times, |token| {
            // todo improve this message
            // miette!("expected times token")
            let labels = vec![
                LabeledSpan::at(token.span(), "expected a `TIMES`"),
            ];

            miette!(
                labels = labels,
                code = "missing_times",
                help = "a REPEAT block requires a `TIMES` keyword after the number of times to repeat",
                "expected `TIMES` found `{}`", token.lexeme
            )
        })?.clone();

        let body = self.statement()?.into();

        Ok(Stmt::RepeatTimes {
            count,
            body,
            repeat_token,
            times_token,
        })
    }

    fn repeat_until(&mut self, repeat_token: Token) -> miette::Result<Stmt> {
        // confirm that the repeat token has been consumed
        self.confirm(&Repeat)?;

        let until_token = self
            .consume(&Until, |token| {
                // todo: improve this error
                // miette!(
                //     "expected until token after repeat token"
                // )
                // let labels = vec![
                //     LabeledSpan::at(token.span(), "expected an `UNTIL`"),
                // ];

                // miette!(
                //     labels = labels,
                //     code = "missing_times",
                //     help = "a REPEAT block requires an `UNTIL` keyword with a condition",
                //     "expected `UNTIL` found {}", token.lexeme
                // )
                // todo consider makeing this advance instad of consume
                // this should never error
                miette!("how tf do i trigger this")
            })?
            .clone();

        let lp_token = self.consume(&LeftParen, |token| {
            // todo: improve this error
            let labels = vec![
                LabeledSpan::at(token.span(), "expected a `(`"),
                LabeledSpan::at(until_token.span(), "REPEAT UNTIL requires `(..)` condition")
            ];

            miette!(
                labels = labels,
                code = "missing_lp",
                help = "a REPEAT UNTIL block requires a condition in `()` after the `UNTIL` keyword",
                "expected `(` found `{}`", token.lexeme
            )
        })?.clone();

        let condition = self.expression()?;

        let rp_token = self
            .consume(&RightParen, |token| {
                // todo: improve this error
                let labels = vec![LabeledSpan::at(token.span(), "expected a `)`")];

                miette!(
                    labels = labels,
                    code = "missing_rp",
                    help = "mismatched `(`, it seems you missed a `)`.",
                    "expected `)`, found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        let body = self.statement()?.into();

        Ok(Stmt::RepeatUntil {
            condition,
            body,
            repeat_token,
            until_token,
        })
    }

    fn for_each(&mut self, for_token: Token) -> miette::Result<Stmt> {
        self.confirm(&For)?;

        let each_token = self
            .consume(&Each, |token| {
                // todo improve this message
                // miette!("expected each token")
                let labels = vec![LabeledSpan::at(token.span(), "expected an `EACH`")];

                miette!(
                    labels = labels,
                    code = "missing_each",
                    help = "a FOR block requires an `EACH` keyword after the `FOR` keyword",
                    "expected `EACH` found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        let item_token = self
            .consume(&Identifier, |token| {
                // todo improve this message
                // miette!("expected an ident")
                let labels = vec![
                    LabeledSpan::at(each_token.span(), "expected an identifier after `EACH`"),
                    LabeledSpan::at(token.span(), "identifier goes here"),
                ];

                miette!(
                    labels = labels,
                    code = "missing_ident",
                    help = "a FOR EACH block requires an identifier after the `EACH` keyword",
                    "expected `IDENTIFIER` found `{}`",
                    token.lexeme
                )
            })?
            .clone();
        let item = item_token.lexeme.clone();

        let in_token = self
            .consume(&In, |token| {
                // miette!("expected in token")
                let labels = vec![
                    LabeledSpan::at(item_token.span(), "expected an `IN` after identifier"),
                    LabeledSpan::at(token.span(), "`IN` goes here"),
                ];

                miette!(
                    labels = labels,
                    code = "missing_in",
                    help = "a FOR EACH block requires an `IN` keyword after the identifier",
                    "expected `IN` found `{}`",
                    token.lexeme
                )
            })?
            .clone();

        let list = self.expression()?;

        let body = self.statement()?.into();

        Ok(Stmt::ForEach {
            item,
            list,
            body,
            item_token,
            for_token,
            each_token,
            in_token,
        })
    }

    fn expression_statement(&mut self) -> miette::Result<Stmt> {
        let expr = self.expression()?;
        if self.is_at_end() {
            return Ok(Stmt::Expr { expr });
        }

        if self.check(&RightBrace) {
            return Ok(Stmt::Expr { expr });
        }

        self.consume(&SoftSemi, |token| {
            // miette!("Expected EOL or semi found {}", token)
            let labels = vec![LabeledSpan::at(
                token.span(),
                "missing End Of Line indicator",
            )];

            miette!(
                labels = labels,
                code = "missing_eol",
                help = "try manually placing a semicolon",
                "expected `End Of Line` found `{}`",
                token.lexeme
            )
        })?;
        Ok(Stmt::Expr { expr })
    }

    pub(crate) fn expression(&mut self) -> miette::Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> miette::Result<Expr> {
        let expr = self.or()?;
        let expr_token = self.previous().clone();

        if self.match_token(&Arrow) {
            let arrow_token = self.previous().clone();
            let value = self.assignment()?; // Recursively parse the assignment value to handle chained assignments

            match expr {
                // Handling assignment to a simple variable
                Expr::Variable { ident, token } => Ok(Expr::Assign {
                    target: ident,
                    value: Box::new(value),
                    ident_token: token,
                    arrow_token,
                }),

                // Handling set assignment for complex expressions like array[index] = value
                Expr::Access {
                    list,
                    key,
                    brackets,
                } => Ok(Expr::Set {
                    target: Box::new(Expr::Access {
                        list,
                        key,
                        brackets,
                    }),
                    value: Box::new(value),
                    arrow_token,
                }),

                // Error for invalid assignment target
                // todo: add better error here
                // _ => Err({
                //     miette!("Invalid assignment target.")
                // })
                _ => {
                    let labels = vec![
                        LabeledSpan::at(arrow_token.span(), "expected an assignment target"),
                        LabeledSpan::at(expr_token.span(), "target goes here"),
                    ];

                    Err(miette!(
                        labels = labels,
                        code = "invalid_assignment_target",
                        help = "an assignment target must be a variable or an access expression (array[index] type)",
                        "expected an assignment target found {}", expr
                    ).with_source_code(self.named_source.clone()))
                }
            }
        } else {
            Ok(expr)
        }
    }

    // and ( "OR" and )*
    fn or(&mut self) -> miette::Result<Expr> {
        let mut expr = self.and()?;

        while self.match_token(&Or) {
            // get or token for spanning
            let token = self.previous().clone();

            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: LogicalOp::Or,
                right: Box::new(right),

                token,
            }
        }

        Ok(expr)
    }

    // logical_and -> equality ( "AND" equality )*
    fn and(&mut self) -> miette::Result<Expr> {
        let mut expr = self.equalitu()?;

        while self.match_token(&And) {
            // get the token for spanning
            let token = self.previous().clone();

            let right = self.and()?;
            expr = Expr::Logical {
                left: expr.into(),
                operator: LogicalOp::Or,
                right: right.into(),

                token,
            }
        }

        Ok(expr)
    }

    fn equalitu(&mut self) -> miette::Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[BangEqual, EqualEqual]) {
            // get equality token
            let token = self.previous().clone();
            let right = self.comparison()?.into();
            let operator = token.to_binary_op()?;
            let left = expr.into();
            expr = Expr::Binary {
                left,
                operator,
                right,
                token,
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> miette::Result<Expr> {
        let mut expr = self.addition()?;

        while self.match_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            // get comparision token
            let token = self.previous().clone();
            let right = self.addition()?.into();
            let operator = token.to_binary_op()?;
            let left = expr.into();
            expr = Expr::Binary {
                left,
                operator,
                right,
                token,
            }
        }

        Ok(expr)
    }

    fn addition(&mut self) -> miette::Result<Expr> {
        let mut expr = self.multiplication()?;

        while self.match_tokens(&[Plus, Minus]) {
            // get addition token
            let token = self.previous().clone();
            let right = self.multiplication()?.into();
            let operator = token.to_binary_op()?;
            let left = expr.into();
            expr = Expr::Binary {
                left,
                operator,
                right,
                token,
            }
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> miette::Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[Star, Slash]) {
            // get multiplication token
            let token = self.previous().clone();
            let right = self.unary()?.into();
            let operator = token.to_binary_op()?;
            let left = expr.into();

            expr = Expr::Binary {
                left,
                operator,
                right,
                token,
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> miette::Result<Expr> {
        if self.match_tokens(&[Not, Minus]) {
            let token = self.previous().clone();
            let right = self.unary()?.into();
            let operator = token.to_unary_op()?;

            let expr = Expr::Unary {
                operator,
                right,
                token,
            };

            Ok(expr)
        } else {
            self.access()
        }
    }

    fn access(&mut self) -> miette::Result<Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&LeftBracket) {
                let lb_token = self.previous().clone();

                let index = self.expression()?;
                let rb_token = self.consume(&RightBracket, |token| {
                    let labels = vec![
                        // todo: make expression span
                        // LabeledSpan::at(index.span(), "expression"),
                        LabeledSpan::at(token.span(), "requires closing `]`")
                    ];

                    miette!(
                        labels = labels,
                        code = "missing_rbracket",
                        help = "when indexing an array you must have a closing `]` bracket following the expresion",
                        "expected ']' found {}", token.lexeme
                    )
                })?.clone();

                expr = Expr::Access {
                    list: Box::new(expr),
                    key: Box::new(index),
                    brackets: (lb_token, rb_token),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    // todo: add access "[" expr "]"
    fn primary(&mut self) -> miette::Result<Expr> {
        // TRUE
        if self.match_token(&True) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            });
        }
        // FALSE
        if self.match_token(&False) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            });
        }
        // NULL
        if self.match_token(&Null) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            });
        }
        // string
        if self.match_token(&StringLiteral) {
            let token = self.previous().clone();

            // todo improve this message
            let literal = token
                .literal
                .clone()
                .miette_expect(|| miette!("internal parser error. could not find literal"));

            // if it is not string
            let LiteralValue::String(literal) = literal else {
                let report = miette!("internal parser error literal is not a string");
                panic!("{:?}", report)
            };

            return Ok(Expr::Literal {
                value: Literal::String(literal),
                token,
            });
        }

        // number
        if self.match_token(&Number) {
            let token = self.previous().clone();

            // todo improve this message
            let literal = token
                .literal
                .clone()
                .miette_expect(|| miette!("internal parser error. could not find literal"));

            // if it is not number
            let LiteralValue::Number(literal) = literal else {
                let report = miette!("internal parser error literal is not a number");
                panic!("{:?}", report)
            };

            return Ok(Expr::Literal {
                value: Literal::Number(literal),
                token,
            });
        }
        // done parsing literals

        // IDENT
        if self.match_token(&Identifier) {
            let token = self.previous().clone();
            // add possible ident warnings
            // self.ident_warning(&token);

            let ident = token.lexeme.clone();

            // function call
            // IDENT "(" ( expr ),* ")"
            if self.match_token(&LeftParen) {
                let lp_token = self.previous().clone();

                let mut arguments = vec![];
                if !self.check(&RightParen) {
                    loop {
                        if arguments.len() >= 255 {
                            let next_token = self.peek();
                            // todo: improve this message
                            let report = miette!(
                                // todo: finish this
                                "todo: max args for function call exceeded"
                            );
                            return Err(report);
                        }

                        let expr = self.expression()?;
                        arguments.push(expr);

                        // we have reached the end of arguments
                        if !self.match_token(&Comma) {
                            break;
                        }
                    }
                }

                let rp_token = self
                    .consume(&RightParen, |token| {
                        // todo
                        // miette!("expected ) after argument list, found {token}")
                        let labels = vec![LabeledSpan::at(token.span(), "expected a `)`")];

                        miette!(
                            labels = labels,
                            code = "missing_rp",
                            help = "mismatched `(`, it seems you missed a `)`.",
                            "expected `)`, found `{}`",
                            token.lexeme
                        )
                    })?
                    .clone();

                return Ok(Expr::ProcCall {
                    ident,
                    arguments,
                    token,
                    parens: (lp_token, rp_token),
                });
            }

            // ident token
            return Ok(Expr::Variable { ident, token });
        }

        // "(" expr ")"
        if self.match_token(&LeftParen) {
            let lp_token = self.previous().clone();
            let expr = self.expression()?.into();

            let rp_token = self.consume(&RightParen, |token| {
                // todo: improve this message
                // miette!("expected `(` found {}", token)
                let labels = vec![LabeledSpan::at(token.span(), "expected a `(`")];

                miette!(
                    labels = labels,
                    code = "missing_lp",
                    help = "mismatched `)`, it seems you missed a `(`.",
                    "expected `(` found `{}`",
                    token.lexeme
                )
            })?;

            return Ok(Expr::Grouping {
                expr,
                parens: (lp_token.clone(), rp_token.clone()),
            });
        }

        // "[" ( expr ),* "]"
        if self.match_token(&LeftBracket) {
            let lb_token = self.previous().clone();

            let mut items = vec![];
            if !self.check(&RightBracket) {
                loop {
                    let expr = self.expression()?;
                    items.push(expr);

                    // we have reached the end of arguments
                    if !self.match_token(&Comma) {
                        break;
                    }
                }
            }

            let rb_token = self.consume(&RightBracket, |token| {
                // todo
                // miette!("expected ] after item list, found {token}")
                let labels = vec![LabeledSpan::at(token.span(), "expected a `]`")];

                miette!(
                    labels = labels,
                    code = "missing_rb",
                    help = "mismatched `[`, it seems you missed a `]`.",
                    "expected `]`, found `{}`",
                    token.lexeme
                )
            })?;

            return Ok(Expr::List {
                items,
                brackets: (lb_token, rb_token.clone()),
            });
        }

        let cspan = self.previous().span_to(self.peek().span());
        let labels = vec![
            LabeledSpan::at(self.peek().span(), "primary expected here"),
            LabeledSpan::at(cspan, "consider checking your upstream code"),
        ];
        // todo improve this message
        let report = miette!(
            labels = labels,
            help = "a primary is made up of the following set:\n\
            [expression | ident | literal | list]",
            "expected primary, instead found {}\n",
            self.peek()
        )
        .with_source_code(self.named_source.clone());
        // mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm
        Err(report)
    }
}

/// Helper methods
impl Parser2 {
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == SoftSemi {
                return;
            }

            // todo: dont know if this is complete but its "good enough"
            match self.peek().token_type {
                Procedure | Repeat | For | If | Return | Continue | Break | Print => return,
                _ => (),
            }

            self.advance();
        }
    }

    fn consume(
        &mut self,
        token_type: &TokenType,
        report: impl FnOnce(&Token) -> Report,
    ) -> miette::Result<&Token> {
        let next_token = self.peek().clone();
        if next_token.token_type() == token_type {
            self.advance();
            let token = self.previous();
            Ok(token)
        } else {
            // self.synchronize();
            Err(report(&next_token).with_source_code(self.named_source.clone()))
        }
    }

    fn take_semis(&mut self) {
        // todo: consider differentiating between if token was added by user or by lexer
        while self.check(&SoftSemi) {
            self.advance();
        }
    }

    // fn consume(&mut self, token_type: &TokenType, error_handler: Box<dyn Fn(&Token) -> Report>) -> miette::Result<&Token> {
    //     let token = self.peek();
    //
    //     if token.token_type() == token_type {
    //         self.advance();
    //         let token = self.previous();
    //         Ok(token)
    //     } else {
    //         Err(error_handler(token))
    //     }
    // }

    fn check(&self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type() == typ
    }

    fn confirm(&self, typ: &TokenType) -> miette::Result<()> {
        let previous = self.previous();

        if &previous.token_type != typ {
            // todo: improve this msg
            return Err(miette!(
                "attempted to look back and find {:?} buf found {}",
                typ,
                previous
            ));
        }

        Ok(())
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }
        false
    }

    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for ty in types {
            if self.match_token(ty) {
                return true;
            }
        }
        false
    }
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            // todo: switch to miette_expect
            .expect("internal error: attempted to peek token when there is no token to peek")
    }

    fn previous(&self) -> &Token {
        if self.current == 0 {
            // todo switch to miette! panic
            panic!("internal error: there is no previous token");
        }

        // todo: idea bug severity
        // Severity::Bug

        self.tokens
            .get(self.current - 1)
            // todo: improve this message include link to github issues (miette_expect)
            .expect(
                "internal error: this should never happen. \
            if it does there is a bug in previous method",
            )
        // .expect_miette(false, || {
        //     todo
        // });
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }
}

pub(super) mod warning {
    use crate::parser2::Parser2;
    use crate::token::TokenType::Identifier;
    use crate::token::{get_keywords_hashmap, Token};
    use miette::{miette, Report, Severity};

    impl Parser2 {
        pub(super) fn warning(&mut self, report: Report) {
            self.warnings
                .push(report.with_source_code(self.named_source.clone()))
        }

        // todo: add warnings to parameters
        // pub(super) fn ident_warning(&mut self, ident: &Token) {
        //     if ident.token_type == Identifier {
        //         panic!("Internal error trying to warn about ident but input is not ident")
        //     }
        //
        //     if get_keywords_hashmap().contains_key(ident.lexeme.to_lowercase().as_str()) {
        //         let lexeme = &ident.lexeme;
        //         let report = miette!(
        //             severity = Severity::Warning,
        //             "it is not recommended that your identifier echos {}", lexeme
        //         );
        //         self.warning(report);
        //     }
        // }
    }
}

trait ExpectMiette<T> {
    fn miette_expect(self, report_handler: fn() -> Report) -> T;
}

impl<T, E> ExpectMiette<T> for Result<T, E> {
    fn miette_expect(self, report_handler: fn() -> Report) -> T {
        match self {
            Ok(t) => t,
            Err(_) => {
                let report = report_handler();
                panic!()
            }
        }
    }
}

impl<T> ExpectMiette<T> for Option<T> {
    fn miette_expect(self, report_handler: fn() -> Report) -> T {
        match self {
            Some(t) => t,
            None => {
                let report = report_handler();
                panic!()
            }
        }
    }
}
