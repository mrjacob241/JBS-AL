use crate::runtime::{Completion, JsError};
use num_bigint::BigInt;

use super::lexer::{Lexer, Token};

#[derive(Clone, Debug)]
pub enum Stmt {
    Var(BindingPattern, Expr),
    Function(String, Vec<FormalParameter>, Vec<Stmt>),
    Block(Vec<Stmt>),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
    Switch(Expr, Vec<SwitchCase>),
    With(Expr, Box<Stmt>),
    For(Option<ForInit>, Option<Expr>, Option<Expr>, Box<Stmt>),
    ForOf(String, Expr, Box<Stmt>),
    ForIn(String, Expr, Box<Stmt>),
    Try(
        Vec<Stmt>,
        Option<(Option<String>, Vec<Stmt>)>,
        Option<Vec<Stmt>>,
    ),
    Break,
    Continue,
    Return(Option<Expr>),
    Throw(Expr),
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub struct SwitchCase {
    pub test: Option<Expr>,
    pub consequent: Vec<Stmt>,
}

#[derive(Clone, Debug)]
pub enum ForInit {
    Var(BindingPattern, Expr),
    Expr(Expr),
}

#[derive(Clone, Debug)]
pub enum BindingPattern {
    Identifier(String),
    Array(Vec<Option<String>>),
    Object(Vec<String>),
}

impl BindingPattern {
    pub fn identifier_name(&self) -> Option<&str> {
        match self {
            BindingPattern::Identifier(name) => Some(name),
            BindingPattern::Array(_) | BindingPattern::Object(_) => None,
        }
    }

