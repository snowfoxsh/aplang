use std::sync::Arc;
use crate::token::Token;


// To facilitate better error handling down the line,  
// we are going to store the tokens that the thing came from
// so we can report back to them later


struct Ast {
    source: Arc<str>,
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
        token: Token
    },
    Grouping {
        expr: Box<Expr>,
        parens: (Token, Token),
    },
    ProcCall {
        ident: String,
        arguments: Arguments,
        
        token: Token,
        parens: (Token, Token)
    },
    Access {
        list: Box<Expr>,
        accessor: Box<Expr>,
        brackets: (Token, Token),
    },
    List {
        items: Vec<Expr>,
        brackets: (Token, Token),
    },
    Variable {
        ident: String,
        token: Token
    }
}

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    True,
    False,
    Null
}

#[derive(Debug, Clone)]
pub struct Arguments {
    arguments: Vec<Expr>,

    tokens: Vec<Token>,
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
    use std::fmt::Display;
    use crate::ast::{BinaryOp, Expr, Literal, LogicalOp, UnaryOp};

    impl TreePrinter for Expr {
        fn node_children(&self) -> Box<dyn Iterator<Item=Box<dyn TreePrinter>> + '_> {
            match self {
                Expr::Binary { left, right, .. } |
                Expr::Logical { left, right, .. } => {
                    Box::new(vec![Box::new((**left).clone()) as Box<dyn TreePrinter>, Box::new((**right).clone()) as Box<dyn TreePrinter>].into_iter())
                },
                Expr::Unary { right, .. } |
                Expr::Grouping { expr: right, .. } => {
                    Box::new(vec![Box::new((**right).clone()) as Box<dyn TreePrinter>].into_iter())
                },
                Expr::ProcCall { arguments, .. } => {
                    let cloned_args = arguments.arguments.iter().map(|arg| Box::new(arg.clone()) as Box<dyn TreePrinter>).collect::<Vec<_>>();
                    Box::new(cloned_args.into_iter())
                },
                Expr::Access { list, accessor, .. } => {
                    Box::new(vec![Box::new((**list).clone()) as Box<dyn TreePrinter>, Box::new((**accessor).clone()) as Box<dyn TreePrinter>].into_iter())
                },
                Expr::List { items, .. } => {
                    let cloned_items = items.iter().map(|item| Box::new(item.clone()) as Box<dyn TreePrinter>).collect::<Vec<_>>();
                    Box::new(cloned_items.into_iter())
                },
                Expr::Literal { .. } |
                Expr::Variable { .. } => {
                    // These are leaf nodes and do not have children
                    Box::new(std::iter::empty())
                },
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
            let mut result = format!("{}{}{}\n", prefix, if last { "└── " } else { "├── " }, self.node());
            let prefix_child = if last { "    " } else { "│   " };
            let children: Vec<_> = self.node_children().collect();
            for (i, child) in children.iter().enumerate() {
                let last_child = i == children.len() - 1;
                result += &child.print_tree_base(&(prefix.to_owned() + prefix_child), last_child);
            }
            result
        }

        fn print_tree(&self) -> String {
            let len = self.node_children().count();
            let tree = self.node_children().enumerate().map(|(i, child)| {
                let last = len - 1 == i;
                child.print_tree_base("", last)
            }).collect::<String>();

            format!("{}\n{}", self.node(), tree)
        }
    }

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
                Expr::Access { .. } => write!(f, "Access"),
                Expr::List { .. } => write!(f, "List"),
                Expr::Variable { ident, .. } => write!(f, "Variable ({})", ident),
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
