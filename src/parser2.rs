

use std::fmt::Display;
use std::ops::Range;
use std::sync::Arc;
use std::thread::current;
use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportBuilder, ReportKind, Span};
use crate::ast::{Ast, Expr, Literal, LogicalOp, Stmt};
use crate::lexer::LiteralValue;
use crate::{LReport, LResult, LResults};
use crate::token::{Token, TokenType};
use crate::token::TokenType::{Eof, LeftParen, RightParen};

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

pub struct Parser2<'p> {
    tokens: Vec<Token>,
    source: Arc<str>,
    current: usize,
    warnings: Vec<LReport<'p>>,
    file_name: &'p str,
}


impl<'p> Parser2<'p> {
    pub(crate) fn new(tokens: Vec<Token>, source: Arc<str>, file_name: &'p str) -> Self {
        Self {
            file_name,
            tokens,
            source,
            current: 0,
            warnings: vec![],
        }
    }

    pub(crate) fn parse(&mut self) -> LResults<'p, Ast> {
        let mut statements= vec![];
        let mut reports = vec![];

        while !self.is_at_end() {
            if self.match_token(&SoftSemi) {
                continue;
            }
            
            match self.declaration() {
                Ok(stmt) => statements.push(stmt),
                Err(report) => {
                    self.synchronize();
                    reports.push(report)
                },
            }
        }

        if !reports.is_empty() {
            return Err(reports)
        }

        Ok(Ast {
            source: self.source.clone(),
            program: statements
        })
    }
}

/// parse expression
impl<'p> Parser2<'p> {

