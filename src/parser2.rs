// use std::fmt::Display;
// use crate::expr::{Expr, Literal};
// use crate::scanner::{LiteralValue, Token, TokenType};
// use crate::scanner::TokenType::*;
// use crate::stmt::Stmt;

// type Result<T> = core::result::Result<T, String>;

// pub struct Parser2 {
//     tokens: Vec<Token>,
//     current: usize,
//     next_id: usize,
// }

// impl Parser2 {
//     pub fn new(tokens: Vec<Token>) -> Self {
//         Self {
//             tokens,
//             current: 0,
//             next_id: 0,
//         }
//     }

//     fn get_id(&mut self) -> usize {
//         let id = self.next_id;
//         self.next_id += 1;

//         id
//     }

//     pub fn parse(&mut self) -> Result<Vec<Stmt>> {
//         let mut stmts = vec![];
//         let mut errors = vec![];

//         while !self.is_at_end() {
//             let stmt = self.declaration();

//             match stmt {
//                 Ok(s) => stmts.push(s),
//                 Err(msg) => {
//                     errors.push(msg);

//                     // if we find an error then we need to sync to the next stmt
//                     self.synchronize();
//                 }
//             }
//         }

//         if errors.len() == 0 {
//             Ok(stmts)
//         } else {
//             Err(errors.join("\n"))
//         }
//     }

//     // START PEG

//     fn declaration(&mut self) -> Result<Stmt> {
//         if self.match_token(&Procedure) {
//             self.procedure()
//         } else {
//             self.statement()
//         }
//     }

//     fn procedure(&mut self) -> Result<Stmt> {
//         todo!()
//     }

//     fn statement(&mut self) -> Result<Stmt> {
//         // { .. }
//         if self.match_token(&LeftBrace) {
//             self.block_statement()
//         }
//         // IF condition | IF condition .. ELSE
//         else if self.match_token(&If) {
//             self.if_statement()
//         }
//         // REPEAT UNTIL | REPEAT ident TIMES
//         else if self.match_token(&Repeat) {
//             self.loop_statement()
//         }
//         // FOR x IN array
//         else if self.match_token(&For) {
//             self.for_statement()
//         }
//         // RETURN x
//         else if self.match_token(&Return) {
//             self.return_statement()
//         }
//         // CONTINUE
//         else if self.match_token(&Continue) {
//             self.continue_statement()
//         }
//         // BREAK
//         else if self.match_token(&Break) {
//             self.break_statement()
//         }
//         // PRINT x
//         if self.match_token(&Print) {
//             self.print_statement()
//         }
//         // ..
//         else {
//             self.expression_statement()
//         }
//     }

//     fn primary(&mut self) -> Result<Expr> {
//         let token = self.peek();

//         Ok(match token.token_type {
//             LeftParen => {
//                 self.advance();
//                 let expr = self.expression()?;
//                 self.consume(RightParen, "Expected ')'")?;
//                 Expr::Parens {
//                     expr: expr.into()
//                 }
//             },
//             _ => Err("Expected Expression".to_string()),
//         })
//     }

//     // UTIlS BELOW
//     fn consume(&mut self, token_type: TokenType, msg: &str) -> Result<Token> {
//         let token = self.peek();
//         if token.token_type == token_type {
//             self.advance();
//             let token = self.previous();
//             Ok(token)
//         } else {
//             Err(format!("Line {}: {}", token.line_number, msg))
//         }
//     }

//     fn check(&mut self, typ: TokenType) -> bool {
//         self.peek().token_type == typ
//     }

//     fn match_token(&mut self, typ: &TokenType) -> bool {
//         if self.is_at_end() {
//             false
//         } else if self.peek().token_type == *typ {
//             self.advance();
//             true
//         } else {
//             false
//         }
//     }

//     fn match_tokens(&mut self, typs: &[TokenType]) -> bool {
//         for typ in typs {
//             if self.match_token(typ) {
//                 return true;
//             }
//         }

//         false
//     }

//     fn advance(&mut self) -> Token {
//         if !self.is_at_end() {
//             self.current += 1;
//         }

//         self.previous()
//     }

//     fn peek(&mut self) -> Token {
//         self.tokens[self.current].clone()
//     }

//     fn previous(&mut self) -> Token {
//         self.tokens[self.current - 1].clone()
//     }



