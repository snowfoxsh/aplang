use crate::token::Token;
use std::sync::Arc;

// To facilitate better error handling down the line,
// we are going to store the tokens that the thing came from
// so we can report back to them later

#[derive(Debug, Clone)]
pub struct Ast {
    pub source: Arc<str>,
    pub program: Vec<Stmt>,
}

type Ident = String;

#[derive(Debug, Clone)]
pub enum Stmt {
    Expr {
        expr: Expr,
    },
    If {
        condition: Expr,
        then_branch: Box<Stmt>,

        else_branch: Option<Box<Stmt>>,

        if_token: Token,
        else_token: Option<Token>,
    },
    RepeatTimes {
        count: Expr,
        body: Box<Stmt>,

        repeat_token: Token,
        times_token: Token,
    },
    RepeatUntil {
        condition: Expr,
        body: Box<Stmt>,

        repeat_token: Token,
        until_token: Token,
    },
    ForEach {
        item: Ident,
        list: Expr,
        body: Box<Stmt>,

        item_token: Token,
        for_token: Token,
        each_token: Token,
        in_token: Token,
    },
    ProcDeclaration {
        name: Ident,
        params: Vec<Ident>,
        body: Box<Stmt>,

        proc_token: Token,
        name_token: Token,
        params_tokens: Vec<Token>,
    },
    Block {
        lb_token: Token,
        statements: Vec<Stmt>,
        rb_token: Token,
    },
    Return {
        token: Token,
        data: Option<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal {
        value: Literal,
        token: Token,
    },
    Binary {
        left: Box<Expr>,
        operator: BinaryOp,
        right: Box<Expr>,
        token: Token,
    },
    Logical {
        left: Box<Expr>,
        operator: LogicalOp,
        right: Box<Expr>,
        token: Token,
    },
    Unary {
        operator: UnaryOp,
        right: Box<Expr>,
        token: Token,
    },
    Grouping {
        expr: Box<Expr>,
        parens: (Token, Token),
    },
    ProcCall {
        ident: String,
        arguments: Vec<Expr>,

        token: Token,
        parens: (Token, Token),
    },
    Access {
        list: Box<Expr>,
        key: Box<Expr>,
        brackets: (Token, Token),
    },
    List {
        items: Vec<Expr>,
        brackets: (Token, Token),
    },
    Variable {
        ident: String,
        token: Token,
    },
    Assign {
        target: Ident,
        value: Box<Expr>,

        ident_token: Token,
        arrow_token: Token,
    },
    Set {
        target: Box<Expr>,
        value: Box<Expr>,

        arrow_token: Token,
    },
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Null,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    EqualEqual,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Plus,
    Minus,
    Star,
    Slash,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Minus,
    Not,
}

#[derive(Debug, Clone)]
pub enum LogicalOp {
    Or,
    And,
}

pub mod pretty {
    use crate::ast::{Ast, BinaryOp, Expr, Ident, Literal, LogicalOp, Stmt, UnaryOp};
    use std::fmt::{write, Display};
    impl TreePrinter for Ast {
        fn node_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreePrinter>> + '_> {
            let statements_printer: Vec<_> = self
                .program
                .iter()
                .map(|stmt| Box::new(stmt.clone()) as Box<dyn TreePrinter>)
                .collect();
            Box::new(statements_printer.into_iter())
        }

        fn node(&self) -> Box<dyn fmt::Display> {
            // Box::new(format!("Ast: {}", self.source))
            Box::new(format!("{}", "Program:".bold()))
        }
    }

