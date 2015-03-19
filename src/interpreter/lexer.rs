use regex::Regex;
use super::issue::{Level, IssueTracker};
use super::identifier::{Identifier, IdMap};
use std::fmt;
use std::ops::Deref;

pub type Number = f32;

/// The various types that a token can be
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Token {
    Ident(Identifier),
    Const(Number),
    Operator(Operator),
    Symbol(Symbol),
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    Exp,
    Mod,
    Neg,
    Less,
    Greater,
    Equ,
    NotEqu,
    ApproxEqu,
    Not,
    And,
    Or,
    Xor,
    GreaterEqual,
    LessEqual,
}

#[derive(PartialEq)]
pub enum Associativity {
    Left,
    Right,
}

impl Operator {
    fn parse(s: &str) -> Option<Operator> {
        use self::Operator::*;
        Some(match s {
            "+" => Add,
            "-" => Sub,
            "*" => Mul,
            "/" => Div,
            "^" => Exp,
            "%" => Mod,
            "==" => Equ,
            "!=" => NotEqu,
            "~=" => ApproxEqu,
            "<" => Less,
            ">" => Greater,
            "<=" => LessEqual,
            ">=" => GreaterEqual,
            "!" => Not,
            "&&" => And,
            "||" => Or,
            "^^" => Xor,
            _ => return None,
        })
    }
    pub fn precedence(&self) -> i32 {
        use self::Operator::*;
        match *self {
            And | Or | Xor => 10,
            Equ | NotEqu | ApproxEqu => 20,
            Less | Greater | GreaterEqual | LessEqual => 30,
            Add | Sub => 40,
            Mul | Div | Mod => 50,
            Neg | Not | Exp => 60,
        }
    }

    pub fn num_args(&self) -> usize {
        use self::Operator::*;
        match *self {
            Neg | Not => 1,
            _ => 2,
        }
    }

    pub fn associativity(&self) -> Associativity {
        use self::Operator::*;
        match *self {
            Exp => Associativity::Right,
            _ => Associativity::Left,
        }
    }
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Symbol {
    Period,
    Comma,
    Equals,
    Colon,
    Semicolon,
    QuestionMark,
    Backslash,
    If,
    Else,
    LeftBracket(Bracket),
    RightBracket(Bracket),
}

impl Symbol {
    fn parse(s: &str) -> Option<Symbol> {
    use self::Symbol::*;
        Some(match s {
            "." => Period,
            "," => Comma,
            "=" => Equals,
            ":" => Colon,
            ";" => Semicolon,
            "?" => QuestionMark,
            "\\" => Backslash,
            "if" => If,
            "else" => Else,
            "(" => LeftBracket(Bracket::Round),
            ")" => RightBracket(Bracket::Round),
            "{" => LeftBracket(Bracket::Curly),
            "}" => RightBracket(Bracket::Curly),
            "[" => LeftBracket(Bracket::Square),
            "]" => RightBracket(Bracket::Square),
            _ => return None,
        })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Symbol::*;
        let string = match *self {
            Period => ".",
            Comma => ",",
            Equals => "=",
            Colon => ":",
            Semicolon => ";",
            QuestionMark => "?",
            Backslash => "\\",
            If => "if",
            Else => "else",
            LeftBracket(Bracket::Round) => "(",
            RightBracket(Bracket::Round) => ")",
            LeftBracket(Bracket::Curly) => "{",
            RightBracket(Bracket::Curly) => "}",
            LeftBracket(Bracket::Square) => "[",
            RightBracket(Bracket::Square) => "]",
        };
        write!(f, "{}", string)
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Token::*;
        match *self {
            Ident(x) => write!(f, "Id({})", x),
            Operator(x) => write!(f, "{:?}", x),
            Const(x) => write!(f, "{}", x),
            Symbol(x) => write!(f, "{}", x),
        }
    }
}

#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Bracket {
    Round,
    Square,
    Curly,
}

#[derive(Copy, Clone, Debug)]
pub struct SourcePos {
    pub line: usize,
    pub column: usize,
    pub index: usize,
    pub line_index: usize, //index of first character of line
}

impl SourcePos {
    pub fn new() -> SourcePos {
        SourcePos {
            line: 1,
            column: 1,
            index: 0,
            line_index: 0,
        }
    }
    /// Requires col to have reached the end of line for indices to be properly incremented.
    pub fn add_line(&mut self) {
        self.line += 1;
        self.column = 1;
        self.index += 1; // newline
        self.line_index = self.index;
    }

    pub fn add_chars(&mut self, num: usize) {
        self.column += num;
        self.index += num;
    }
}

impl fmt::Display for SourcePos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