    pub fn into_identifier(self) -> Option<String> {
        match self {
            BindingPattern::Identifier(name) => Some(name),
            BindingPattern::Array(_) | BindingPattern::Object(_) => None,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Expr {
    Identifier(String),
    Number(f64),
    BigInt(BigInt),
    String(String),
    RegExp(String, String),
    Boolean(bool),
    Null,
    Undefined,
    ArrayHole,
    Object(Vec<ObjectProperty>),
    Array(Vec<Expr>),
    Function(Option<String>, Vec<FormalParameter>, Vec<Stmt>),
    Arrow(Vec<FormalParameter>, Vec<Stmt>),
    NewTarget,
    Member(Box<Expr>, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    New(Box<Expr>, Vec<Expr>),
    Void(Box<Expr>),
    TypeOf(Box<Expr>),
    Delete(Box<Expr>),
    Not(Box<Expr>),
    Pos(Box<Expr>),
    Neg(Box<Expr>),
    Update(Box<Expr>, bool, bool),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
    Logical(Box<Expr>, LogicalOp, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
    Comma(Vec<Expr>),
    Assign(Box<Expr>, AssignOp, Box<Expr>),
    Equal(Box<Expr>, Box<Expr>, bool, bool),
}

#[derive(Clone, Debug)]
pub struct ObjectProperty {
    pub key: Expr,
    pub kind: ObjectPropertyKind,
}

#[derive(Clone, Debug)]
pub enum ObjectPropertyKind {
    Data(Expr),
    Get(Expr),
    Set(Expr),
    Spread(Expr),
}

#[derive(Clone, Debug)]
pub struct FormalParameter {
    pub name: String,
    pub default: Option<Expr>,
}

impl FormalParameter {
    pub fn simple(name: String) -> Self {
        Self {
            name,
            default: None,
        }
    }

    pub fn is_simple(&self) -> bool {
        self.default.is_none()
    }
}

pub fn formal_parameter_names(params: &[FormalParameter]) -> Vec<String> {
    params.iter().map(|param| param.name.clone()).collect()
}

pub fn is_simple_parameter_list(params: &[FormalParameter]) -> bool {
    params.iter().all(FormalParameter::is_simple)
}

pub fn formal_parameter_length(params: &[FormalParameter]) -> u32 {
    params
        .iter()
        .take_while(|param| param.default.is_none())
        .count() as u32
}

#[derive(Clone, Debug)]
pub enum AssignOp {
    Simple,
    Binary(BinaryOp),
}

#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Shl,
    Shr,
    UShr,
    Lt,
    Le,
    Gt,
    Ge,
    In,
    InstanceOf,
}

#[derive(Clone, Debug)]
pub enum LogicalOp {
    And,
    Or,
}

pub fn parse_script(source: &str) -> Completion<Vec<Stmt>> {
    let tokens = Lexer::new(source).tokenize()?;
    Parser { tokens, index: 0 }.script()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn script(&mut self) -> Completion<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.at_eof() {
            if self.consume_punct(';') {
                continue;
            }
            statements.push(self.statement()?);
            self.consume_punct(';');
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Completion<Stmt> {
        if self.consume_punct('{') {
            return self.block_statement();
        }
        if self.consume_ident("function") {
            return self.function_declaration();
        }
        if self.consume_ident("if") {
            return self.if_statement();
        }
        if self.consume_ident("while") {
            return self.while_statement();
        }
        if self.consume_ident("switch") {
            return self.switch_statement();
        }
        if self.consume_ident("with") {
            return self.with_statement();
        }
        if self.consume_ident("for") {
            return self.for_statement();
        }
        if self.consume_ident("try") {
            return self.try_catch_statement();
        }
        if self.consume_ident("break") {
            return Ok(Stmt::Break);
        }
        if self.consume_ident("continue") {
            return Ok(Stmt::Continue);
        }
        if self.consume_ident("return") {
            if self.peek_punct(';') || self.peek_punct('}') || self.at_eof() {
                return Ok(Stmt::Return(None));
            }
            return Ok(Stmt::Return(Some(self.expression()?)));
        }
        if self.consume_ident("throw") {
            return Ok(Stmt::Throw(self.expression()?));
        }
        if self.consume_ident("var") || self.consume_ident("let") || self.consume_ident("const") {
            return self.variable_statement();
        }
        Ok(Stmt::Expr(self.expression()?))
    }

    fn variable_statement(&mut self) -> Completion<Stmt> {
        let mut declarations = Vec::new();
        loop {
            let binding = self.binding_pattern()?;
            let expr = if self.consume_punct('=') {
                self.assignment()?
            } else {
                Expr::Undefined
            };
            declarations.push(Stmt::Var(binding, expr));
            if !self.consume_punct(',') {
                break;
            }
        }
        if declarations.len() == 1 {
            Ok(declarations.remove(0))
        } else {
            Ok(Stmt::Block(declarations))
        }
    }

    fn block_statement(&mut self) -> Completion<Stmt> {
        let mut statements = Vec::new();
        while !self.consume_punct('}') {
            if self.at_eof() {
                return Err(JsError::syntax("unterminated block"));
            }
            if self.consume_punct(';') {
                continue;
            }
            statements.push(self.statement()?);
            self.consume_punct(';');
        }
        Ok(Stmt::Block(statements))
    }

    fn function_declaration(&mut self) -> Completion<Stmt> {
        let name = self.expect_identifier()?;
        let (params, body) = self.function_tail("function body must be a block")?;
        Ok(Stmt::Function(name, params, body))
    }

    fn if_statement(&mut self) -> Completion<Stmt> {
        self.expect_punct('(')?;
        let test = self.expression()?;
        self.expect_punct(')')?;
        let then_branch = Box::new(self.statement()?);
        self.consume_punct(';');
        let else_branch = if self.consume_ident("else") {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If(test, then_branch, else_branch))
    }

    fn while_statement(&mut self) -> Completion<Stmt> {
        self.expect_punct('(')?;
        let test = self.expression()?;
        self.expect_punct(')')?;
        Ok(Stmt::While(test, Box::new(self.statement()?)))
    }

    fn switch_statement(&mut self) -> Completion<Stmt> {
        self.expect_punct('(')?;
        let discriminant = self.expression()?;
        self.expect_punct(')')?;
        self.expect_punct('{')?;

        let mut cases = Vec::new();
        while !self.consume_punct('}') {
            let test = if self.consume_ident("case") {
                let expr = self.expression()?;
                self.expect_punct(':')?;
                Some(expr)
            } else if self.consume_ident("default") {
                self.expect_punct(':')?;
                None
            } else if self.at_eof() {
                return Err(JsError::syntax("unterminated switch statement"));
            } else {
                return Err(JsError::syntax(format!(
                    "expected switch case, found {:?}",
                    self.peek()
                )));
            };

            let mut consequent = Vec::new();
            while !self.peek_punct('}')
                && !matches!(self.peek(), Token::Identifier(name) if name == "case" || name == "default")
            {
                if self.consume_punct(';') {
                    continue;
                }
                if self.at_eof() {
                    return Err(JsError::syntax("unterminated switch statement"));
                }
                consequent.push(self.statement()?);
                self.consume_punct(';');
            }
            cases.push(SwitchCase { test, consequent });
        }

        Ok(Stmt::Switch(discriminant, cases))
    }

    fn with_statement(&mut self) -> Completion<Stmt> {
        self.expect_punct('(')?;
        let object = self.expression()?;
        self.expect_punct(')')?;
        Ok(Stmt::With(object, Box::new(self.statement()?)))
    }

    fn for_statement(&mut self) -> Completion<Stmt> {
        self.expect_punct('(')?;
        let init = if self.consume_punct(';') {
            None
        } else if self.consume_ident("var")
            || self.consume_ident("let")
            || self.consume_ident("const")
        {
            let binding = self.binding_pattern()?;
            if self.consume_ident("in") {
                let name = binding_identifier_name(binding.clone())?;
                let object = self.expression()?;
                self.expect_punct(')')?;
                return Ok(Stmt::ForIn(name, object, Box::new(self.statement()?)));
            }
            if self.consume_ident("of") {
                let name = binding_identifier_name(binding.clone())?;
                let iterable = self.expression()?;
                self.expect_punct(')')?;
                return Ok(Stmt::ForOf(name, iterable, Box::new(self.statement()?)));
            }
            let expr = if self.consume_punct('=') {
                self.assignment()?
            } else {
                Expr::Undefined
            };
            self.expect_punct(';')?;
            Some(ForInit::Var(binding, expr))
        } else {
            let expr = self.expression()?;
            self.expect_punct(';')?;
            Some(ForInit::Expr(expr))
        };

        let test = if self.consume_punct(';') {
            None
        } else {
            let expr = self.expression()?;
            self.expect_punct(';')?;
            Some(expr)
        };

        let update = if self.consume_punct(')') {
            None
        } else {
            let expr = self.expression()?;
            self.expect_punct(')')?;
            Some(expr)
        };

        Ok(Stmt::For(init, test, update, Box::new(self.statement()?)))
    }

    fn try_catch_statement(&mut self) -> Completion<Stmt> {
        let Stmt::Block(try_body) = self.statement()? else {
            return Err(JsError::syntax("try body must be a block"));
        };
        let catch = if self.consume_ident("catch") {
            let binding = if self.consume_punct('(') {
                let name = self.expect_identifier()?;
                self.expect_punct(')')?;
                Some(name)
            } else {
                None
            };
            let Stmt::Block(catch_body) = self.statement()? else {
                return Err(JsError::syntax("catch body must be a block"));
            };
            Some((binding, catch_body))
        } else {
            None
        };
        let finally = if self.consume_ident("finally") {
            let Stmt::Block(finally_body) = self.statement()? else {
                return Err(JsError::syntax("finally body must be a block"));
            };
            Some(finally_body)
        } else {
            None
        };
        if catch.is_none() && finally.is_none() {
            return Err(JsError::syntax("try must be followed by catch or finally"));
        }
        Ok(Stmt::Try(try_body, catch, finally))
    }

    fn expression(&mut self) -> Completion<Expr> {
        self.comma()
    }

    fn comma(&mut self) -> Completion<Expr> {
        let first = self.assignment()?;
        if !self.consume_punct(',') {
            return Ok(first);
        }
        let mut exprs = vec![first, self.assignment()?];
        while self.consume_punct(',') {
            exprs.push(self.assignment()?);
        }
        Ok(Expr::Comma(exprs))
    }

    fn assignment(&mut self) -> Completion<Expr> {
        let left = self.conditional()?;
        if self.peek_punct('=') && self.peek_next_punct('>') {
            self.expect_punct('=')?;
            self.expect_punct('>')?;
            return self.arrow_function(left);
        }
        let op = if self.consume_punct('=') {
            Some(AssignOp::Simple)
        } else if self.consume_punct('+') {
            self.expect_punct('=')?;
            Some(AssignOp::Binary(BinaryOp::Add))
        } else if self.consume_punct('-') {
            self.expect_punct('=')?;
            Some(AssignOp::Binary(BinaryOp::Sub))
        } else if self.consume_punct('*') {
            self.expect_punct('=')?;
            Some(AssignOp::Binary(BinaryOp::Mul))
        } else if self.consume_punct('/') {
            self.expect_punct('=')?;
            Some(AssignOp::Binary(BinaryOp::Div))
        } else if self.consume_punct('%') {
            self.expect_punct('=')?;
            Some(AssignOp::Binary(BinaryOp::Mod))
        } else if self.peek_punct('<') && self.peek_next_punct('<') {
            self.expect_punct('<')?;
            self.expect_punct('<')?;
            self.expect_punct('=')?;
            Some(AssignOp::Binary(BinaryOp::Shl))
        } else if self.peek_punct('>') && self.peek_next_punct('>') {
            self.expect_punct('>')?;
            self.expect_punct('>')?;
            if self.consume_punct('>') {
                self.expect_punct('=')?;
                Some(AssignOp::Binary(BinaryOp::UShr))
            } else {
                self.expect_punct('=')?;
                Some(AssignOp::Binary(BinaryOp::Shr))
            }
        } else {
            None
        };
        if let Some(op) = op {
            return Ok(Expr::Assign(
                Box::new(left),
                op,
                Box::new(self.assignment()?),
            ));
        }
        Ok(left)
    }

    fn conditional(&mut self) -> Completion<Expr> {
        let test = self.logical_or()?;
        if self.consume_punct('?') {
            let consequent = self.assignment()?;
            self.expect_punct(':')?;
            let alternate = self.assignment()?;
            Ok(Expr::Conditional(
                Box::new(test),
                Box::new(consequent),
                Box::new(alternate),
            ))
        } else {
            Ok(test)
        }
    }

    fn logical_or(&mut self) -> Completion<Expr> {
        let mut expr = self.logical_and()?;
        while self.peek_punct('|') && self.peek_next_punct('|') {
            self.expect_punct('|')?;
            self.expect_punct('|')?;
            expr = Expr::Logical(Box::new(expr), LogicalOp::Or, Box::new(self.logical_and()?));
        }
        Ok(expr)
    }

    fn logical_and(&mut self) -> Completion<Expr> {
        let mut expr = self.equality()?;
        while self.peek_punct('&') && self.peek_next_punct('&') {
            self.expect_punct('&')?;
            self.expect_punct('&')?;
            expr = Expr::Logical(Box::new(expr), LogicalOp::And, Box::new(self.equality()?));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Completion<Expr> {
        let mut expr = self.relational()?;
        loop {
            if self.peek_punct('=') && self.peek_next_punct('=') {
                self.expect_punct('=')?;
                self.expect_punct('=')?;
                let strict = self.consume_punct('=');
                let right = self.relational()?;
                expr = Expr::Equal(Box::new(expr), Box::new(right), strict, false);
            } else if self.peek_punct('!') && self.peek_next_punct('=') {
                self.expect_punct('!')?;
                self.expect_punct('=')?;
                let strict = self.consume_punct('=');
                let right = self.relational()?;
                expr = Expr::Equal(Box::new(expr), Box::new(right), strict, true);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn relational(&mut self) -> Completion<Expr> {
        let mut expr = self.shift()?;
        loop {
            if self.peek_punct('<') && self.peek_next_punct('<') {
                break;
            } else if self.consume_punct('<') {
                let op = if self.consume_punct('=') {
                    BinaryOp::Le
                } else {
                    BinaryOp::Lt
                };
                expr = Expr::Binary(Box::new(expr), op, Box::new(self.shift()?));
            } else if self.peek_punct('>') && self.peek_next_punct('>') {
                break;
            } else if self.consume_punct('>') {
                let op = if self.consume_punct('=') {
                    BinaryOp::Ge
                } else {
                    BinaryOp::Gt
                };
                expr = Expr::Binary(Box::new(expr), op, Box::new(self.shift()?));
            } else if self.consume_ident("in") {
                expr = Expr::Binary(Box::new(expr), BinaryOp::In, Box::new(self.shift()?));
            } else if self.consume_ident("instanceof") {
                expr = Expr::Binary(
                    Box::new(expr),
                    BinaryOp::InstanceOf,
                    Box::new(self.shift()?),
                );
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn shift(&mut self) -> Completion<Expr> {
        let mut expr = self.additive()?;
        loop {
            if self.peek_punct('<') && self.peek_next_punct('<') {
                if self.peek_n_punct(2, '=') {
                    break;
                }
                self.expect_punct('<')?;
                self.expect_punct('<')?;
                expr = Expr::Binary(Box::new(expr), BinaryOp::Shl, Box::new(self.additive()?));
            } else if self.peek_punct('>') && self.peek_next_punct('>') {
                if self.peek_n_punct(2, '=') || self.peek_n_punct(3, '=') {
                    break;
                }
                self.expect_punct('>')?;
                self.expect_punct('>')?;
                let op = if self.consume_punct('>') {
                    BinaryOp::UShr
                } else {
                    BinaryOp::Shr
                };
                expr = Expr::Binary(Box::new(expr), op, Box::new(self.additive()?));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn additive(&mut self) -> Completion<Expr> {
        let mut expr = self.multiplicative()?;
        loop {
            if self.peek_punct('+') && self.peek_next_punct('=') {
                break;
            } else if self.consume_punct('+') {
                expr = Expr::Binary(
                    Box::new(expr),
                    BinaryOp::Add,
                    Box::new(self.multiplicative()?),
                );
            } else if self.peek_punct('-') && self.peek_next_punct('=') {
                break;
            } else if self.consume_punct('-') {
                expr = Expr::Binary(
                    Box::new(expr),
                    BinaryOp::Sub,
                    Box::new(self.multiplicative()?),
                );
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn multiplicative(&mut self) -> Completion<Expr> {
        let mut expr = self.exponentiation()?;
        loop {
            if self.peek_punct('*') && self.peek_next_punct('=') {
                break;
            } else if self.consume_punct('*') {
                expr = Expr::Binary(
                    Box::new(expr),
                    BinaryOp::Mul,
                    Box::new(self.exponentiation()?),
                );
            } else if self.peek_punct('/') && self.peek_next_punct('=') {
                break;
            } else if self.consume_punct('/') {
                expr = Expr::Binary(
                    Box::new(expr),
                    BinaryOp::Div,
                    Box::new(self.exponentiation()?),
                );
            } else if self.peek_punct('%') && self.peek_next_punct('=') {
                break;
            } else if self.consume_punct('%') {
                expr = Expr::Binary(
                    Box::new(expr),
                    BinaryOp::Mod,
                    Box::new(self.exponentiation()?),
                );
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn exponentiation(&mut self) -> Completion<Expr> {
        let left = self.unary()?;
        if self.peek_punct('*') && self.peek_next_punct('*') {
            self.expect_punct('*')?;
            self.expect_punct('*')?;
            let right = self.exponentiation()?;
            Ok(Expr::Binary(Box::new(left), BinaryOp::Pow, Box::new(right)))
        } else {
            Ok(left)
        }
    }

    fn unary(&mut self) -> Completion<Expr> {
        if self.consume_ident("typeof") {
            return Ok(Expr::TypeOf(Box::new(self.unary()?)));
        }
        if self.consume_ident("delete") {
            return Ok(Expr::Delete(Box::new(self.unary()?)));
        }
        if self.consume_ident("void") {
            return Ok(Expr::Void(Box::new(self.unary()?)));
        }
        if self.peek_punct('+') && self.peek_next_punct('+') {
            self.expect_punct('+')?;
            self.expect_punct('+')?;
            return Ok(Expr::Update(Box::new(self.unary()?), true, true));
        }
        if self.peek_punct('-') && self.peek_next_punct('-') {
            self.expect_punct('-')?;
            self.expect_punct('-')?;
            return Ok(Expr::Update(Box::new(self.unary()?), false, true));
        }
        if self.consume_punct('!') {
            return Ok(Expr::Not(Box::new(self.unary()?)));
        }
        if self.consume_punct('+') {
            return Ok(Expr::Pos(Box::new(self.unary()?)));
        }
        if self.consume_punct('-') {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Completion<Expr> {
        let mut expr = if self.consume_ident("new") {
            if self.consume_punct('.') {
                let name = self.expect_identifier()?;
                if name != "target" {
                    return Err(JsError::syntax("expected target after new."));
                }
                Expr::NewTarget
            } else {
                self.new_expression()?
            }
        } else {
            self.primary()?
        };
        loop {
            if self.consume_punct('.') {
                let name = self.expect_identifier()?;
                expr = Expr::Member(Box::new(expr), Box::new(Expr::String(name)));
            } else if self.consume_punct('[') {
                let key = self.expression()?;
                self.expect_punct(']')?;
                expr = Expr::Member(Box::new(expr), Box::new(key));
            } else if self.consume_punct('(') {
                let mut args = Vec::new();
                if !self.consume_punct(')') {
                    loop {
                        args.push(self.assignment()?);
                        if self.consume_punct(')') {
                            break;
                        }
                        self.expect_punct(',')?;
                        if self.consume_punct(')') {
                            break;
                        }
                    }
                }
                expr = Expr::Call(Box::new(expr), args);
            } else if self.peek_punct('+') && self.peek_next_punct('+') {
                self.expect_punct('+')?;
                self.expect_punct('+')?;
                expr = Expr::Update(Box::new(expr), true, false);
            } else if self.peek_punct('-') && self.peek_next_punct('-') {
                self.expect_punct('-')?;
                self.expect_punct('-')?;
                expr = Expr::Update(Box::new(expr), false, false);
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn new_expression(&mut self) -> Completion<Expr> {
        let callee = self.member_without_call()?;
        let args = if self.consume_punct('(') {
            self.argument_list_after_open_paren()?
        } else {
            Vec::new()
        };
        Ok(Expr::New(Box::new(callee), args))
    }

    fn member_without_call(&mut self) -> Completion<Expr> {
        let mut expr = self.primary()?;
        loop {
            if self.consume_punct('.') {
                let name = self.expect_identifier()?;
                expr = Expr::Member(Box::new(expr), Box::new(Expr::String(name)));
            } else if self.consume_punct('[') {
                let key = self.expression()?;
                self.expect_punct(']')?;
                expr = Expr::Member(Box::new(expr), Box::new(key));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn argument_list_after_open_paren(&mut self) -> Completion<Vec<Expr>> {
        let mut args = Vec::new();
        if self.consume_punct(')') {
            return Ok(args);
        }
        loop {
            args.push(self.assignment()?);
            if self.consume_punct(')') {
                break;
            }
            self.expect_punct(',')?;
            if self.consume_punct(')') {
                break;
            }
        }
        Ok(args)
    }

    fn primary(&mut self) -> Completion<Expr> {
        match self.bump().clone() {
            Token::Identifier(name) => match name.as_str() {
                "true" => Ok(Expr::Boolean(true)),
                "false" => Ok(Expr::Boolean(false)),
                "null" => Ok(Expr::Null),
                "undefined" => Ok(Expr::Undefined),
                "function" => self.function_expression(None),
                "this" => Ok(Expr::Identifier("this".to_owned())),
                _ => Ok(Expr::Identifier(name)),
            },
            Token::Number(value) => Ok(Expr::Number(value)),
            Token::BigInt(value) => Ok(Expr::BigInt(value)),
            Token::String(value) => Ok(Expr::String(value)),
            Token::RegExp(pattern, flags) => Ok(Expr::RegExp(pattern, flags)),
            Token::Punct('(') => {
                if self.consume_punct(')') {
                    if self.peek_punct('=') && self.peek_next_punct('>') {
                        return Ok(Expr::Comma(Vec::new()));
                    }
                    return Err(JsError::syntax("expected expression in parentheses"));
                }
                let expr = self.expression()?;
                self.expect_punct(')')?;
                Ok(expr)
            }
            Token::Punct('{') => self.object_literal(),
            Token::Punct('[') => self.array_literal(),
            token => Err(JsError::syntax(format!(
                "unexpected token in expression: {token:?}"
            ))),
        }
    }

    fn function_expression(&mut self, first_name: Option<String>) -> Completion<Expr> {
        let name = if first_name.is_some() {
            first_name
        } else if matches!(self.peek(), Token::Identifier(_)) && !self.peek_punct('(') {
            match self.bump().clone() {
                Token::Identifier(name) if self.peek_punct('(') => Some(name),
                Token::Identifier(name) => {
                    return Err(JsError::syntax(format!(
                        "unexpected token after function name {name}"
                    )))
                }
                _ => None,
            }
        } else {
            None
        };
        let (params, body) = self.function_tail("function expression body must be a block")?;
        Ok(Expr::Function(name, params, body))
    }

    fn function_tail(&mut self, body_error: &str) -> Completion<(Vec<FormalParameter>, Vec<Stmt>)> {
        self.expect_punct('(')?;
        let mut params = Vec::new();
        if !self.consume_punct(')') {
            loop {
                let name = self.expect_identifier()?;
                if is_reserved_formal_parameter_name(&name) {
                    return Err(JsError::syntax(format!(
                        "reserved word `{name}` cannot be used as a formal parameter"
                    )));
                }
                let default = if self.consume_punct('=') {
                    Some(self.assignment()?)
                } else {
                    None
                };
                params.push(FormalParameter { name, default });
                if self.consume_punct(')') {
                    break;
                }
                self.expect_punct(',')?;
                if self.consume_punct(')') {
                    break;
                }
            }
        }
        let Stmt::Block(body) = self.statement()? else {
            return Err(JsError::syntax(body_error));
        };
        Ok((params, body))
    }

    fn arrow_function(&mut self, head: Expr) -> Completion<Expr> {
        let params = arrow_parameters(head)?;
        let body = if self.peek_punct('{') {
            let Stmt::Block(body) = self.statement()? else {
                return Err(JsError::syntax("arrow function body must be a block"));
            };
            body
        } else {
            vec![Stmt::Return(Some(self.assignment()?))]
        };
        Ok(Expr::Arrow(params, body))
    }

    fn object_literal(&mut self) -> Completion<Expr> {
        let mut properties = Vec::new();
        if self.consume_punct('}') {
            return Ok(Expr::Object(properties));
        }
        loop {
            let property = match self.bump().clone() {
                Token::Identifier(name) => {
                    if (name == "get" || name == "set")
                        && !self.peek_punct('(')
                        && !self.peek_punct(':')
                    {
                        self.accessor_property(name == "get")?
                    } else if self.peek_punct('(') {
                        let (params, body) =
                            self.function_tail("object literal method body must be a block")?;
                        ObjectProperty {
                            key: Expr::String(name.clone()),
                            kind: ObjectPropertyKind::Data(Expr::Function(
                                Some(name),
                                params,
                                body,
                            )),
                        }
                    } else if self.consume_punct(':') {
                        ObjectProperty {
                            key: Expr::String(name),
                            kind: ObjectPropertyKind::Data(self.assignment()?),
                        }
                    } else {
                        ObjectProperty {
                            key: Expr::String(name.clone()),
                            kind: ObjectPropertyKind::Data(Expr::Identifier(name)),
                        }
                    }
                }
                Token::String(name) => {
                    if self.peek_punct('(') {
                        let (params, body) =
                            self.function_tail("object literal method body must be a block")?;
                        ObjectProperty {
                            key: Expr::String(name.clone()),
                            kind: ObjectPropertyKind::Data(Expr::Function(
                                Some(name),
                                params,
                                body,
                            )),
                        }
                    } else {
                        self.expect_punct(':')?;
                        ObjectProperty {
                            key: Expr::String(name),
                            kind: ObjectPropertyKind::Data(self.assignment()?),
                        }
                    }
                }
                Token::Number(value) => {
                    let key = number_key(value);
                    if self.peek_punct('(') {
                        let (params, body) =
                            self.function_tail("object literal method body must be a block")?;
                        ObjectProperty {
                            key: Expr::String(key.clone()),
                            kind: ObjectPropertyKind::Data(Expr::Function(Some(key), params, body)),
                        }
                    } else {
                        self.expect_punct(':')?;
                        ObjectProperty {
                            key: Expr::String(key),
                            kind: ObjectPropertyKind::Data(self.assignment()?),
                        }
                    }
                }
                Token::Punct('[') => {
                    let key = self.expression()?;
                    self.expect_punct(']')?;
                    if self.peek_punct('(') {
                        let (params, body) =
                            self.function_tail("object literal method body must be a block")?;
                        ObjectProperty {
                            key,
                            kind: ObjectPropertyKind::Data(Expr::Function(None, params, body)),
                        }
                    } else {
                        self.expect_punct(':')?;
                        ObjectProperty {
                            key,
                            kind: ObjectPropertyKind::Data(self.assignment()?),
                        }
                    }
                }
                Token::Punct('.') if self.consume_punct('.') && self.consume_punct('.') => {
                    ObjectProperty {
                        key: Expr::Undefined,
                        kind: ObjectPropertyKind::Spread(self.assignment()?),
                    }
                }
                token => {
                    return Err(JsError::syntax(format!(
                        "unexpected object literal key: {token:?}"
                    )))
                }
            };
            properties.push(property);
            if self.consume_punct('}') {
                break;
            }
            self.expect_punct(',')?;
            if self.consume_punct('}') {
                break;
            }
        }
        Ok(Expr::Object(properties))
    }

    fn accessor_property(&mut self, getter: bool) -> Completion<ObjectProperty> {
        let key = match self.bump().clone() {
            Token::Identifier(name) => Expr::String(name),
            Token::String(name) => Expr::String(name),
            Token::Number(value) => Expr::String(number_key(value)),
            Token::Punct('[') => {
                let key = self.expression()?;
                self.expect_punct(']')?;
                key
            }
            token => {
                return Err(JsError::syntax(format!(
                    "unexpected object literal accessor key: {token:?}"
                )))
            }
        };
        let (params, body) = self.function_tail("object literal accessor body must be a block")?;
        if getter && !params.is_empty() {
            return Err(JsError::syntax("getter must not declare parameters"));
        }
        if !getter && params.len() != 1 {
            return Err(JsError::syntax("setter must declare exactly one parameter"));
        }
        let value = Expr::Function(None, params, body);
        Ok(ObjectProperty {
            key,
            kind: if getter {
                ObjectPropertyKind::Get(value)
            } else {
                ObjectPropertyKind::Set(value)
            },
        })
    }

    fn array_literal(&mut self) -> Completion<Expr> {
        let mut elements = Vec::new();
        if self.consume_punct(']') {
            return Ok(Expr::Array(elements));
        }
        loop {
            if self.consume_punct(',') {
                elements.push(Expr::ArrayHole);
                if self.consume_punct(']') {
                    break;
                }
                continue;
            }
            elements.push(self.assignment()?);
            if self.consume_punct(']') {
                break;
            }
            self.expect_punct(',')?;
            if self.consume_punct(']') {
                break;
            }
        }
        Ok(Expr::Array(elements))
    }

    fn expect_identifier(&mut self) -> Completion<String> {
        match self.bump().clone() {
            Token::Identifier(name) => Ok(name),
            token => Err(JsError::syntax(format!(
                "expected identifier, found {token:?}"
            ))),
        }
    }

    fn binding_pattern(&mut self) -> Completion<BindingPattern> {
        match self.peek() {
            Token::Identifier(_) => Ok(BindingPattern::Identifier(self.expect_identifier()?)),
            Token::Punct('[') => self.array_binding_pattern(),
            Token::Punct('{') => self.object_binding_pattern(),
            token => Err(JsError::syntax(format!(
                "expected binding pattern, found {token:?}"
            ))),
        }
    }

    fn array_binding_pattern(&mut self) -> Completion<BindingPattern> {
        self.expect_punct('[')?;
        let mut names = Vec::new();
        if self.consume_punct(']') {
            return Ok(BindingPattern::Array(names));
        }
        loop {
            if self.consume_punct(',') {
                names.push(None);
                if self.consume_punct(']') {
                    break;
                }
                continue;
            }
            names.push(Some(self.expect_identifier()?));
            if self.consume_punct(']') {
                break;
            }
            self.expect_punct(',')?;
            if self.consume_punct(']') {
                break;
            }
        }
        Ok(BindingPattern::Array(names))
    }

    fn object_binding_pattern(&mut self) -> Completion<BindingPattern> {
        self.expect_punct('{')?;
        let mut names = Vec::new();
        if self.consume_punct('}') {
            return Ok(BindingPattern::Object(names));
        }
        loop {
            names.push(self.expect_identifier()?);
            if self.consume_punct('}') {
                break;
            }
            self.expect_punct(',')?;
            if self.consume_punct('}') {
                break;
            }
        }
        Ok(BindingPattern::Object(names))
    }

    fn consume_ident(&mut self, expected: &str) -> bool {
        if matches!(self.peek(), Token::Identifier(name) if name == expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn expect_punct(&mut self, expected: char) -> Completion<()> {
        if self.consume_punct(expected) {
            Ok(())
        } else {
            Err(JsError::syntax(format!("expected `{expected}`")))
        }
    }

    fn consume_punct(&mut self, expected: char) -> bool {
        if matches!(self.peek(), Token::Punct(ch) if *ch == expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn at_eof(&self) -> bool {
        matches!(self.peek(), Token::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.index).unwrap_or(&Token::Eof)
    }

    fn peek_punct(&self, expected: char) -> bool {
        matches!(self.peek(), Token::Punct(ch) if *ch == expected)
    }

    fn peek_next_punct(&self, expected: char) -> bool {
        matches!(self.tokens.get(self.index + 1).unwrap_or(&Token::Eof), Token::Punct(ch) if *ch == expected)
    }

    fn peek_n_punct(&self, offset: usize, expected: char) -> bool {
        matches!(self.tokens.get(self.index + offset).unwrap_or(&Token::Eof), Token::Punct(ch) if *ch == expected)
    }

    fn bump(&mut self) -> &Token {
        let token = self.tokens.get(self.index).unwrap_or(&Token::Eof);
        self.index += 1;
        token
    }
}

fn number_key(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn is_reserved_formal_parameter_name(name: &str) -> bool {
    matches!(
        name,
        "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "export"
            | "extends"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
            | "null"
            | "true"
            | "false"
    )
}

fn arrow_parameters(head: Expr) -> Completion<Vec<FormalParameter>> {
    let names = match head {
        Expr::Identifier(name) => vec![name],
        Expr::Comma(exprs) => {
            let mut names = Vec::new();
            for expr in exprs {
                let Expr::Identifier(name) = expr else {
                    return Err(JsError::syntax("arrow parameter must be an identifier"));
                };
                names.push(name);
            }
            names
        }
        _ => return Err(JsError::syntax("invalid arrow parameter list")),
    };
    let mut params = Vec::new();
    for name in names {
        if is_reserved_formal_parameter_name(&name) {
            return Err(JsError::syntax(format!(
                "reserved word `{name}` cannot be used as a formal parameter"
            )));
        }
        params.push(FormalParameter::simple(name));
    }
    Ok(params)
}

fn binding_identifier_name(binding: BindingPattern) -> Completion<String> {
    binding
        .into_identifier()
        .ok_or_else(|| JsError::syntax("for-in/of binding must be an identifier"))
}