    fn declaration(&mut self) -> LResult<'p, Stmt> {
        if self.match_token(&Procedure) {
            let proc_token = self.previous().clone();

            return self.procedure(proc_token);
        }
        self.statement()
    }



    fn procedure(&mut self, proc_token: Token) -> LResult<'p, Stmt> {
        let offset = self.offset();
        let file_name = self.file_name;




        let name_token = self.consume(&Identifier, |token, builder| {
            builder
                .with_message("expected a name for procedure")
                .finish()
        })?;
        
        // self.ident_warning(&name_token);
        
        let name = name_token.lexeme.clone();

        let offset = self.offset();
        
        let lp_token = self.consume(&LeftParen, |token, builder |{
            // miette!(
            //     labels = vec![LabeledSpan::at(token.span, "kill yourself2")],
            //     "expected lp token, found {token}"
            // )
            builder
                .with_message(format!("expected lp token, found {token}"))
                .finish()
        })?.clone();
        
        let (params, params_tokens) = if !self.check(&RightParen) {
            // parse shit
            // todo

            (vec![], vec![])
        } else {
            (vec![], vec![])
        };
        
        let rp_token = self.consume(&RightParen, |token, builder| {
            // miette!("expected a rparen, found {token}")
            builder
                .with_message(format!("expected a rparen, found {token}")).finish()
        })?.clone();
        
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


    fn statement(&mut self) -> LResult<'p, Stmt> {
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
    
    
    fn block(&mut self, lb_token: Token) -> LResult<'p, Stmt> {
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

        let rb_token = self.consume(&RightBrace, |token, builder| {
            // miette!("expected right brace")
            builder
                .with_message("expected right brace").finish()
        })?.clone();


        Ok(Stmt::Block {
            lb_token,
            statements,
            rb_token
        })
    }

    fn if_statement(&mut self, if_token: Token) -> LResult<'p, Stmt> {
        // todo: improve this report
        let lp_token = self.consume(&LeftParen, |token, builder|
            // miette!("expected lp_token")
            builder
                .with_message("expected lp_token".to_string())
                .finish()
        )?.clone();

        let condition = self.expression()?;

        let rp_token = self.consume(&RightParen, |token, builder| {
            // miette!("Expected `)` found {}", token)
            builder
                .with_message(format!("Expected `)` found {}", token))
                .finish()
        })?.clone();

        println!("\nbegin");
        let then_branch = self.statement()?.into();
        println!("end");
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
    
    fn repeat_times(&mut self, repeat_token: Token) -> LResult<'p, Stmt> {
        // confirm that the repeat token was consumed
        self.confirm(&Repeat)?;
        
        // expected expression 
        let count = self.expression()?;
        
        let times_token = self.consume(&Times, |token, report| {
            // todo improve this message
            // miette!("expected times token")
            report
                .with_message("expected `TIMES`, found {token}")
                .finish()
        })?.clone();
        
        let body = self.statement()?.into();
        
        Ok(Stmt::RepeatTimes {
            count,
            body,
            repeat_token,
            times_token,
        })
    }
    
    fn repeat_until(&mut self, repeat_token: Token) -> LResult<'p, Stmt> {
        // confirm that the repeat token has been consumed
        self.confirm(&Repeat)?;
        
        let until_token= self.consume(&Until, |token,report| {
            // todo: improve this error
            // miette!(
            //     "expected until token after repeat token"
            // )
            report
                .with_message("expected until token after repeat token")
                .finish()
        })?.clone();
        
        let lp_token = self.consume(&LeftParen, |token, report| {
            // todo: improve this error
            // miette!(
            //     "expected lp token"
            // )
            report
                .with_message("expected lp token")
                .finish()
        })?.clone();
        
        let condition = self.expression()?;
        
        let rp_token = self.consume(&RightParen, |token, report| {
            // todo: improve this error
            // miette!(
            //     "expected rp token"
            // )
            report
                .with_message("expected rp token")
                .finish()
        })?.clone();
        
        let body = self.statement()?.into();
        
        Ok(Stmt::RepeatUntil {
            condition, 
            body,
            repeat_token,
            until_token,
        })
    }
    
    fn for_each(&mut self, for_token: Token) -> LResult<'p, Stmt> {
        {
            self.confirm(&For)?;
        }
        
        let each_token = self.consume(&Each, |token, report| {
            // todo improve this message
            // miette!("expected each token")
            report
                .with_message("expected each token")
                .finish()
        })?;
        
        let item_token = self.consume(&Identifier, |token, report| {
            // todo improve this message
            // miette!("expected an ident")
            report
                .with_message("expected an ident")
                .finish()
        })?;

        let item = item_token.lexeme.clone().clone();
        let in_token= self.consume(&In, |token, report| {
            // miette!("expected in token")
            report
                .with_message("expected `IN` token")
                .finish()
        })?;
        
        let list = self.expression()?;
        
        let body = self.statement()?.into();
        
        Ok(Stmt::ForEach {
            item,
            list,
            body,
            item_token,
            for_token,
            in_token
        })
    }
    
    fn expression_statement(&mut self) -> LResult<'p, Stmt> {
        let expr = self.expression()?;
        if self.is_at_end() {
            return Ok(Stmt::Expr {expr})
        }
        
        if self.check(&RightBrace) {
            return Ok(Stmt::Expr {expr})
        }
        
        self.consume(&SoftSemi, |token, report| {
            report.with_message(format!("expected eol or semi found {}", token)).finish()
        })?;
        Ok(Stmt::Expr {expr})
    }

    pub(crate) fn expression(&mut self) -> LResult<'p, Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> LResult<'p, Expr> {
        let expr = self.or()?;

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
                Expr::Access { list, key, brackets } => Ok(Expr::Set {
                    target: Box::new(Expr::Access { list, key, brackets }),
                    value: Box::new(value),
                    arrow_token,
                }),

                // Error for invalid assignment target
                // todo: add better error here
                // _ => Err(miette!("Invalid assignment target.")),
                _ => {
                    let report = Report::build(ReportKind::Error, self.file_name, self.offset())
                        .with_message("Invalid assignment target.")
                        .finish();
                    Err(report)
                }
            }
        } else {
            Ok(expr)
        }
    }


    // and ( "OR" and )*
    fn or(&mut self) -> LResult<'p, Expr> {
        let mut expr = self.and()?;

        while self.match_token(&Or) {
            // get or token for spanning
            let token = self.previous().clone();

            let right = self.and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator: LogicalOp::Or,
                right: Box::new(right),

                token
            }
        }

        Ok(expr)
    }

    // logical_and -> equality ( "AND" equality )*
    fn and(&mut self) -> LResult<'p, Expr> {
        let mut expr = self.equalitu()?;

        while self.match_token(&And) {
            // get the token for spanning
            let token = self.previous().clone();

            let right = self.and()?;
            expr = Expr::Logical {
                left: expr.into(),
                operator: LogicalOp::Or,
                right: right.into(),

                token
            }
        }

        Ok(expr)
    }

    fn equalitu(&mut self) -> LResult<'p, Expr> {
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
                token
            }
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> LResult<'p, Expr> {
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
                token
            }
        }

        Ok(expr)
    }

    fn addition(&mut self) -> LResult<'p, Expr> {
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
                token
            }
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> LResult<'p, Expr> {
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
                token
            }
        }

        Ok(expr)
    }

    fn unary(&mut self) -> LResult<'p, Expr> {
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

    fn access(&mut self) -> LResult<'p, Expr> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&LeftBracket) {
                let lb_token = self.previous().clone();

                let index = self.expression()?;
                let rb_token = self.consume(&RightBracket, |token, builder| {
                    builder
                        .with_message("Expected ']' after index")
                        .finish()
                })?.clone();

                expr = Expr::Access {
                    list: Box::new(expr),
                    key: Box::new(index),
                    brackets: (lb_token, rb_token)
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }
    
    // todo: add access "[" expr "]"
    fn primary(&mut self) -> LResult<'p, Expr> {
        // TRUE
        if self.match_token(&True) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            })
        }
        // FALSE
        if self.match_token(&False) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            })
        }
        // NULL
        if self.match_token(&Null) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            })
        }
        // string
        if self.match_token(&StringLiteral) {
            let token = self.previous().clone();

            // todo improve this message
            let literal = token.literal.clone().expect(
                "internal parser error. could not find literal"
            );

            // if it is not string
            let LiteralValue::String(literal) = literal else {
                panic!("internal parser parser error. literal is not a string")
            };

            return Ok(Expr::Literal {
                value: Literal::String(literal),
                token
            })
        }

        // number
        if self.match_token(&Number) {
            let token = self.previous().clone();


            // todo improve this message
            let literal = token.literal.clone().expect(
                "interal parser error. could not find literal"
            );

            // if it is not number
            let LiteralValue::Number(literal) = literal else {
                // let report = miette!(
                //     "internal parser error literal is not a number"
                // );
                panic!("internal parser error literal is not a number")
            };

            return Ok(Expr::Literal {
                value: Literal::Number(literal),
                token
            })
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
                            // let report = miette!(
                            //     "todo: max args for function call exceeded"
                            // );
                            let report = Report::build(ReportKind::Error, self.file_name, self.offset())
                                .with_message("todo: max args for function call exceeded")
                                .finish();
                            return Err(report)
                        }

                        let expr = self.expression()?;
                        arguments.push(expr);

                        // we have reached the end of arguments
                        if !self.match_token(&Comma) {
                            break;
                        }
                    }
                }

                let rp_token = self.consume(&RightParen,  |token, report| {
                    // todo
                    // miette!("expected ) after argument list, found {token}")
                    report
                        .with_message(format!("expected ) after argument list, found {token}"))
                        .finish()
                })?.clone();

                return Ok(Expr::ProcCall {
                    ident,
                    arguments,
                    token,
                    parens: (lp_token, rp_token),
                })
            }
            
            // ident token
            return Ok(Expr::Variable {
                ident,
                token,
            })
        }

        // "(" expr ")"
        if self.match_token(&LeftParen) {
            let lp_token = self.previous().clone();
            let expr = self.expression()?.into();

            let rp_token = self.consume(&RightParen, |token, report| {
                // todo: improve this message
                // miette!("expected `(` found {}", token)
                report
                    .with_message(format!("expected `(` found {}", token))
                    .finish()
            })?;

            return Ok(Expr::Grouping {
                expr,
                parens: (lp_token.clone(), rp_token.clone())
            })
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

            let rb_token = self.consume(&RightBracket,  |token, report| {
                // todo
                // miette!("expected ] after item list, found {token}")
                report
                    .with_message(format!("expected ] after item list, found {token}"))
                    .finish()
            })?;

            return Ok(Expr::List {
                items,
                brackets: (lb_token, rb_token.clone()),
            });
        }

        // todo improve this message
        // let report = miette!(
        //     labels = vec![LabeledSpan::at(self.peek().span, "kill yourself")],
        //     "expected primary found1 {}", self.peek()
        // ).with_source_code(self.source.clone());
        let span = self.peek().span.clone();
        let report = Report::build(ReportKind::Error, self.file_name, self.offset())
            .with_message(format!("expected primary found1 {}", self.peek()))
            .with_label(
                Label::new((self.file_name, span))
                    .with_message("hello")
            )
            .finish();
        // mmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmmm
        Err(report)
    }
}