///PW/PowWrapper holds an arbitrary object along with a position
#[derive(Copy, Clone, Debug)]
pub struct PW<T: Copy + Clone>(T, SourcePos);

pub trait ToPW<T> {
    fn pw(self, pos: SourcePos) -> PW<T>;
}
impl<T: Copy + Clone> ToPW<T> for T {
    fn pw(self, pos: SourcePos) -> PW<T> {
        PW(self, pos)
    }
}
pub trait PWImpl {
    type Token;
    type Pos;
    fn token(self) -> <Self as PWImpl>::Token;
    fn pos(self) -> <Self as PWImpl>::Pos;
}
impl<T: Copy + Clone> PWImpl for PW<T> {
    type Token = T;
    type Pos = SourcePos;
    fn token(self) -> T {
        self.0
    }
    fn pos(self) -> SourcePos {
        self.1
    }
}
impl<T: Copy + Clone> PWImpl for Option<PW<T>> {
    type Token = Option<T>;
    type Pos = Option<SourcePos>;
    fn token(self) -> Option<T> {
        self.map(|x| x.0)
    }
    fn pos(self) -> Option<SourcePos> {
        self.map(|x| x.1)
    }
}
impl<T: Copy + Clone> Deref for PW<T> {
    type Target = T;

    fn deref<'a>(&'a self) -> &'a T {
        &self.0
    }
}
impl<T: fmt::Debug + Copy + Clone> fmt::Display for PW<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}@{}", self.0, self.1)
    }
}

static IDENT_REGEX: Regex = regex!(r"[a-zA-Z_~']+[a-zA-Z_~0-9']*");
static WHITESPACE_REGEX: Regex = regex!(r"[ \t]+");
static CONST_REGEX: Regex = regex!(r"([0-9]+\.?[0-9]*|[0-9]*\.?[0-9]+)([eE]-?[0-9]+)?");
static OPERATOR_REGEX: Regex = regex!(r"\^\^|>=|<=|~=|[\+\*/\^><!%-]|&&|\|\||==|!=");
static SYMBOL_REGEX: Regex = regex!(r"if|else|[\.,=:;\?\(\)\{\}\]\[\\]");
static COMMENT_REGEX: Regex = regex!(r"//.*");
static NEWLINE_REGEX: Regex = regex!(r"[\n\r]");

pub fn lex<'a>(issues: &'a IssueTracker<'a>,
               string: &'a str,
               idmap: &'a IdMap<'a>) -> Option<Vec<PW<Token>>> {

    let mut walk = string;
    let mut tokens = Vec::new();
    let mut pos = SourcePos::new();

    while walk.len() > 0 {
        // Strip comments
        if let Some((0, x)) = COMMENT_REGEX.find(walk) {
            walk = &walk[x..];
            pos.add_chars(x);
            continue;
        }

        // Strip whitespace
        if let Some((0, x)) = WHITESPACE_REGEX.find(walk) {
            walk = &walk[x..];
            pos.add_chars(x);
            continue;
        }

        // Strip newlines
        if let Some((0, x)) = NEWLINE_REGEX.find(walk) {
            walk = &walk[x..];
            pos.add_line();
            continue;
        }

        // Add symbols
        if let Some((0, x)) = SYMBOL_REGEX.find(walk) {
            let sym = Symbol::parse(&walk[0..x]).unwrap(); // If this fails either the regex or the parser is wrong.
            tokens.push(Token::Symbol(sym).pw(pos));
            walk = &walk[x..];
            pos.add_chars(x);
            continue;
        }

        // Add identifiers
        if let Some((0, x)) = IDENT_REGEX.find(walk) {
            tokens.push(Token::Ident(idmap.id(&walk[0..x])).pw(pos));
            walk = &walk[x..];
            pos.add_chars(x);
            continue;
        }

        // Add operators
        if let Some((0, x)) = OPERATOR_REGEX.find(walk) {
            let op = Operator::parse(&walk[0..x]).unwrap(); // If this fails either the regex or the parser is wrong.
            tokens.push(Token::Operator(op).pw(pos));
            walk = &walk[x..];
            pos.add_chars(x);
            continue;
        }

        if let Some((0, x)) = CONST_REGEX.find(walk) {
            let v = walk[0..x].parse().unwrap(); // If this fails either the regex or the parser is wrong.
            tokens.push(Token::Const(v).pw(pos));
            walk = &walk[x..];
            pos.add_chars(x);
            continue;
        }

        // If none of the checks above found a token, then it's not supported.
        issues.new_issue(pos, Level::Error, "unrecognized token");
        walk = &walk[1..];
        pos.add_chars(1);
    }
    Some(tokens)
}