    impl TreePrinter for Stmt {
        fn node_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreePrinter>> + '_> {
            match self {
                Stmt::Expr { expr } => {
                    Box::new(vec![Box::new(expr.clone()) as Box<dyn TreePrinter>].into_iter())
                }
                Stmt::If {
                    condition,
                    then_branch,
                    else_branch,
                    ..
                } => {
                    let mut children = vec![
                        Box::new(condition.clone()) as Box<dyn TreePrinter>,
                        Box::new((**then_branch).clone()) as Box<dyn TreePrinter>,
                    ];
                    if let Some(else_branch) = else_branch {
                        children.push(Box::new((**else_branch).clone()) as Box<dyn TreePrinter>);
                    }
                    Box::new(children.into_iter())
                }
                Stmt::RepeatTimes { count, body, .. } => Box::new(
                    vec![
                        Box::new(count.clone()) as Box<dyn TreePrinter>,
                        Box::new((**body).clone()) as Box<dyn TreePrinter>,
                    ]
                    .into_iter(),
                ),
                Stmt::RepeatUntil {
                    condition, body, ..
                } => Box::new(
                    vec![
                        Box::new(condition.clone()) as Box<dyn TreePrinter>,
                        Box::new((**body).clone()) as Box<dyn TreePrinter>,
                    ]
                    .into_iter(),
                ),
                Stmt::ForEach {
                    item: _,
                    list,
                    item_token: _,
                    for_token: _,
                    ..
                } => Box::new(vec![Box::new(list.clone()) as Box<dyn TreePrinter>].into_iter()),
                Stmt::ProcDeclaration {
                    name: _,
                    params: _,
                    body,
                    proc_token: _,
                    name_token: _,
                    params_tokens: _,
                } => Box::new(vec![Box::new((**body).clone()) as Box<dyn TreePrinter>].into_iter()),
                Stmt::Block {
                    lb_token: _,
                    statements,
                    rb_token: _,
                } => {
                    let children = statements
                        .iter()
                        .map(|stmt| Box::new(stmt.clone()) as Box<dyn TreePrinter>)
                        .collect::<Vec<_>>();
                    Box::new(children.into_iter())
                }
                Stmt::Return { token: _, data } => {
                    if let Some(expr) = data {
                        Box::new(vec![Box::new(expr.clone()) as Box<dyn TreePrinter>].into_iter())
                    } else {
                        Box::new(std::iter::empty())
                    }
                }
            }
        }

