use crate::expr::{Expr, Literal};
use crate::token::TokenType::*;
use std::fmt::Display;
use crate::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn expression(&mut self) -> Expr {
        self.equality()
    }

    fn equality(&mut self) -> Expr {
        let mut expr = self.comparison();

        while self.match_tokens(&[BangEqual, EqualEqual]) {
            let operator = self.previous();
            let rhs = self.comparison();
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: rhs.into(),
            };
        }

        expr
    }

    fn comparison(&mut self) -> Expr {
        let mut expr = self.term();

        while self.match_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            let operator = self.previous();
            let rhs = self.term();
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: rhs.into(),
            }
        }

        expr
    }

    fn term(&mut self) -> Expr {
        let mut expr = self.factor();

        while self.match_tokens(&[Minus, Plus]) {
            let operator = self.previous();
            let rhs = self.factor();
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: rhs.into(),
            }
        }

        expr
    }

    fn factor(&mut self) -> Expr {
        let mut expr = self.unary();

        while self.match_tokens(&[Slash, Star]) {
            let operator = self.previous();
            let rhs = self.unary();
            expr = Expr::Binary {
                left: expr.into(),
                operator,
                right: rhs.into(),
            }
        }

        expr
    }

    fn unary(&mut self) -> Expr {
        if self.match_tokens(&[Not, Minus]) {
            let operator = self.previous();
            let rhs = self.unary();
            Expr::Unary {
                operator,
                right: rhs.into(),
            }
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Expr {
        if self.match_token(&LeftParen) {
            let expr = self.expression();
            self.consume(&RightParen, "Expected ')'");
            Expr::Parens { expr: expr.into() }
        } else {
            let token = self.peek();
            self.advance();
            Expr::Literal {
                value: Literal::from(token),
            }
        }
    }

    fn consume(&mut self, token_type: &TokenType, msg: impl Display) {
        let token = self.peek();
        if token.token_type == *token_type {
            self.advance();
        } else {
            panic!("{}", msg)
        }
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else if self.peek().token_type == *token_type {
            self.advance();
            true
        } else {
            false
        }
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
            .expect("Internal Parser Error! Parser off track. Bounds are fucked")
            .clone()
    }

    fn previous(&self) -> Token {
        if self.current == 0 {
            panic!("there is not previous token");
        }
        self.tokens
            .get(self.current - 1)
            .expect("Internal Parser Error! Bounds are fucked. *Prev")
            .clone()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_add() {
        let input = "NOT false + \nnot true";
        let mut scanner = Lexer::new(input, String::default());
        let tokens = scanner.scan_tokens().unwrap();

        let mut parser = Parser::new(tokens);
        let expr = parser.expression();

        println!("{:#?}", expr);
    }
}