/// Helper methods
impl<'p> Parser2<'p> {
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

    fn consume(&mut self, token_type: &TokenType, error_fn: impl FnOnce(Token, ReportBuilder<'p, (&'p str, Range<usize>)>) -> LReport<'p>) -> LResult<'p, Token> {
        let token_type_matches = {
            let token = self.peek(); // Immutable borrow is limited to this block
            token.token_type() == token_type
        };
        self.advance();
        if token_type_matches {
            let token = self.previous();
            Ok(token.clone())
        } else {
            let token = self.previous().clone();
            let builder = Report::build(ReportKind::Error, self.file_name.clone(), self.offset());
            let report: LReport<'p> = error_fn(token, builder);
            Err(report)
        }
    }


    // fn consume33(&mut self, token_type: &TokenType, error_fn: impl FnOnce(Token, ReportBuilder<'p, (&str, Range<usize>)>) -> ReportBuilder<'p, (&'p str, Range<usize>)>) -> LResult<'p, & Token> {
    //     let token_type_matches = {
    //         let token = self.peek(); // Immutable borrow is limited to this block
    //         token.token_type() == token_type
    //     };
    // 
    //     if token_type_matches {
    //         let token = self.previous();
    //         Ok(token)
    //     } else {
    //         let token = self.previous().clone();
    //         let builder = Report::build(ReportKind::Error, self.file_name.clone(), self.offset());
    //         let report: LReport<'p> = error_fn(token, builder).finish();
    //         Err(report)
    //     }
    // }
    // 



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
            return false
        }

        self.peek().token_type() == typ
    }
    
    fn confirm(&self, typ: &TokenType) -> LResult<'p, ()> {
        let previous = self.previous();
        
        if &previous.token_type != typ {
            // todo: improve this msg
            // return Err(miette!(
            //     "attempted to look back and find {:?} buf found {}", typ, previous
            // ));
            let report = Report::build(ReportKind::Error, self.file_name, self.offset())
                .with_message(format!("attempted to look back and find {:?} buf found {}", typ, previous))
                .finish();
            return Err(report)
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
    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    fn peek(&self) -> Token {
        self.tokens
            .get(self.current)
            // todo: switch to miette_expect
            .expect("internal error: attempted to peek token when there is no token to peek").clone()
    }

    fn previous(&self) -> Token {
        if self.current == 0 {
            // todo switch to miette! panic
            panic!("internal error: there is no previous token");
        }

        // todo: idea bug severity
        // Severity::Bug

        self.tokens
            .get(self.current - 1)
            // todo: improve this message include link to github issues (miette_expect)
            .expect("internal error: this should never happen. \
            if it does there is a bug in previous method").clone()
            // .expect_miette(false, || {
            //     todo
            // });
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }

    fn offset(&self) -> usize {
        self.tokens[self.current].span.start
    }
}

