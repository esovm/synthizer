use super::scope::CowScope;
use super::CompileError;
use super::lexer::{Token, Symbol, Operator};
use super::function::Function;
use super::parser::{self, Parser, TokenStream};
use super::identifier::{IdMap, Identifier};
use super::expr::Expression;
use std::collections::VecMap;

/// Represents a function definition written in synthizer
#[derive(Debug)]
pub struct FunctionDef {
    statement: Option<Statement>,
    args: VecMap<Option<f32>>, // Default arguments
    ident: Option<Identifier>,
}

impl FunctionDef {
    pub fn new() -> FunctionDef {
        FunctionDef {
            statement: None,
            args: VecMap::new(),
            ident: None,
        }
    }

    pub fn ident(&self) -> Identifier {
        self.ident.expect("incomplete function definition! no identifier was attached")
    }

    pub fn set_ident(&mut self, ident: Identifier) {
        self.ident = Some(ident);
    }

    pub fn set_statement(&mut self, s: Statement) {
        self.statement = Some(s);
    }

    pub fn statement(&self) -> &Statement {
        self.statement.as_ref().expect("incomplete function definition! no statement was attached")
    }

    pub fn statement_mut(&mut self) -> &mut Statement {
        self.statement.as_mut().expect("incomplete function definition! no statement was attached")
    }

    pub fn args(&self) -> &VecMap<Option<f32>> {
        &self.args
    }

    pub fn args_mut(&mut self) -> &mut VecMap<Option<f32>> {
        &mut self.args
    }
}

impl<'a> Parser<'a> for FunctionDef {
    /// Parse a function definition from a token stream. Scope is used to find function definitions
    fn parse(tokens: TokenStream<'a>, scope: CowScope<'a>) -> Result<FunctionDef, CompileError> {
        let mut tokens = tokens;
        let mut def = FunctionDef::new();

        try!(expect!(tokens.next(), Token::Symbol(Symbol::LeftBracket), "expected `[`, got `{}`"));

        def.set_ident(try!(expect_value!(tokens.next(), Token::Ident, "expected function name, got `{}`")));

        loop {
            let mut args = def.args_mut();
            let next = tokens.next();
            if expect!(next, Token::Symbol(Symbol::RightBracket)).is_ok() {
                break;
            }
            let pos = next.map(|x| x.pos);
            let arg_ident = try!(expect_value!(next, Token::Ident, "expected argument name, got `{}`"));
            if args.contains_key(&arg_ident) {
                return Err(CompileError::new(format!("argument {} defined twice", arg_ident))
                           .with_pos(pos.unwrap()));
            }

            let next = tokens.next();
            let (pos, mut token) = (next.map(|x| x.pos), next.map(|x| x.token));
            if let Some(Token::Symbol(Symbol::Equals)) = token {
                let nexttoken = tokens.next();
                let value = if let Ok(_) = expect!(nexttoken, Token::Operator(Operator::Sub)) {
                    -try!(expect_value!(tokens.next(), Token::Const, "expected numerical constant, got `{}`"))
                } else {
                    try!(expect_value!(nexttoken, Token::Const, "expected numerical constant, got `{}`"))
                };
                args.insert(arg_ident, Some(value));
                token = tokens.next().map(|x| x.token);
            }
            match token {
                Some(Token::Symbol(Symbol::Comma)) => {
                    if !args.contains_key(&arg_ident) {
                        args.insert(arg_ident, None);
                    }
                }
                Some(Token::Symbol(Symbol::RightBracket)) => {
                    if !args.contains_key(&arg_ident) {
                        args.insert(arg_ident, None);
                    }
                    break;
                }
                Some(token) => return Err(CompileError::new(format!(
                            "expected `=`, `,` or `]`, got `{}`", token))
                            .with_pos(pos.unwrap())),
                None => return Err(CompileError::new_static("expected `=`, `,` or `]`, got EOF")
                                   .with_pos(tokens.end_source_pos())),
            }
        }

        def.set_statement(
            if !tokens.is_empty() {
                try!(Parser::parse(tokens, scope))
            } else {
                return Err(CompileError::new_static("expected block, got EOF")
                           .with_pos(tokens.end_source_pos()));
            });

        Ok(def)
    }
}
impl Function for FunctionDef {
    fn call(&self, _: CowScope, _: &IdMap) -> Result<f32, CompileError> {
        unimplemented!();
    }
}

#[derive(Debug)]
pub enum Statement {
    Block(Vec<(Option<Expression>, Operator, Statement)>),
    Expr(Expression),
}

impl<'a> Parser<'a> for Statement {
    fn parse(tokens: TokenStream<'a>, scope: CowScope<'a>) -> Result<Statement, CompileError> {
        let mut tokens = tokens;
        let mut statements = Vec::new();
        match expect!(tokens.peek(0), Token::Symbol(Symbol::LeftBrace)) {
            // If it starts with `{` it's a block
            Ok(_) => {
                tokens.next(); // consume brace
                loop {
                    let operator = match tokens.next().map(|x| x.token) {
                        Some(Token::Operator(op)) => op,
                        Some(Token::Symbol(Symbol::Semicolon)) => continue,
                        Some(Token::Symbol(Symbol::RightBrace)) => break,
                        Some(x) =>
                            return Err(CompileError::new(format!("expected operator, `;` or `}}`, got {}", x))
                                       .with_pos(tokens.peek(0).map(|x| x.pos).unwrap())),
                        None =>
                            return Err(CompileError::new_static("expected operator, ';' or `}`, got EOF")
                                       .with_pos(tokens.end_source_pos()))
                    };
                    let mut condition = None;
                    let start_pos = tokens.pos();
                    let mut pos;
                    if let Some(Token::Symbol(Symbol::LeftBrace)) = tokens.peek(0).map(|x| x.token) {
                        tokens = try!(parser::match_paren(tokens,
                                                          Token::Symbol(Symbol::LeftBrace),
                                                          Token::Symbol(Symbol::RightBrace)));
                    }
                    loop {
                        pos = tokens.pos();
                        match tokens.next().map(|x| x.token) {
                            Some(Token::Symbol(Symbol::Semicolon)) => break,
                            Some(Token::Symbol(Symbol::QuestionMark)) => {
                                // parse condition
                                let start_pos = pos;
                                let mut pos;
                                loop {
                                    pos = tokens.pos();
                                    match tokens.next().map(|x| x.token) {
                                        Some(Token::Symbol(Symbol::Semicolon)) => break,
                                        Some(_) => { },
                                        None => return Err(CompileError::new_static(
                                                "expected `;` to end condition, got EOF")
                                            .with_pos(tokens.end_source_pos())),
                                    }
                                }
                                condition = Some(try!(Parser::parse(
                                            tokens.slice(start_pos+1, pos),
                                            scope.clone())));
                                break;
                            }
                            Some(_) => { }
                            None =>
                                return Err(CompileError::new_static(
                                        "expected `;` or `?` to end expression, got EOF")
                                    .with_pos(tokens.end_source_pos())),
                        }
                    }
                    let statement = try!(Parser::parse(tokens.slice(start_pos, pos), scope.clone()));
                    statements.push((condition, operator, statement));
                }
            }

            // otherwise it's an expression
            Err(_) => {
                return Ok(Statement::Expr(try!(Parser::parse(tokens, scope))));
            }
        }
        Ok(Statement::Block(statements))
    }
}
impl Function for Statement {
    fn call(&self, scope: CowScope, idmap: &IdMap) -> Result<f32, CompileError> {
        match self {
            &Statement::Expr(ref x) =>
                x.call(scope, idmap),
            _ => unimplemented!(),
        }
    }
}
