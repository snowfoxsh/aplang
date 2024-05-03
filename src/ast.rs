use crate::token::Token;
use std::ops::Deref;
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
    Expr(Arc<Expr>),

    IfStmt(Arc<IfStmt>),

    RepeatTimes(Arc<RepeatTimes>),

    RepeatUntil(Arc<RepeatUntil>),

    ForEach(Arc<ForEach>),

    ProcDeclaration(Arc<ProcDeclaration>),

    Block(Arc<Block>),

    Return(Arc<Return>),
}
#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_branch: Stmt,
    pub else_branch: Option<Stmt>,

    pub if_token: Token,
    pub else_token: Option<Token>,
}
#[derive(Debug, Clone)]
pub struct RepeatTimes {
    pub count: Expr,
    pub body: Stmt,

    pub repeat_token: Token,
    pub times_token: Token,
}
#[derive(Debug, Clone)]
pub struct RepeatUntil {
    pub condition: Expr,
    pub body: Stmt,

    pub repeat_token: Token,
    pub until_token: Token,
}
#[derive(Debug, Clone)]
pub struct ForEach {
    pub item: Variable,
    pub list: Expr,
    pub body: Stmt,

    pub item_token: Token,
    pub for_token: Token,
    pub each_token: Token,
    pub in_token: Token,
}
#[derive(Debug, Clone)]
pub struct ProcDeclaration {
    pub name: Ident,
    pub params: Vec<Variable>,
    pub body: Stmt,

    pub proc_token: Token,
    pub name_token: Token,
}
#[derive(Debug, Clone)]
pub struct Block {
    pub lb_token: Token,
    pub statements: Vec<Stmt>,
    pub rb_token: Token,
}
#[derive(Debug, Clone)]
pub struct Return {
    pub token: Token,
    pub data: Option<Expr>,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Arc<ExprLiteral>),
    Binary(Arc<Binary>),
    Logical(Arc<Logical>),

    Unary(Arc<Unary>),

    Grouping(Arc<Grouping>),

    ProcCall(Arc<ProcCall>),

    Access(Arc<Access>),

    List(Arc<List>),

    Variable(Arc<Variable>),

    Assign(Arc<Assignment>),

    Set(Arc<Set>),
}
#[derive(Debug, Clone)]
pub struct ExprLiteral {
    pub value: Literal,
    pub token: Token,
}
#[derive(Debug, Clone)]
pub struct Binary {
    pub left: Expr,
    pub operator: BinaryOp,
    pub right: Expr,
    pub token: Token,
}
#[derive(Debug, Clone)]
pub struct Logical {
    pub left: Expr,
    pub operator: LogicalOp,
    pub right: Expr,
    pub token: Token,
}
#[derive(Debug, Clone)]
pub struct Unary {
    pub operator: UnaryOp,
    pub right: Expr,
    pub token: Token,
}
#[derive(Debug, Clone)]
pub struct Grouping {
    pub expr: Expr,
    pub parens: (Token, Token),
}
#[derive(Debug, Clone)]
pub struct ProcCall {
    pub ident: String,
    pub arguments: Vec<Expr>,

    pub token: Token,
    pub parens: (Token, Token),
}
#[derive(Debug, Clone)]
pub struct Access {
    pub list: Expr,
    pub key: Expr,
    pub brackets: (Token, Token),
}
#[derive(Debug, Clone)]
pub struct List {
    pub items: Vec<Expr>,
    pub brackets: (Token, Token),
}
#[derive(Debug, Clone)]
pub struct Variable {
    pub ident: String,
    pub token: Token,
}
#[derive(Debug, Clone)]
pub struct Assignment {
    pub target: Arc<Variable>,
    pub value: Expr,

    pub ident_token: Token,
    pub arrow_token: Token,
}
#[derive(Debug, Clone)]
pub struct Set {
    pub target: Expr,
    pub value: Expr,

    pub arrow_token: Token,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogicalOp {
    Or,
    And,
}

pub mod pretty {
    use super::*;
    use std::fmt;
    use std::fmt::{Display, Formatter};

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

    impl TreePrinter for Ast {
        fn node_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreePrinter>> + '_> {
            Box::new(
                self.program
                    .iter()
                    .map(|stmt| Box::new(stmt.clone()) as Box<dyn TreePrinter>),
            )
        }