//     fn is_at_end(&mut self) -> bool {
//         self.peek().token_type == Eof
//     }
// }



use std::fmt::Display;
use std::sync::Arc;
use miette::{Diagnostic, miette, NamedSource, Report, Severity, SourceSpan};
use thiserror::Error;
use crate::ast::{Expr, Literal, LogicalOp};
use crate::lexer::LiteralValue;
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

pub struct Parser2 {
    tokens: Vec<Token>,
    source: Arc<str>,
    current: usize,
}


impl Parser2 {
    pub(crate) fn new(tokens: Vec<Token>, source: Arc<str>) -> Self {
        Self {
            tokens,
            source,
            current: 0
        }
    }
}

/// parse expression
impl Parser2 {
    
    pub(crate) fn expression(&mut self) -> miette::Result<Expr> {
        self.or()
        // todo finish assignment
        // self.assignment()
    }
    
    fn assignment(&mut self) -> miette::Result<Expr> {
        todo!()
    }

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

                token
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

                token
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
                token
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
                token
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
                token
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
                token
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
            self.primary()
        }
    }

    
    // todo: add access "[" expr "]"
    fn primary(&mut self) -> miette::Result<Expr> {
        if self.match_token(&True) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            })
        }

        if self.match_token(&False) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            })
        }

        if self.match_token(&Null) {
            let token = self.previous().clone();
            return Ok(Expr::Literal {
                value: Literal::True,
                token,
            })
        }

        if self.match_token(&StringLiteral) {
            let token = self.previous().clone();


            // todo improve this message
            let literal = token.literal.clone().miette_expect(|| {
                miette!("internal parser error. could not find literal")
            });

            // if it is not string
            let LiteralValue::String(literal) = literal else {
                let report = miette!(
                    "internal parser error literal is not a string"
                );
                panic!("{:?}", report)
            };

            return Ok(Expr::Literal {
                value: Literal::String(literal),
                token
            })
        }

        if self.match_token(&Number) {
            let token = self.previous().clone();


            // todo improve this message
            let literal = token.literal.clone().miette_expect(|| {
                miette!("internal parser error. could not find literal")
            });

            // if it is not string
            let LiteralValue::Number(literal) = literal else {
                let report = miette!(
                    "internal parser error literal is not a number"
                );
                panic!("{:?}", report)
            };

            return Ok(Expr::Literal {
                value: Literal::Number(literal),
                token
            })
        }

        // done parsing literals

        if self.match_token(&Identifier) {
            let token = self.previous().clone();
            let ident = token.lexeme.clone();
            
            // function call
            if self.match_token(&LeftParen) {
                todo!()
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

            let lp_temp = lp_token.clone();
            let rp_token = self.consume(&RightParen, Box::new(move |token| {
                let lp = &lp_temp;
                // todo: improve this message
                miette!("could not find this error")
            }))?;

            return Ok(Expr::Grouping {
                expr,
                parens: (lp_token.clone(), rp_token.clone())
            })
        }
        
        // lists here
        if self.match_token(&LeftBracket) {
            // todo list code
            todo!()
        }

        // todo improve this message
        let report = miette!("expected found {}", self.peek());
        Err(report)
    }

    fn proc_call(&mut self) -> miette::Result<Expr> {
        todo!()
    }

    fn access(&mut self) -> miette::Result<Expr> {
        todo!()
    }

    fn list(&mut self) -> miette::Result<Expr> {
        todo!()
    }

    fn arguments(&mut self) -> miette::Result<Expr> {
        todo!()
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
                Procedure | Repeat | For | If | Return | Continue | Break | Print=> return,
                _ => (),
            }

            self.advance();
        }
    }

    fn consume(&mut self, token_type: &TokenType, error_handler: Box<dyn Fn(&Token) -> Report>) -> miette::Result<&Token> {
        let token = self.peek();

        if token.token_type() == token_type {
            self.advance();
            let token = self.previous();
            Ok(token)
        } else {
            Err(error_handler(token))
        }
    }

    fn check(&self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false
        }

        self.peek().token_type() == typ
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
            .expect("internal error: this should never happen. \
            if it does there is a bug in previous method")
            // .expect_miette(false, || {
            //     todo
            // });
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }
}

trait ExpectMiette<T> {

    fn miette_expect(self,  report_handler: fn() -> Report) -> T;
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