        fn node(&self) -> Box<dyn fmt::Display> {
            Box::new(format!("{}", self))
        }
    }

    impl TreePrinter for Expr {
        fn node_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreePrinter>> + '_> {
            match self {
                Expr::Binary { left, right, .. } | Expr::Logical { left, right, .. } => Box::new(
                    vec![
                        Box::new((**left).clone()) as Box<dyn TreePrinter>,
                        Box::new((**right).clone()) as Box<dyn TreePrinter>,
                    ]
                    .into_iter(),
                ),
                Expr::Unary { right, .. } | Expr::Grouping { expr: right, .. } => {
                    Box::new(vec![Box::new((**right).clone()) as Box<dyn TreePrinter>].into_iter())
                }
                Expr::ProcCall { arguments, .. } => {
                    let cloned_args = arguments
                        .iter()
                        .map(|arg| Box::new(arg.clone()) as Box<dyn TreePrinter>)
                        .collect::<Vec<_>>();
                    Box::new(cloned_args.into_iter())
                }
                Expr::Access {
                    list,
                    key: accessor,
                    ..
                } => Box::new(
                    vec![
                        Box::new((**list).clone()) as Box<dyn TreePrinter>,
                        Box::new((**accessor).clone()) as Box<dyn TreePrinter>,
                    ]
                    .into_iter(),
                ),
                Expr::List { items, .. } => {
                    let cloned_items = items
                        .iter()
                        .map(|item| Box::new(item.clone()) as Box<dyn TreePrinter>)
                        .collect::<Vec<_>>();
                    Box::new(cloned_items.into_iter())
                }
                Expr::Literal { .. } | Expr::Variable { .. } => {
                    // These are leaf nodes and do not have children
                    Box::new(std::iter::empty())
                }
                // Handle Assign and Set variants
                Expr::Assign {
                    target,
                    value,
                    ident_token,
                    arrow_token,
                } => {
                    Box::new(vec![Box::new((**value).clone()) as Box<dyn TreePrinter>].into_iter())
                }
                Expr::Set { value, target, .. } => Box::new(
                    vec![
                        Box::new((**target).clone()) as Box<dyn TreePrinter>,
                        Box::new((**value).clone()) as Box<dyn TreePrinter>,
                    ]
                    .into_iter(),
                ),
            }
        }

        fn node(&self) -> Box<dyn Display> {
            Box::new(format!("{}", self))
        }
    }

    pub trait TreePrinter {
        fn node_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreePrinter>> + '_>;

        fn node(&self) -> Box<dyn Display>;

        fn print_tree_base(&self, prefix: &str, last: bool) -> String {
            let mut result = format!(
                "{}{}{}\n",
                prefix,
                if last { "└── " } else { "├── " },
                self.node()
            );
            let prefix_child = if last { "    " } else { "│   " };
            let children: Vec<_> = self.node_children().collect();
            for (i, child) in children.iter().enumerate() {
                let last_child = i == children.len() - 1;
                result += &child.print_tree_base(&(prefix.to_owned() + prefix_child), last_child);
            }
            result
        }

        fn header(&self) -> Box<dyn Display> {
            Box::<String>::default()
        }

        fn print_tree(&self) -> String {
            let len = self.node_children().count();
            let tree = self
                .node_children()
                .enumerate()
                .map(|(i, child)| {
                    let last = len - 1 == i;
                    child.print_tree_base("", last)
                })
                .collect::<String>();

            format!("{}{}\n{}", String::default(), self.node(), tree)
        }
    }

    use owo_colors::OwoColorize;
    use std::fmt;

    impl fmt::Display for Expr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Expr::Literal { value, .. } => write!(f, "Literal ({})", value),
                Expr::Binary { operator, .. } => write!(f, "Binary ({})", operator),
                Expr::Logical { operator, .. } => write!(f, "Logical ({})", operator),
                Expr::Unary { operator, .. } => write!(f, "Unary ({})", operator),
                Expr::Grouping { .. } => write!(f, "Group"),
                Expr::ProcCall { ident, .. } => write!(f, "Call ({})", ident),
                Expr::Access { list, key, .. } => write!(f, "Access {}[{}]", list, key),
                Expr::List { .. } => write!(f, "List"),
                Expr::Variable { ident, .. } => write!(f, "Variable ({})", ident),
                Expr::Assign { target, value, .. } => write!(f, "Assign ({} <- {})", target, value),
                Expr::Set { target, value, .. } => write!(f, "Set ({}[{})", target, value),
            }
        }
    }

    impl fmt::Display for Literal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Literal::Number(n) => write!(f, "{}", n),
                Literal::String(s) => write!(f, "\"{}\"", s),
                Literal::True => write!(f, "true"),
                Literal::False => write!(f, "false"),
                Literal::Null => write!(f, "null"),
            }
        }
    }

    impl fmt::Display for BinaryOp {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let op = match self {
                BinaryOp::EqualEqual => "==",
                BinaryOp::NotEqual => "!=",
                BinaryOp::Less => "<",
                BinaryOp::LessEqual => "<=",
                BinaryOp::Greater => ">",
                BinaryOp::GreaterEqual => ">=",
                BinaryOp::Plus => "+",
                BinaryOp::Minus => "-",
                BinaryOp::Star => "*",
                BinaryOp::Slash => "/",
            };
            write!(f, "{}", op)
        }
    }

    impl fmt::Display for Stmt {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Stmt::Expr { expr } => write!(f, "Expr"),
                Stmt::If {
                    condition,
                    then_branch: _,
                    else_branch: _,
                    if_token: _,
                    else_token: _,
                } => write!(f, "If(Condition: {})", condition),
                Stmt::RepeatTimes {
                    count,
                    body: _,
                    repeat_token: _,
                    times_token: _,
                } => write!(f, "RepeatTimes(Count: {})", count),
                Stmt::RepeatUntil { condition, .. } => {
                    write!(f, "RepeatUntil(Condition: {})", condition)
                }
                Stmt::ForEach { item, list, .. } => {
                    write!(f, "ForEach(Item: {}, List: {})", item, list)
                }
                Stmt::ProcDeclaration {
                    name,
                    params,
                    body: _,
                    proc_token: _,
                    name_token: _,
                    params_tokens: _,
                } => write!(f, "ProcDeclaration(Name: {}, Params: {:?})", name, params),
                Stmt::Block {
                    lb_token: _,
                    statements: _,
                    rb_token: _,
                } => write!(f, "Block"),
                Stmt::Return { token: _, data } => match data {
                    Some(expr) => write!(f, "Return({})", expr),
                    None => write!(f, "Return"),
                },
            }
        }
    }

    impl fmt::Display for UnaryOp {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let op = match self {
                UnaryOp::Minus => "-",
                UnaryOp::Not => "!",
            };
            write!(f, "{}", op)
        }
    }

    impl fmt::Display for LogicalOp {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let op = match self {
                LogicalOp::And => "and",
                LogicalOp::Or => "or",
            };
            write!(f, "{}", op)
        }
    }
}