// pub(super) mod warning {
//     use crate::parser2::Parser2;
//     use crate::token::{get_keywords_hashmap, Token};
//     use crate::token::TokenType::Identifier;
//     use crate::{LReport, LResult, LResults};
// 
//     impl Parser2<'p> {
//         pub(super) fn warning(&mut self, report: LReport) {
//             self.warnings.push(report.with_source_code(self.source.clone()))
//         }
// 
//         // todo: add warnings to parameters
//         // pub(super) fn ident_warning(&mut self, ident: &Token) {
//         //     if ident.token_type == Identifier {
//         //         panic!("Internal error trying to warn about ident but input is not ident")
//         //     }
//         //     
//         //     if get_keywords_hashmap().contains_key(ident.lexeme.to_lowercase().as_str()) {
//         //         let lexeme = &ident.lexeme;
//         //         let report = miette!(
//         //             severity = Severity::Warning,
//         //             "it is not recommended that your identifier echos {}", lexeme
//         //         );
//         //         self.warning(report);
//         //     }
//         // }
//     }
// }

// trait ExpectMiette<T> {
// 
//     fn miette_expect(self,  report_handler: fn() -> LReport) -> T;
// }
// 
// impl<T, E> ExpectMiette<T> for Result<T, E> {
//     fn miette_expect(self, report_handler: fn() -> LReport) -> T {
//         match self {
//             Ok(t) => t,
//             Err(_) => {
//                 let report = report_handler();
//                 panic!()
//             }
//         }
//     }
// }
// 
// impl<T> ExpectMiette<T> for Option<T> {
//     fn miette_expect(self, report_handler: fn() -> LReport) -> T {
//         match self {
//             Some(t) => t,
//             None => {
//                 let report = report_handler();
//                 panic!()
//             }
//         }
//     }
// }
