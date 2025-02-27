use crate::lexer::token::LiteralValue;
use crate::lexer::token::TokenType::{Eof, LeftParen, RightParen};
use crate::lexer::token::{Token, TokenType};
use crate::parser::ast::Break as BreakStatement;
use crate::parser::ast::Continue as ContinueStatement;
use crate::parser::ast::Import as ImportStatement;
use crate::parser::ast::Return as ReturnValue;
use crate::parser::ast::*;
use miette::{miette, LabeledSpan, NamedSource, Report, SourceSpan};
use std::sync::Arc;

use crate::lexer::token::TokenType::*;
use crate::parser::ast::If as IfStmt;

pub struct Parser {
    tokens: Vec<Token>,
    source: Arc<str>,
    named_source: NamedSource<Arc<str>>,
    current: usize,
    in_function_scope: bool,
    _warnings: Vec<Report>,
    in_loop_scope: bool,
}

impl Parser {
    pub(crate) fn new(tokens: Vec<Token>, source: Arc<str>, file_name: &str) -> Self {
        Self {
            tokens,
            source: source.clone(),
            in_function_scope: false,
            in_loop_scope: false,
            named_source: NamedSource::new(file_name, source),
            current: 0,
            _warnings: vec![],
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
                    reports.push(report);
                    self.synchronize();
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
impl Parser {
    fn declaration(&mut self) -> miette::Result<Stmt> {
        // Procedure might start with export.
        // If it needs special treatment
        if self.match_tokens(&[Export, Procedure]) {
            return self.procedure();
        }
        self.statement()
    }

    fn procedure(&mut self) -> miette::Result<Stmt> {
        let export_or_procedure = self.previous().clone();

        let (proc_token, exported) = if export_or_procedure.token_type == Export {
            let proc_token = self
                .consume(&Procedure, |token| {
                    let labels = vec![
                        LabeledSpan::at(token.span(), "expected keyword 'PROCEDURE' here"),
                        LabeledSpan::at(token.span(), "'EXPORT' cannot exist alone"),
                    ];

                    miette!(
                        labels = labels,
                        code = "standalone_export",
                        help = "you can only export a procedure from a module",
                        "expected 'PROCEDURE' following 'EXPORT' found {}",
                        token.lexeme,
                    )
                })?
                .clone();

            (proc_token, true)
        } else {
            (export_or_procedure, false)
        };

        let name_token = self
            .consume(&Identifier, |token| {
                let labels = vec![
                    LabeledSpan::at(proc_token.span(), "this procedure requires a name"),
                    LabeledSpan::at(token.span(), "name goes here"),
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

        let _lp_token = self
            .consume(&LeftParen, |token| {
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

        let mut params = vec![];
        if !self.check(&RightParen) {
            loop {
                if params.len() > 255 {
                    let _peeked = self.peek();
                    return Err(miette! {
                        "todo: params cannot exceed 255, why the f**k do you need so many?"
                    });
                }

                // we expect there to be parameters
                let token = self
                    .consume(&Identifier, |_token| miette!("hello"))?
                    .clone();

                params.push(Variable {
                    ident: token.lexeme.clone(),
                    token,
                });

                if !self.match_token(&Comma) {
                    break;
                }
            }
        }

        let _rp_token = self
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

        // cache previous function state and set to true temporarily, since we're in a
        let function_scope_state_cache = self.in_function_scope;
        self.in_function_scope = true;

        // parse the body of the function
        let body = self.statement()?;
        // restore the previous function scope state
        self.in_function_scope = function_scope_state_cache;

        Ok(Stmt::ProcDeclaration(Arc::new(ProcDeclaration {
            name,
            params,
            body,
            exported,
            proc_token,
            name_token,
        })))
    }

    fn statement(&mut self) -> miette::Result<Stmt> {
        // import statement
        if self.match_token(&Import) {
            let import_token = self.previous().clone();
            return self.import_statement(import_token);
        }

        // IF (condition)
        if self.match_token(&If) {
            let if_token = self.previous().clone();
            return self.if_statement(if_token);
        }

        // REPEAT UNTIL (condition)
        if self.match_token(&Repeat) {
            let repeat_token = self.previous().clone();

            // we're now in a loop
            let cache_loop_state = self.in_loop_scope;
            self.in_loop_scope = true;

            // this is a repeat until block
            let result = if self.check(&Until) {
                self.repeat_until(repeat_token)
            } else {
                self.repeat_times(repeat_token)
            };

            // we *might* not be in a loop anymore
            self.in_loop_scope = cache_loop_state;

            // finish
            return result;
        }

        if self.match_token(&For) {
            let for_token = self.previous().clone();

            // we're now in a loop
            let cache_loop_state = self.in_loop_scope;
            self.in_loop_scope = true;
            
            // parse FOR EACH loop
            let result = self.for_each(for_token);

            // we *might* not be in a loop anymore
            self.in_loop_scope = cache_loop_state;
            
            return result
        }

        // { expr }
        if self.match_token(&LeftBrace) {
            let lb_token = self.previous().clone();

            return self.block(lb_token);
        }

        if self.match_token(&Continue) {
            let cont_token = self.previous().clone();
            return self.continue_statement(cont_token);
        }

        if self.match_token(&Break) {
            let brk_token = self.previous().clone();
            return self.break_statement(brk_token);
        }

        if self.match_token(&Return) {
            let return_token = self.previous().clone();
            return self.return_statement(return_token);
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
            .consume(&RightBrace, |_token| {
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

        Ok(Stmt::Block(
            Block {
                lb_token,
                statements,
                rb_token,
            }
            .into(),
        ))
    }

    fn break_statement(&mut self, break_token: Token) -> miette::Result<Stmt> {
        if !self.in_loop_scope {
            // todo improve this message
            return Err(miette! {
                "BREAK can only be called in a loop"
            });
        }

        Ok(Stmt::Break(Arc::new(BreakStatement { token: break_token })))
    }

    fn continue_statement(&mut self, continue_token: Token) -> miette::Result<Stmt> {
        if !self.in_loop_scope {
            // todo: make this error bearable
            return Err(miette! {
                "CONTINUE can only be called in a loop",
            });
        }

        Ok(Stmt::Continue(Arc::new(ContinueStatement {
            token: continue_token,
        })))
    }

    fn return_statement(&mut self, return_token: Token) -> miette::Result<Stmt> {
        if !self.in_function_scope {
            // todo make this error better
            return Err(miette! {
                "RETURN can only be called in a PROCEDURE"
            });
        }

        let maybe_value = if !self.match_token(&SoftSemi) {
            Some(self.expression()?)
        } else {
            None
        };

        if maybe_value.is_some() {
            self.consume(&SoftSemi, |_token| {
                miette! {
                    "todo: expected semicolon after return statement"
                }
            })?;
        }

        Ok(Stmt::Return(Arc::new(ReturnValue {
            token: return_token,
            data: maybe_value,
        })))
    }

    fn import_statement(&mut self, import_token: Token) -> miette::Result<Stmt> {
        // get the specific function imports
        let only_functions = if self.match_token(&LeftBracket) {
            // matching import ["f1", "f2", "f3"] from mod
            let lbracket = self.previous().clone();

            let mut specific_functions: Vec<Token> = vec![];
            loop {
                // todo: consider making this an argument because arbitrary
                // todo: like max param limits or something
                // set an arbitrary limit for number of specific functions
                const MAX_SPECIFIC_FUNCTIONS: usize = 63;
                if specific_functions.len() >= MAX_SPECIFIC_FUNCTIONS {
                    let correct_span =
                        lbracket.span_until_token(specific_functions.last().unwrap());
                    let labels = vec![LabeledSpan::at(
                        correct_span,
                        "just import the entire module",
                    )];
                    let _s = 2.0;

                    return Err(miette!(
                        labels = labels,
                        help = "what the freak dude. are you okay?",
                        "cannot have more than {} specific imports",
                        MAX_SPECIFIC_FUNCTIONS
                    ));
                }

                let specific_function = self.consume(&StringLiteral, |found| {
                    miette!("expected a specific function instead found {}", found)
                })?;

                specific_functions.push(specific_function.clone());

                // we've reached the end of the specific functions
                if !self.match_token(&Comma) {
                    break;
                }
            }

            println!("debug: specific functions: {:?}", specific_functions);

            // close off the specific functions
            let _rbracket = self.consume(&RightBracket, |_found| {
                miette! {
                    ""
                }
            })?;

            Some(specific_functions)
        } else if self.match_token(&StringLiteral) {
            // matching single specific function ("string literal")
            let one_specific_function = self.previous().clone();
            Some(vec![one_specific_function])
        } else {
            None
        };

        let maybe_from_token =
            if only_functions.is_some() {
                Some(self.consume(&From, |found| miette! {
                "(todo) Expected from following specific imports, found {}", found.lexeme
            })?.clone())
            } else {
                None
            };

        let mod_token = self
            .consume(&Mod, |_token| {
                miette! {
                    "todo: expected a mod token following import. could also be a specific function" // todo make this better
                }
            })?
            .clone();

        let module_name = self
            .consume(&StringLiteral, |_token| {
                miette! {
                    "todo: expected a string literal specifying the type of import"
                }
            })?
            .clone();

        self.consume(&SoftSemi, |_token| {
            miette! {
                "todo: expected a semicolon following import statement"
            }
        })?;

        Ok(Stmt::Import(Arc::new(ImportStatement {
            import_token,
            mod_token,
            maybe_from_token,

            only_functions,
            module_name,
        })))
    }

    fn if_statement(&mut self, if_token: Token) -> miette::Result<Stmt> {
        // todo: improve this report
        let _lp_token = self
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

        let _rp_token = self
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

        let then_branch = self.statement()?;

        let (else_branch, else_token) = if self.match_token(&Else) {
            // there is an ELSE branch
            let else_token = self.previous().clone();
            let else_branch = self.statement()?;

            (Some(else_branch), Some(else_token))
        } else {
            (None, None)
        };

        Ok(Stmt::If(Arc::new(IfStmt {
            condition,
            then_branch,
            else_branch,
            if_token,
            else_token,
        })))
    }

    fn repeat_times(&mut self, repeat_token: Token) -> miette::Result<Stmt> {
        // confirm that the repeat token was consumed
        self.confirm(&Repeat)?;

        // expected expression
        let count = self.expression()?;

        let count_token = self.previous().clone();

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

        let body = self.statement()?;

        Ok(Stmt::RepeatTimes(
            RepeatTimes {
                count,
                body,
                repeat_token,
                times_token,
                count_token,
            }
            .into(),
        ))
    }

    fn repeat_until(&mut self, repeat_token: Token) -> miette::Result<Stmt> {
        // confirm that the repeat token has been consumed
        self.confirm(&Repeat)?;

        let until_token = self
            .consume(&Until, |_token| {
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
                // todo consider making this advance instead of consume
                // this should never error
                miette!("how tf do i trigger this")
            })?
            .clone();

        let _lp_token = self.consume(&LeftParen, |token| {
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

        let _rp_token = self
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

        let body = self.statement()?;

        Ok(Stmt::RepeatUntil(
            RepeatUntil {
                condition,
                body,
                repeat_token,
                until_token,
            }
            .into(),
        ))
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
        // let item = item_token.lexeme.clone();
        let item = Variable {
            ident: item_token.lexeme.to_string(),
            token: item_token.clone(),
        };

        // this is sus?
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

        let list_token = self.previous().clone();

        let body = self.statement()?;

        Ok(Stmt::ForEach(
            ForEach {
                item,
                list,
                body,
                item_token,
                for_token,
                each_token,
                in_token,
                list_token,
            }
            .into(),
        ))
    }

    fn expression_statement(&mut self) -> miette::Result<Stmt> {
        let expr = self.expression()?;
        if self.is_at_end() {
            return Ok(Stmt::Expr(Arc::new(expr)));
        }

        if self.check(&RightBrace) {
            return Ok(Stmt::Expr(Arc::new(expr)));
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
        Ok(Stmt::Expr(Arc::new(expr)))
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
                Expr::Variable(ref variable) => Ok(Expr::Assign(
                    Assignment {
                        target: variable.clone(),
                        value,
                        ident_token: variable.token.clone(),
                        arrow_token,
                    }
                    .into(),
                )),

                // Handling set assignment for complex expressions like array[index] = value
                Expr::Access(ref access) => Ok(Expr::Set(
                    Set {
                        target: Expr::Access(
                            Access {
                                list: access.list.clone(),
                                list_token: access.list_token.clone(),
                                key: access.key.clone(),
                                brackets: access.brackets.clone(),
                            }
                            .into(),
                        ),
                        list: access.list.clone(),
                        idx: access.key.clone(),
                        value,
                        list_token: access.list_token.clone(),
                        brackets: access.brackets.clone(),
                        arrow_token,
                    }
                    .into(),
                )),

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
            expr = Expr::Logical(Arc::new(Logical {
                left: expr,
                operator: LogicalOp::Or,
                right,

                token,
            }))
        }

        Ok(expr)
    }

    // logical_and → equality ( "AND" equality )*
    fn and(&mut self) -> miette::Result<Expr> {
        let mut expr = self.equality()?;

        while self.match_token(&And) {
            // get the token for spanning
            let token = self.previous().clone();

            let right = self.and()?;
            expr = Expr::Logical(Arc::new(Logical {
                left: expr,
                operator: LogicalOp::And,
                right,

                token,
            }))
        }

        Ok(expr)
    }

    fn equality(&mut self) -> miette::Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_tokens(&[BangEqual, EqualEqual]) {
            // get equality token
            let token = self.previous().clone();
            let right = self.comparison()?;
            let operator = token.to_binary_op()?;
            let left = expr;
            expr = Expr::Binary(
                Binary {
                    left,
                    operator,
                    right,
                    token,
                }
                .into(),
            )
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> miette::Result<Expr> {
        let mut expr = self.addition()?;

        while self.match_tokens(&[Greater, GreaterEqual, Less, LessEqual]) {
            // get comparison token
            let token = self.previous().clone();
            let right = self.addition()?;
            let operator = token.to_binary_op()?;
            let left = expr;
            expr = Expr::Binary(
                Binary {
                    left,
                    operator,
                    right,
                    token,
                }
                .into(),
            )
        }

        Ok(expr)
    }

    fn addition(&mut self) -> miette::Result<Expr> {
        let mut expr = self.multiplication()?;

        while self.match_tokens(&[Plus, Minus]) {
            // get addition token
            let token = self.previous().clone();
            let right = self.multiplication()?;
            let operator = token.to_binary_op()?;
            let left = expr;
            expr = Expr::Binary(
                Binary {
                    left,
                    operator,
                    right,
                    token,
                }
                .into(),
            )
        }

        Ok(expr)
    }

    fn multiplication(&mut self) -> miette::Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_tokens(&[Star, Slash, Mod]) {
            // get multiplication token
            let token = self.previous().clone();
            let right = self.unary()?;
            let operator = token.to_binary_op()?;
            let left = expr;
            expr = Expr::Binary(
                Binary {
                    left,
                    operator,
                    right,
                    token,
                }
                .into(),
            )
        }

        Ok(expr)
    }

    fn unary(&mut self) -> miette::Result<Expr> {
        if self.match_tokens(&[Not, Minus]) {
            let token = self.previous().clone();
            let right = self.unary()?;
            let operator = token.to_unary_op()?;

            let expr = Expr::Unary(
                Unary {
                    operator,
                    right,
                    token,
                }
                .into(),
            );

            Ok(expr)
        } else {
            self.access()
        }
    }

    fn access(&mut self) -> miette::Result<Expr> {
        let mut expr = self.primary()?;
        let expr_token = self.previous().clone();

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
                        help = "when indexing an array you must have a closing `]` bracket following the expression",
                        "expected ']' found {}", token.lexeme
                    )
                })?.clone();

                expr = Expr::Access(Arc::new(Access {
                    list: expr,
                    list_token: expr_token.clone(),
                    key: index,
                    brackets: (lb_token, rb_token),
                }));
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
            return Ok(Expr::Literal(Arc::new(ExprLiteral {
                value: Literal::True,
                token,
            })));
        }
        // FALSE
        if self.match_token(&False) {
            let token = self.previous().clone();
            return Ok(Expr::Literal(Arc::new(ExprLiteral {
                value: Literal::False,
                token,
            })));
        }
        // NULL
        if self.match_token(&Null) {
            let token = self.previous().clone();
            return Ok(Expr::Literal(Arc::new(ExprLiteral {
                value: Literal::Null,
                token,
            })));
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

            return Ok(Expr::Literal(
                ExprLiteral {
                    value: Literal::String(literal),
                    token,
                }
                .into(),
            ));
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

            return Ok(Expr::Literal(
                ExprLiteral {
                    value: Literal::Number(literal),
                    token,
                }
                .into(),
            ));
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
                let mut arguments_tokens = vec![lp_token.clone()];
                if !self.check(&RightParen) {
                    loop {
                        if arguments.len() >= 255 {
                            // let next_token = self.peek();
                            // todo: improve this message
                            let report = miette!(
                                // todo: finish this
                                "todo: max args for function call exceeded"
                            );
                            return Err(report);
                        }

                        let expr = self.expression()?;
                        arguments.push(expr);
                        arguments_tokens.push(self.peek().clone());

                        // we've reached the end of arguments
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

                let arguments_spans: Vec<SourceSpan> = arguments_tokens
                    .windows(2)
                    .map(|tok| tok[0].span_until_token(&tok[1]))
                    .collect();

                return Ok(Expr::ProcCall(Arc::new(ProcCall {
                    ident,
                    arguments,
                    arguments_spans,
                    token,
                    parens: (lp_token, rp_token),
                })));
            }

            // ident token
            return Ok(Expr::Variable(Arc::new(Variable { ident, token })));
        }

        // "(" expr ")"
        if self.match_token(&LeftParen) {
            let lp_token = self.previous().clone();
            let expr = self.expression()?;

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

            return Ok(Expr::Grouping(Arc::new(Grouping {
                expr,
                parens: (lp_token.clone(), rp_token.clone()),
            })));
        }

        // "[" ( expr ),* "]"
        if self.match_token(&LeftBracket) {
            let lb_token = self.previous().clone();

            let mut items = vec![];
            if !self.check(&RightBracket) {
                loop {
                    let expr = self.expression()?;
                    items.push(expr);

                    // we've reached the end of arguments
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

            return Ok(Expr::List(Arc::new(List {
                items,
                brackets: (lb_token, rb_token.clone()),
            })));
        }

        let cspan = self.previous().span_to(self.peek().span());
        let labels = vec![
            LabeledSpan::at(self.peek().span(), "primary expected here"),
            // LabeledSpan::at(cspan, "consider checking your upstream code"),
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
/// Helper methods for the `Parser2` struct.
impl Parser {
    /// Synchronizes the parser by advancing tokens until it reaches a likely
    /// starting point for a new statement or declaration.
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            // this is "good enough" for now
            // sometimes it does not recover properly
            // more robust recovery would great
            // it is worth looking into...
            match self.peek().token_type {
                Procedure | Repeat | For | If | Return | Continue | Break | Import | Export => {
                    return
                }
                _ => (),
            }

            self.advance();
        }
    }

    /// Consumes the next token if it matches the expected `token_type`.
    ///
    /// If the next token matches the expected type, it is consumed and returned.
    /// Otherwise, it reports an error using the provided `report` function.
    ///
    /// # Arguments
    ///
    /// * `token_type` - The expected token type to consume.
    /// * `report` - A closure that generates an error report when the token does not match.
    ///
    /// # Returns
    ///
    /// * `Ok(&Token)` - The consumed token if it matches the expected type.
    /// * `Err(miette::Report)` - An error report if the token does not match.
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
            Err(report(&next_token).with_source_code(self.named_source.clone()))
        }
    }

    /// Consumes consecutive semicolon tokens (`SoftSemi`).
    ///
    /// This method advances the parser while the current token is a semicolon.
    /// It helps in ignoring redundant semicolons in the input.
    fn take_semis(&mut self) {
        while self.check(&SoftSemi) {
            self.advance();
        }
    }

    /// Checks if the current token matches the given `typ` without consuming it.
    ///
    /// # Arguments
    ///
    /// * `typ` - The token type to check for.
    ///
    /// # Returns
    ///
    /// * `true` if the current token matches `typ`.
    /// * `false` otherwise.
    fn check(&self, typ: &TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }

        self.peek().token_type() == typ
    }

    /// Confirms that the previous token matches the expected `typ`.
    ///
    /// # Arguments
    ///
    /// * `typ` - The expected token type of the previous token.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the previous token matches `typ`.
    /// * `Err(miette::Report)` if it does not match.
    ///
    /// # Errors
    ///
    /// Returns an error report indicating the mismatch between the expected and actual token types.
    fn confirm(&self, typ: &TokenType) -> miette::Result<()> {
        let previous = self.previous();

        if &previous.token_type != typ {
            return Err(miette!(
                "Expected previous token to be {:?}, but found {:?}.",
                typ,
                previous.token_type
            ));
        }

        Ok(())
    }

    /// Attempts to match and consume the next token if it matches `token_type`.
    ///
    /// # Arguments
    ///
    /// * `token_type` - The token type to match and consume.
    ///
    /// # Returns
    ///
    /// * `true` if the token was matched and consumed.
    /// * `false` otherwise.
    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            return true;
        }
        false
    }

    /// Attempts to match and consume any one of the specified token types.
    ///
    /// # Arguments
    ///
    /// * `types` - A slice of token types to match and consume.
    ///
    /// # Returns
    ///
    /// * `true` if any of the token types were matched and consumed.
    /// * `false` otherwise.
    fn match_tokens(&mut self, types: &[TokenType]) -> bool {
        for ty in types {
            if self.match_token(ty) {
                return true;
            }
        }
        false
    }

    /// Consumes the next token and returns it.
    ///
    /// Advances the parser's current position and returns the token that was just consumed.
    ///
    /// # Returns
    ///
    /// * `&Token` - The token that was consumed.
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }

        self.previous()
    }

    /// Returns the current token without consuming it.
    ///
    /// # Returns
    ///
    /// * `&Token` - The current token.
    ///
    /// # Panics
    ///
    /// Panics if there is no token to peek at (should not happen if `is_at_end` is correctly used).
    fn peek(&self) -> &Token {
        self.tokens
            .get(self.current)
            .expect("Internal error: attempted to peek token when there is no token to peek.")
    }

    /// Returns the previously consumed token.
    ///
    /// # Returns
    ///
    /// * `&Token` - The previously consumed token.
    ///
    /// # Panics
    ///
    /// Panics if there is no previous token (i.e., if at the start of the token stream).
    fn previous(&self) -> &Token {
        if self.current == 0 {
            panic!("Internal error: there is no previous token.");
        }

        self.tokens
            .get(self.current - 1)
            .expect("Internal error: failed to retrieve previous token.")
    }

    /// Determines if the parser has reached the end of the token stream.
    ///
    /// # Returns
    ///
    /// * `true` if the current token is `Eof`.
    /// * `false` otherwise.
    fn is_at_end(&self) -> bool {
        self.peek().token_type == Eof
    }
}

pub(super) mod warning {
    use crate::parser::Parser;
    use miette::Report;

    impl Parser {
        pub(super) fn warning(&mut self, report: Report) {
            self._warnings
                .push(report.with_source_code(self.named_source.clone()))
        }
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
                let _report = report_handler();
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
                let _report = report_handler();
                panic!()
            }
        }
    }
}