        fn node(&self) -> Box<dyn Display> {
            Box::new(format!("Ast (Source: {:?})", self.source))
        }
    }

    impl TreePrinter for Stmt {
        fn node_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreePrinter>> + '_> {
            match self {
                Stmt::Expr(expr) => Box::new(std::iter::once(
                    Box::new(expr.deref().clone()) as Box<dyn TreePrinter>
                )),
                Stmt::IfStmt(if_stmt) => Box::new(
                    std::iter::once(Box::new(if_stmt.condition.clone()) as Box<dyn TreePrinter>)
                        .chain(std::iter::once(
                            Box::new(if_stmt.then_branch.clone()) as Box<dyn TreePrinter>
                        ))
                        .chain(if_stmt.else_branch.as_ref().map(|else_branch| {
                            Box::new(else_branch.clone()) as Box<dyn TreePrinter>
                        })),
                ),
                Stmt::RepeatTimes(repeat_times) => Box::new(
                    std::iter::once(Box::new(repeat_times.count.clone()) as Box<dyn TreePrinter>)
                        .chain(std::iter::once(
                            Box::new(repeat_times.body.clone()) as Box<dyn TreePrinter>
                        )),
                ),
                Stmt::RepeatUntil(repeat_until) => Box::new(
                    std::iter::once(
                        Box::new(repeat_until.condition.clone()) as Box<dyn TreePrinter>
                    )
                    .chain(std::iter::once(
                        Box::new(repeat_until.body.clone()) as Box<dyn TreePrinter>,
                    )),
                ),
                Stmt::ForEach(for_each) => Box::new(
                    std::iter::once(Box::new(for_each.list.clone()) as Box<dyn TreePrinter>).chain(
                        std::iter::once(Box::new(for_each.body.clone()) as Box<dyn TreePrinter>),
                    ),
                ),
                Stmt::ProcDeclaration(proc_decl) => Box::new(std::iter::once(Box::new(
                    proc_decl.body.clone(),
                )
                    as Box<dyn TreePrinter>)),
                Stmt::Block(block) => Box::new(
                    block
                        .statements
                        .iter()
                        .map(|stmt| Box::new(stmt.clone()) as Box<dyn TreePrinter>)
                        .collect::<Vec<_>>()
                        .into_iter(),
                ),
                Stmt::Return(return_stmt) => Box::new(
                    return_stmt
                        .data
                        .as_ref()
                        .map(|expr| Box::new(expr.clone()) as Box<dyn TreePrinter>)
                        .into_iter(),
                ),
                // .into_iter()
                // .collect::<Vec<_>>().into_iter()
            }
        }

        fn node(&self) -> Box<dyn Display> {
            Box::new(format!("{}", self)) // Implement Display for Stmt or adjust this to a custom string representation
        }
    }

    impl TreePrinter for Expr {
        fn node_children(&self) -> Box<dyn Iterator<Item = Box<dyn TreePrinter>> + '_> {
            match self {
                Expr::Binary(binary) => Box::new(
                    std::iter::once(Box::new(binary.left.clone()) as Box<dyn TreePrinter>).chain(
                        std::iter::once(Box::new(binary.right.clone()) as Box<dyn TreePrinter>),
                    ),
                ),
                Expr::Logical(logical) => Box::new(
                    std::iter::once(Box::new(logical.left.clone()) as Box<dyn TreePrinter>).chain(
                        std::iter::once(Box::new(logical.right.clone()) as Box<dyn TreePrinter>),
                    ),
                ),
                Expr::Unary(unary) => Box::new(std::iter::once(
                    Box::new(unary.right.clone()) as Box<dyn TreePrinter>
                )),
                Expr::Grouping(grouping) => Box::new(std::iter::once(
                    Box::new(grouping.expr.clone()) as Box<dyn TreePrinter>,
                )),
                Expr::ProcCall(proc_call) => Box::new(
                    proc_call
                        .arguments
                        .iter()
                        .map(|arg| Box::new(arg.clone()) as Box<dyn TreePrinter>)
                        .collect::<Vec<_>>()
                        .into_iter(),
                ),
                Expr::Access(access) => Box::new(
                    std::iter::once(Box::new(access.list.clone()) as Box<dyn TreePrinter>).chain(
                        std::iter::once(Box::new(access.key.clone()) as Box<dyn TreePrinter>),
                    ),
                ),
                Expr::List(list) => Box::new(
                    list.items
                        .iter()
                        .map(|item| Box::new(item.clone()) as Box<dyn TreePrinter>)
                        .collect::<Vec<_>>()
                        .into_iter(),
                ),
                Expr::Variable(_) | Expr::Literal(_) => Box::new(std::iter::empty()),
                Expr::Assign(assignment) => Box::new(std::iter::once(Box::new(
                    assignment.value.clone(),
                )
                    as Box<dyn TreePrinter>)),
                Expr::Set(set) => Box::new(
                    std::iter::once(Box::new(set.target.clone()) as Box<dyn TreePrinter>).chain(
                        std::iter::once(Box::new(set.value.clone()) as Box<dyn TreePrinter>),
                    ),
                ),
            }
        }

        fn node(&self) -> Box<dyn Display> {
            Box::new(format!("{}", self)) // Implement Display for Expr or adjust this to a custom string representation
        }
    }

    impl fmt::Display for Expr {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Expr::Literal(literal) => write!(f, "{}", literal.value),
                Expr::Binary(binary) => {
                    write!(f, "({} {} {})", binary.left, binary.operator, binary.right)
                }
                Expr::Logical(logical) => write!(
                    f,
                    "({} {} {})",
                    logical.left, logical.operator, logical.right
                ),
                Expr::Unary(unary) => write!(f, "({}{})", unary.operator, unary.right),
                Expr::Grouping(grouping) => write!(f, "(group {})", grouping.expr),
                Expr::ProcCall(proc_call) => {
                    let args = proc_call
                        .arguments
                        .iter()
                        .map(|arg| format!("{}", arg))
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(f, "{}({})", proc_call.ident, args)
                }
                Expr::Access(access) => write!(f, "{}[{}]", access.list, access.key),
                Expr::List(list) => {
                    let items = list
                        .items
                        .iter()
                        .map(|item| format!("{}", item))
                        .collect::<Vec<_>>()
                        .join(", ");
                    write!(f, "[{}]", items)
                }
                Expr::Variable(variable) => write!(f, "{}", variable.ident),
                Expr::Assign(assignment) => {
                    write!(f, "{} <- {}", assignment.target, assignment.value)
                }
                Expr::Set(set) => write!(f, "{}[{}] = {}", set.target, set.arrow_token, set.value),
            }
        }
    }

    impl fmt::Display for Stmt {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Stmt::Expr(expr) => write!(f, "{}", expr),
                Stmt::IfStmt(if_stmt) => {
                    let else_part = if let Some(else_branch) = &if_stmt.else_branch {
                        format!(" else {}", else_branch)
                    } else {
                        String::new()
                    };
                    write!(
                        f,
                        "if {} then {}{}",
                        if_stmt.condition, if_stmt.then_branch, else_part
                    )
                }
                Stmt::RepeatTimes(repeat_times) => write!(
                    f,
                    "repeat {} times {}",
                    repeat_times.count, repeat_times.body
                ),
                Stmt::RepeatUntil(repeat_until) => write!(
                    f,
                    "repeat until {} {}",
                    repeat_until.condition, repeat_until.body
                ),
                Stmt::ForEach(for_each) => write!(
                    f,
                    "for {} in {} do {}",
                    for_each.item, for_each.list, for_each.body
                ),
                Stmt::ProcDeclaration(proc_decl) => {
                    // let params = proc_decl.pjoin(", ");
                    let params= proc_decl.params.iter()
                        .map(|var| var.ident.clone())
                        .collect::<Vec<_>>()
                        .join(", ");
                    
                    write!(
                        f,
                        "procedure {}({}) {}",
                        proc_decl.name, params, proc_decl.body
                    )
                }
                Stmt::Block(block) => {
                    let statements = block
                        .statements
                        .iter()
                        .map(|stmt| format!("{}", stmt))
                        .collect::<Vec<_>>()
                        .join("; ");
                    write!(f, "{{ {} }}", statements)
                }
                Stmt::Return(return_stmt) => match &return_stmt.data {
                    Some(data) => write!(f, "return {}", data),
                    None => write!(f, "return"),
                },
            }
        }
    }

    impl fmt::Display for Variable {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.ident)
        }
    }
    //     use owo_colors::OwoColorize;
    //     use std::fmt;

    //     impl fmt::Display for Expr {
    //         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //             match self {
    //                 Expr::Literal { value, .. } => write!(f, "Literal ({})", value),
    //                 Expr::Binary { operator, .. } => write!(f, "Binary ({})", operator),
    //                 Expr::Logical { operator, .. } => write!(f, "Logical ({})", operator),
    //                 Expr::Unary { operator, .. } => write!(f, "Unary ({})", operator),
    //                 Expr::Grouping { .. } => write!(f, "Group"),
    //                 Expr::ProcCall { ident, .. } => write!(f, "Call ({})", ident),
    //                 Expr::Access { list, key, .. } => write!(f, "Access {}[{}]", list, key),
    //                 Expr::List { .. } => write!(f, "List"),
    //                 Expr::Variable { ident, .. } => write!(f, "Variable ({})", ident),
    //                 Expr::Assign { target, value, .. } => write!(f, "Assign ({} <- {})", target, value),
    //                 Expr::Set { target, value, .. } => write!(f, "Set ({}[{})", target, value),
    //             }
    //         }
    //     }

    //     impl fmt::Display for Literal {
    //         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //             match self {
    //                 Literal::Number(n) => write!(f, "{}", n),
    //                 Literal::String(s) => write!(f, "\"{}\"", s),
    //                 Literal::True => write!(f, "true"),
    //                 Literal::False => write!(f, "false"),
    //                 Literal::Null => write!(f, "null"),
    //             }
    //         }
    //     }

    //     impl fmt::Display for Stmt {
    //         fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    //             match self {
    //                 Stmt::Expr { expr } => write!(f, "Expr"),
    //                 Stmt::If {
    //                     condition,
    //                     then_branch: _,
    //                     else_branch: _,
    //                     if_token: _,
    //                     else_token: _,
    //                 } => write!(f, "If(Condition: {})", condition),
    //                 Stmt::RepeatTimes {
    //                     count,
    //                     body: _,
    //                     repeat_token: _,
    //                     times_token: _,
    //                 } => write!(f, "RepeatTimes(Count: {})", count),
    //                 Stmt::RepeatUntil { condition, .. } => {
    //                     write!(f, "RepeatUntil(Condition: {})", condition)
    //                 }
    //                 Stmt::ForEach { item, list, .. } => {
    //                     write!(f, "ForEach(Item: {}, List: {})", item, list)
    //                 }
    //                 Stmt::ProcDeclaration {
    //                     name,
    //                     params,
    //                     body: _,
    //                     proc_token: _,
    //                     name_token: _,
    //                     params_tokens: _,
    //                 } => write!(f, "ProcDeclaration(Name: {}, Params: {:?})", name, params),
    //                 Stmt::Block {
    //                     lb_token: _,
    //                     statements: _,
    //                     rb_token: _,
    //                 } => write!(f, "Block"),
    //                 Stmt::Return { token: _, data } => match data {
    //                     Some(expr) => write!(f, "Return({})", expr),
    //                     None => write!(f, "Return"),
    //                 },
    //             }
    //         }
    //     }

    impl fmt::Display for Literal {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Literal::Number(num) => write!(f, "{}", num),
                Literal::String(s) => write!(f, "\"{}\"", s), // Enclose strings in quotes
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

#[macro_export]
macro_rules! BinaryOp [
    [==] => [crate::ast::BinaryOp::EqualEqual];
    [!=] => [crate::ast::BinaryOp::NotEqual];
    [<] => [crate::ast::BinaryOp::Less];
    [<=] => [crate::ast::BinaryOp::LessEqual];
    [>] => [crate::ast::BinaryOp::Greater];
    [>=] => [crate::ast::BinaryOp::GreaterEqual];
    [+] => [crate::ast::BinaryOp::Plus];
    [-] => [crate::ast::BinaryOp::Minus];
    [*] => [crate::ast::BinaryOp::Star];
    [/] => [crate::ast::BinaryOp::Slash];
];
