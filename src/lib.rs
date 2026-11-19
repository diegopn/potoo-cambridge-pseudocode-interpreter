//! Interpreter for the shared core of the Cambridge 9618 pseudocode guides.

use std::collections::HashMap;
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edition {
    Cambridge2026,
    Cambridge2027To2029,
}

impl Edition {
    pub fn label(self) -> &'static str {
        match self {
            Self::Cambridge2026 => "2026",
            Self::Cambridge2027To2029 => "2027-2029",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}: {}",
            self.span.line, self.span.column, self.message
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CalendarDate {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl CalendarDate {
    fn new(day: u8, month: u8, year: u16) -> Option<Self> {
        if year == 0 {
            return None;
        }
        let days_in_month = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if year % 400 == 0 || year % 4 == 0 && year % 100 != 0 => 29,
            2 => 28,
            _ => return None,
        };
        (day > 0 && day <= days_in_month).then_some(Self { year, month, day })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Character(char),
    String(String),
    Boolean(bool),
    Date(Option<CalendarDate>),
    Enumeration {
        type_name: String,
        member: String,
        ordinal: usize,
        members: Vec<String>,
    },
    Array {
        bounds: Vec<(i64, i64)>,
        values: Vec<Value>,
    },
    Record {
        type_name: String,
        fields: Vec<(String, Value)>,
    },
    Object {
        class_name: String,
        fields: Vec<(String, Value)>,
    },
    Set {
        type_name: String,
        element_type: Box<Value>,
        values: Vec<Value>,
    },
    Pointer {
        type_name: String,
        target_type: String,
        address: Option<u64>,
    },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(n) => write!(f, "{n}"),
            Self::Real(n) => write!(f, "{n}"),
            Self::Character(c) => write!(f, "{c}"),
            Self::String(s) => f.write_str(s),
            Self::Boolean(b) => write!(f, "{}", if *b { "TRUE" } else { "FALSE" }),
            Self::Date(Some(date)) => {
                write!(f, "{:02}/{:02}/{:04}", date.day, date.month, date.year)
            }
            Self::Date(None) => Ok(()),
            Self::Enumeration { member, .. } => f.write_str(member),
            Self::Array { values, .. } => {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{value}")?;
                }
                Ok(())
            }
            Self::Record { fields, .. } => {
                for (i, (_, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{value}")?;
                }
                Ok(())
            }
            Self::Object { class_name, .. } => write!(f, "<{class_name} object>"),
            Self::Set { values, .. } => {
                for (i, value) in values.iter().enumerate() {
                    if i > 0 {
                        f.write_str(",")?;
                    }
                    write!(f, "{value}")?;
                }
                Ok(())
            }
            Self::Pointer { address, .. } => match address {
                Some(_) => f.write_str("<pointer>"),
                None => f.write_str("NULL"),
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct Program {
    pub edition: Edition,
    variables: HashMap<String, Value>,
    constants: HashMap<String, Value>,
    types: HashMap<String, Value>,
    routines: HashMap<String, Routine>,
    classes: HashMap<String, ClassDef>,
    body: Vec<Stmt>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Visibility {
    Public,
    Private,
}

#[derive(Clone, Debug)]
struct ClassField {
    name: String,
    owner: String,
    visibility: Visibility,
    value: Value,
}

#[derive(Clone, Debug)]
struct ClassMethod {
    routine: Routine,
    owner: String,
    visibility: Visibility,
}

#[derive(Clone, Debug)]
struct ClassDef {
    parent: Option<String>,
    fields: Vec<ClassField>,
    methods: HashMap<String, ClassMethod>,
}

#[derive(Clone, Debug)]
struct Routine {
    name: String,
    parameters: Vec<RoutineParameter>,
    return_type: Option<String>,
    body: Vec<Stmt>,
    line: usize,
}

#[derive(Clone, Debug)]
struct RoutineParameter {
    name: String,
    type_name: String,
    by_reference: bool,
}

#[derive(Clone, Debug)]
enum Stmt {
    Declare(String, Value),
    Constant(String, Value),
    Assign(Target, Expr),
    Input(Target),
    Output(Vec<Expr>),
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    Case(Expr, Vec<CaseArm>),
    For(String, Expr, Expr, Expr, Vec<Stmt>),
    While(Expr, Vec<Stmt>),
    Repeat(Vec<Stmt>, Expr),
    Call(String, Vec<Expr>),
    MethodCall(Expr),
    SuperCall(String, Vec<Expr>),
    Return(Option<Expr>),
    OpenFile(Expr, FileMode),
    ReadFile(Expr, Target),
    WriteFile(Expr, Expr),
    CloseFile(Expr),
    Seek(Expr, Expr),
    GetRecord(Expr, Target),
    PutRecord(Expr, Expr),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileMode {
    Read,
    Write,
    Append,
    Random,
}

#[derive(Clone, Debug)]
struct CaseArm {
    lower: Option<Expr>,
    upper: Option<Expr>,
    body: Vec<Stmt>,
}

#[derive(Clone, Debug, PartialEq)]
enum Target {
    Variable(String),
    Index(String, Vec<Expr>),
    Field(Box<Target>, String),
    NestedIndex(Box<Target>, Vec<Expr>),
    Dereference(Box<Expr>),
}

#[derive(Clone, Debug, PartialEq)]
enum Expr {
    Integer(i64),
    Real(f64),
    Character(char),
    String(String),
    Boolean(bool),
    Date(CalendarDate),
    Variable(String),
    Index(String, Vec<Expr>),
    Field(Box<Expr>, String),
    NestedIndex(Box<Expr>, Vec<Expr>),
    AddressOf(Box<Expr>),
    Dereference(Box<Expr>),
    Unary(Unary, Box<Expr>),
    Binary(Box<Expr>, Op, Box<Expr>),
    Call(String, Vec<Expr>),
    MethodCall(Box<Expr>, String, Vec<Expr>),
    New(String, Vec<Expr>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Unary {
    Negate,
    Not,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
    Mod,
    Concat,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
enum Tk {
    Integer(i64),
    Real(f64),
    Character(char),
    String(String),
    Date(CalendarDate),
    Name(String),
    LParen,
    RParen,
    LBracket,
    RBracket,
    Dot,
    Comma,
    Colon,
    Plus,
    Minus,
    Star,
    Caret,
    Slash,
    Ampersand,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Arrow,
    Eof,
}
#[derive(Clone, Debug)]
struct Token {
    kind: Tk,
    column: usize,
}

fn scan_date_literal(
    chars: &[char],
    start: usize,
    line: usize,
    column: usize,
) -> Result<Option<(CalendarDate, usize)>, Diagnostic> {
    if start + 10 > chars.len()
        || chars[start..start + 2].iter().any(|c| !c.is_ascii_digit())
        || chars[start + 2] != '/'
        || chars[start + 3..start + 5]
            .iter()
            .any(|c| !c.is_ascii_digit())
        || chars[start + 5] != '/'
        || chars[start + 6..start + 10]
            .iter()
            .any(|c| !c.is_ascii_digit())
        || chars.get(start + 10).is_some_and(char::is_ascii_digit)
    {
        return Ok(None);
    }
    let number = |range: std::ops::Range<usize>| -> u16 {
        chars[range]
            .iter()
            .collect::<String>()
            .parse()
            .expect("fixed-width date digits are numeric")
    };
    let day = number(start..start + 2) as u8;
    let month = number(start + 3..start + 5) as u8;
    let year = number(start + 6..start + 10);
    CalendarDate::new(day, month, year)
        .map(|date| Some((date, start + 10)))
        .ok_or_else(|| {
            diag(
                line,
                column,
                "invalid calendar date; expected a valid dd/mm/yyyy value",
            )
        })
}

fn parse_date_text(source: &str) -> Option<CalendarDate> {
    let chars = source.chars().collect::<Vec<_>>();
    let (date, end) = scan_date_literal(&chars, 0, 0, 1).ok().flatten()?;
    (end == chars.len()).then_some(date)
}

fn lex(source: &str, line: usize) -> Result<Vec<Token>, Diagnostic> {
    let chars: Vec<char> = source.chars().collect();
    let mut at = 0;
    let mut tokens = Vec::new();
    while at < chars.len() {
        let c = chars[at];
        let column = at + 1;
        if c.is_whitespace() {
            at += 1;
            continue;
        }
        if c == '"' || c == '\'' || c == 'ꞌ' {
            let quote = c;
            at += 1;
            let mut value = String::new();
            let mut closed = false;
            while at < chars.len() {
                if chars[at] == quote {
                    if chars.get(at + 1) == Some(&quote) {
                        value.push(quote);
                        at += 2;
                        continue;
                    }
                    at += 1;
                    closed = true;
                    break;
                }
                if quote == '"' && chars[at] == '\\' {
                    at += 1;
                    let Some(next) = chars.get(at).copied() else {
                        break;
                    };
                    value.push(match next {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        other => other,
                    });
                    at += 1;
                } else {
                    value.push(chars[at]);
                    at += 1;
                }
            }
            if !closed {
                return Err(diag(line, column, "unterminated literal"));
            }
            if quote == '\'' {
                let mut chars = value.chars();
                let Some(character) = chars.next() else {
                    return Err(diag(line, column, "empty character literal"));
                };
                if chars.next().is_some() {
                    return Err(diag(
                        line,
                        column,
                        "character literal must contain one character",
                    ));
                }
                tokens.push(Token {
                    kind: Tk::Character(character),
                    column,
                });
            } else {
                tokens.push(Token {
                    kind: Tk::String(value),
                    column,
                });
            }
            continue;
        }
        if c.is_ascii_digit() {
            if let Some((date, end)) = scan_date_literal(&chars, at, line, column)? {
                tokens.push(Token {
                    kind: Tk::Date(date),
                    column,
                });
                at = end;
                continue;
            }
            let start = at;
            at += 1;
            let mut decimal_points = 0;
            while at < chars.len() && (chars[at].is_ascii_digit() || chars[at] == '.') {
                if chars[at] == '.' {
                    decimal_points += 1;
                }
                at += 1;
            }
            let raw: String = chars[start..at].iter().collect();
            if decimal_points > 1 || raw.starts_with('.') || raw.ends_with('.') {
                return Err(diag(
                    line,
                    column,
                    "real literals need digits on both sides of the decimal point",
                ));
            }
            let kind = if raw.contains('.') {
                Tk::Real(
                    raw.parse()
                        .map_err(|_| diag(line, column, "invalid real literal"))?,
                )
            } else {
                Tk::Integer(
                    raw.parse()
                        .map_err(|_| diag(line, column, "integer literal out of range"))?,
                )
            };
            tokens.push(Token { kind, column });
            continue;
        }
        if c.is_alphanumeric() || c == '_' {
            let start = at;
            at += 1;
            while at < chars.len() && (chars[at].is_alphanumeric() || chars[at] == '_') {
                at += 1;
            }
            tokens.push(Token {
                kind: Tk::Name(chars[start..at].iter().collect()),
                column,
            });
            continue;
        }
        let kind = match c {
            '(' => Some(Tk::LParen),
            ')' => Some(Tk::RParen),
            '[' => Some(Tk::LBracket),
            ']' => Some(Tk::RBracket),
            '.' => Some(Tk::Dot),
            ',' => Some(Tk::Comma),
            ':' => Some(Tk::Colon),
            '+' => Some(Tk::Plus),
            '-' => Some(Tk::Minus),
            '*' => Some(Tk::Star),
            '^' => Some(Tk::Caret),
            '/' => Some(Tk::Slash),
            '&' => Some(Tk::Ampersand),
            '=' => Some(Tk::Eq),
            '<' => Some(Tk::Lt),
            '>' => Some(Tk::Gt),
            '←' => Some(Tk::Arrow),
            _ => None,
        };
        if let Some(mut kind) = kind {
            at += 1;
            let next = chars.get(at).copied();
            let pair = match (c, next) {
                ('<', Some('=')) => Some(Tk::Le),
                ('>', Some('=')) => Some(Tk::Ge),
                ('<', Some('>')) => Some(Tk::Ne),
                ('=', Some('=')) => Some(Tk::Eq),
                ('<', Some('-')) => Some(Tk::Arrow),
                _ => None,
            };
            if let Some(paired) = pair {
                kind = paired;
                at += 1;
            }
            tokens.push(Token { kind, column });
            continue;
        }
        return Err(diag(line, column, format!("unsupported character `{c}`")));
    }
    tokens.push(Token {
        kind: Tk::Eof,
        column: source.chars().count() + 1,
    });
    Ok(tokens)
}

struct ExprParser {
    tokens: Vec<Token>,
    at: usize,
    line: usize,
}
impl ExprParser {
    fn parse(source: &str, line: usize) -> Result<Expr, Diagnostic> {
        let mut parser = Self {
            tokens: lex(source, line)?,
            at: 0,
            line,
        };
        let expression = parser.expression(0)?;
        if parser.current().kind != Tk::Eof {
            return Err(parser.error("unexpected text after expression"));
        }
        Ok(expression)
    }
    fn expression(&mut self, min: u8) -> Result<Expr, Diagnostic> {
        let mut left = self.prefix()?;
        loop {
            if self.current().kind == Tk::LBracket {
                self.at += 1;
                let mut indices = Vec::new();
                loop {
                    indices.push(self.expression(0)?);
                    if self.take(&Tk::Comma) {
                        continue;
                    }
                    break;
                }
                self.expect(Tk::RBracket, "expected `]`")?;
                left = match left {
                    Expr::Variable(name) => Expr::Index(name, indices),
                    base => Expr::NestedIndex(Box::new(base), indices),
                };
                continue;
            }
            if self.take(&Tk::Dot) {
                let token = self.current().clone();
                let Tk::Name(field) = token.kind else {
                    return Err(self.error("expected a field or method name after `.`"));
                };
                self.at += 1;
                let name = normalize(&field);
                if self.take(&Tk::LParen) {
                    left = Expr::MethodCall(Box::new(left), name, self.argument_list()?);
                } else {
                    left = Expr::Field(Box::new(left), name);
                }
                continue;
            }
            if self.take(&Tk::Caret) {
                left = Expr::Dereference(Box::new(left));
                continue;
            }
            let Some((op, precedence)) = self.operator() else {
                break;
            };
            if precedence < min {
                break;
            }
            self.at += 1;
            let right = self.expression(precedence + 1)?;
            left = Expr::Binary(Box::new(left), op, Box::new(right));
        }
        Ok(left)
    }
    fn prefix(&mut self) -> Result<Expr, Diagnostic> {
        let token = self.current().clone();
        match token.kind {
            Tk::Integer(n) => {
                self.at += 1;
                Ok(Expr::Integer(n))
            }
            Tk::Real(n) => {
                self.at += 1;
                Ok(Expr::Real(n))
            }
            Tk::Character(c) => {
                self.at += 1;
                Ok(Expr::Character(c))
            }
            Tk::String(s) => {
                self.at += 1;
                Ok(Expr::String(s))
            }
            Tk::Date(date) => {
                self.at += 1;
                Ok(Expr::Date(date))
            }
            Tk::Caret => {
                self.at += 1;
                Ok(Expr::AddressOf(Box::new(self.expression(7)?)))
            }
            Tk::Name(name) if eq(&name, "TRUE") => {
                self.at += 1;
                Ok(Expr::Boolean(true))
            }
            Tk::Name(name) if eq(&name, "FALSE") => {
                self.at += 1;
                Ok(Expr::Boolean(false))
            }
            Tk::Name(name) if eq(&name, "NEW") => {
                self.at += 1;
                let class = self.current().clone();
                let Tk::Name(class_name) = class.kind else {
                    return Err(self.error("NEW requires a class name"));
                };
                self.at += 1;
                self.expect(Tk::LParen, "NEW requires parentheses")?;
                Ok(Expr::New(normalize(&class_name), self.argument_list()?))
            }
            Tk::Name(name) if eq(&name, "NOT") => {
                self.at += 1;
                Ok(Expr::Unary(Unary::Not, Box::new(self.expression(7)?)))
            }
            Tk::Name(name) if eq(&name, "DIV") || eq(&name, "MOD") => {
                Err(self.error("binary operator missing left operand"))
            }
            Tk::Name(name) => {
                self.at += 1;
                if self.take(&Tk::LParen) {
                    let mut args = Vec::new();
                    if self.current().kind != Tk::RParen {
                        loop {
                            args.push(self.expression(0)?);
                            if self.take(&Tk::Comma) {
                                continue;
                            }
                            break;
                        }
                    }
                    self.expect(Tk::RParen, "expected `)`")?;
                    Ok(Expr::Call(name, args))
                } else {
                    Ok(Expr::Variable(name))
                }
            }
            Tk::Minus => {
                self.at += 1;
                Ok(Expr::Unary(Unary::Negate, Box::new(self.expression(7)?)))
            }
            Tk::LParen => {
                self.at += 1;
                let expr = self.expression(0)?;
                self.expect(Tk::RParen, "expected `)`")?;
                Ok(expr)
            }
            _ => Err(Diagnostic {
                span: Span {
                    line: self.line,
                    column: token.column,
                },
                message: "expected expression".into(),
            }),
        }
    }
    fn operator(&self) -> Option<(Op, u8)> {
        let op = match &self.current().kind {
            Tk::Name(s) if eq(s, "OR") => (Op::Or, 1),
            Tk::Name(s) if eq(s, "AND") => (Op::And, 2),
            Tk::Name(s) if eq(s, "DIV") => (Op::IntDiv, 5),
            Tk::Name(s) if eq(s, "MOD") => (Op::Mod, 5),
            Tk::Eq => (Op::Eq, 3),
            Tk::Ne => (Op::Ne, 3),
            Tk::Lt => (Op::Lt, 3),
            Tk::Le => (Op::Le, 3),
            Tk::Gt => (Op::Gt, 3),
            Tk::Ge => (Op::Ge, 3),
            Tk::Plus => (Op::Add, 4),
            Tk::Minus => (Op::Sub, 4),
            Tk::Ampersand => (Op::Concat, 4),
            Tk::Star => (Op::Mul, 5),
            Tk::Slash => (Op::Div, 5),
            _ => return None,
        };
        Some(op)
    }
    fn argument_list(&mut self) -> Result<Vec<Expr>, Diagnostic> {
        let mut args = Vec::new();
        if self.current().kind != Tk::RParen {
            loop {
                args.push(self.expression(0)?);
                if self.take(&Tk::Comma) {
                    continue;
                }
                break;
            }
        }
        self.expect(Tk::RParen, "expected `)`")?;
        Ok(args)
    }
    fn current(&self) -> &Token {
        &self.tokens[self.at.min(self.tokens.len() - 1)]
    }
    fn take(&mut self, expected: &Tk) -> bool {
        if std::mem::discriminant(&self.current().kind) == std::mem::discriminant(expected) {
            self.at += 1;
            true
        } else {
            false
        }
    }
    fn expect(&mut self, expected: Tk, message: &str) -> Result<(), Diagnostic> {
        if self.take(&expected) {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }
    fn error(&self, message: &str) -> Diagnostic {
        diag(self.line, self.current().column, message)
    }
}

fn eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}
fn diag(line: usize, column: usize, message: impl Into<String>) -> Diagnostic {
    Diagnostic {
        span: Span { line, column },
        message: message.into(),
    }
}

struct Parser {
    lines: Vec<(usize, String)>,
    at: usize,
    routine_body_depth: usize,
    variables: HashMap<String, Value>,
    constants: HashMap<String, Value>,
    types: HashMap<String, Value>,
    routines: HashMap<String, Routine>,
    classes: HashMap<String, ClassDef>,
}

pub fn parse(source: &str, edition: Edition) -> Result<Program, Vec<Diagnostic>> {
    let lines = source
        .lines()
        .enumerate()
        .filter_map(|(i, raw)| {
            let line = strip_comment(raw).trim().to_owned();
            (!line.is_empty()).then_some((i + 1, line))
        })
        .collect::<Vec<_>>();
    let mut parser = Parser {
        lines,
        at: 0,
        routine_body_depth: 0,
        variables: HashMap::new(),
        constants: HashMap::new(),
        types: HashMap::new(),
        routines: HashMap::new(),
        classes: HashMap::new(),
    };
    let mut body = Vec::new();
    while parser.at < parser.lines.len() {
        let (line, text) = parser.lines[parser.at].clone();
        let upper = text.to_ascii_uppercase();
        if upper.starts_with("DECLARE ") {
            parser.at += 1;
            parser.declare(&text[8..], line).map_err(|e| vec![e])?;
        } else if upper.starts_with("TYPE ") {
            parser.type_definition(line, &text).map_err(|e| vec![e])?;
        } else if upper.starts_with("CLASS ") {
            parser.class_definition(line, &text).map_err(|e| vec![e])?;
        } else if upper.starts_with("DEFINE ") {
            parser.define(&text[7..], line).map_err(|e| vec![e])?;
        } else if upper.starts_with("CONSTANT ") {
            parser.at += 1;
            parser.constant(&text[9..], line).map_err(|e| vec![e])?;
        } else if upper.starts_with("PROCEDURE ") || upper.starts_with("FUNCTION ") {
            let routine = parser.routine(line, &text).map_err(|e| vec![e])?;
            let key = normalize(&routine.name);
            if parser.routines.insert(key.clone(), routine).is_some() {
                return Err(vec![diag(
                    line,
                    1,
                    format!("routine `{key}` is defined more than once"),
                )]);
            }
        } else {
            parser.at += 1;
            body.push(parser.statement(line, &text).map_err(|e| vec![e])?);
        }
    }
    Ok(Program {
        edition,
        variables: parser.variables,
        constants: parser.constants,
        types: parser.types,
        routines: parser.routines,
        classes: parser.classes,
        body,
    })
}

pub fn check(program: &Program) -> Result<(), Vec<Diagnostic>> {
    let mut routines = program.routines.values().collect::<Vec<_>>();
    routines.extend(
        program
            .classes
            .values()
            .flat_map(|class| class.methods.values().map(|method| &method.routine)),
    );
    for routine in routines {
        let mut declared = Vec::new();
        for parameter in &routine.parameters {
            let name = normalize(&parameter.name);
            if declared.contains(&name) {
                return Err(vec![diag(
                    routine.line,
                    1,
                    format!("identifier `{}` is declared more than once", parameter.name),
                )]);
            }
            declared.push(name);
        }
        for (name, _) in declared_local_initializers(&routine.body) {
            if declared.contains(&name) {
                return Err(vec![diag(
                    routine.line,
                    1,
                    format!(
                        "identifier `{name}` is declared more than once in routine `{}`",
                        routine.name
                    ),
                )]);
            }
            declared.push(name);
        }
        for (name, _) in declared_local_constants(&routine.body) {
            if declared.contains(&name) {
                return Err(vec![diag(
                    routine.line,
                    1,
                    format!(
                        "identifier `{name}` is declared more than once in routine `{}`",
                        routine.name
                    ),
                )]);
            }
            declared.push(name);
        }
        if routine.return_type.is_some() && !contains_return(&routine.body) {
            return Err(vec![diag(
                routine.line,
                1,
                format!("function `{}` has no RETURN statement", routine.name),
            )]);
        }
    }
    Ok(())
}

fn contains_return(body: &[Stmt]) -> bool {
    body.iter().any(|stmt| match stmt {
        Stmt::Return(_) => true,
        Stmt::If(_, a, b) => contains_return(a) || contains_return(b),
        Stmt::Case(_, arms) => arms.iter().any(|arm| contains_return(&arm.body)),
        Stmt::For(_, _, _, _, b) | Stmt::While(_, b) | Stmt::Repeat(b, _) => contains_return(b),
        _ => false,
    })
}

fn strip_comment(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut quote = None;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'"' || bytes[i] == b'\'' {
            if quote == Some(bytes[i]) && bytes.get(i + 1) == Some(&bytes[i]) {
                i += 2;
                continue;
            }
            quote = if quote == Some(bytes[i]) {
                None
            } else if quote.is_none() {
                Some(bytes[i])
            } else {
                quote
            };
        } else if quote.is_none() && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            return source[..i].to_owned();
        }
        i += 1;
    }
    source.to_owned()
}

fn normalize(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn is_terminator(source: &str, keyword: &str) -> bool {
    if eq(source, keyword) {
        return true;
    }
    if !matches!(keyword, "NEXT" | "UNTIL") {
        return false;
    }
    source
        .to_ascii_uppercase()
        .strip_prefix(keyword)
        .and_then(|rest| rest.chars().next())
        .is_some_and(|character| character.is_whitespace())
}

impl Parser {
    fn declare(&mut self, source: &str, line: usize) -> Result<(), Diagnostic> {
        let (name, value) = parse_declaration(source, line, &self.types)?;
        let key = name.clone();
        if self.variables.contains_key(&key) || self.constants.contains_key(&key) {
            return Err(diag(line, 9, format!("`{name}` is already declared")));
        }
        self.variables.insert(key, value);
        Ok(())
    }

    fn type_definition(&mut self, line: usize, header: &str) -> Result<(), Diagnostic> {
        let definition = header[5..].trim();
        let (type_name, assigned_type) = match definition.split_once('=') {
            Some((name, values)) => (name.trim(), Some(values.trim())),
            None => (definition, None),
        };
        if !valid_identifier(type_name) {
            return Err(diag(line, 6, "invalid type name"));
        }
        let key = normalize(type_name);
        if self.types.contains_key(&key) {
            return Err(diag(
                line,
                6,
                format!("type `{type_name}` is already defined"),
            ));
        }
        if let Some(assigned_type) = assigned_type {
            let upper_assigned_type = assigned_type.to_ascii_uppercase();
            if let Some(pointed_type_name) = assigned_type.strip_prefix('^') {
                let pointed_type_name = pointed_type_name.trim();
                let pointed_type =
                    default_value(pointed_type_name, &self.types).ok_or_else(|| {
                        diag(
                            line,
                            6,
                            format!("unsupported pointer target type `{pointed_type_name}`"),
                        )
                    })?;
                self.types.insert(
                    key,
                    Value::Pointer {
                        type_name: normalize(type_name),
                        target_type: value_type_signature(&pointed_type),
                        address: None,
                    },
                );
                self.at += 1;
                return Ok(());
            }
            if upper_assigned_type.starts_with("SET OF ") {
                let element_type_name = assigned_type[7..].trim();
                let element_type =
                    default_value(element_type_name, &self.types).ok_or_else(|| {
                        diag(
                            line,
                            6,
                            format!("unsupported set element type `{element_type_name}`"),
                        )
                    })?;
                self.types.insert(
                    key,
                    Value::Set {
                        type_name: normalize(type_name),
                        element_type: Box::new(element_type),
                        values: Vec::new(),
                    },
                );
                self.at += 1;
                return Ok(());
            }
            let values = assigned_type
                .strip_prefix('(')
                .and_then(|values| values.strip_suffix(')'))
                .ok_or_else(|| diag(line, 6, "enumerated TYPE requires `(value1, value2, ...)`"))?;
            let mut members = Vec::new();
            for value in split_commas(values) {
                let member = value.trim();
                if !valid_identifier(member) {
                    return Err(diag(
                        line,
                        6,
                        format!("invalid enumerated value `{member}`"),
                    ));
                }
                let member_key = normalize(member);
                if members
                    .iter()
                    .any(|(existing, _): &(String, String)| existing == &member_key)
                {
                    return Err(diag(
                        line,
                        6,
                        format!("enumerated value `{member}` is repeated"),
                    ));
                }
                if self.variables.contains_key(&member_key)
                    || self.constants.contains_key(&member_key)
                {
                    return Err(diag(
                        line,
                        6,
                        format!("enumerated value `{member}` is already declared"),
                    ));
                }
                members.push((member_key, member.to_owned()));
            }
            let Some((_, first)) = members.first() else {
                return Err(diag(
                    line,
                    6,
                    "enumerated TYPE must include at least one value",
                ));
            };
            let type_name = normalize(type_name);
            let enum_members = members
                .iter()
                .map(|(_, member)| member.clone())
                .collect::<Vec<_>>();
            self.types.insert(
                key,
                Value::Enumeration {
                    type_name: type_name.clone(),
                    member: first.clone(),
                    ordinal: 0,
                    members: enum_members.clone(),
                },
            );
            for (ordinal, (member_key, member)) in members.into_iter().enumerate() {
                self.constants.insert(
                    member_key,
                    Value::Enumeration {
                        type_name: type_name.clone(),
                        member,
                        ordinal,
                        members: enum_members.clone(),
                    },
                );
            }
            self.at += 1;
            return Ok(());
        }

        self.at += 1;
        let mut fields = Vec::new();
        while self.at < self.lines.len() {
            let (field_line, source) = self.lines[self.at].clone();
            if eq(&source, "ENDTYPE") {
                self.at += 1;
                if fields.is_empty() {
                    return Err(diag(line, 1, "record type must declare at least one field"));
                }
                self.types.insert(
                    key,
                    Value::Record {
                        type_name: normalize(type_name),
                        fields,
                    },
                );
                return Ok(());
            }
            let upper = source.to_ascii_uppercase();
            if !upper.starts_with("DECLARE ") {
                return Err(diag(
                    field_line,
                    1,
                    "record fields must use `DECLARE name : type`",
                ));
            }
            let (field_name, value) = parse_declaration(&source[8..], field_line, &self.types)?;
            if fields.iter().any(|(name, _)| name == &field_name) {
                return Err(diag(
                    field_line,
                    9,
                    format!("record field `{field_name}` is already declared"),
                ));
            }
            fields.push((field_name, value));
            self.at += 1;
        }
        Err(diag(line, 1, "record type is missing ENDTYPE"))
    }

    fn class_definition(&mut self, line: usize, header: &str) -> Result<(), Diagnostic> {
        let declaration = header[6..].trim();
        let upper = declaration.to_ascii_uppercase();
        let inherits_at = upper.find(" INHERITS ");
        let (name, parent) = if let Some(position) = inherits_at {
            (
                declaration[..position].trim(),
                Some(normalize(declaration[position + 10..].trim())),
            )
        } else {
            (declaration, None)
        };
        if !valid_identifier(name) {
            return Err(diag(line, 7, "invalid class name"));
        }
        let class_name = normalize(name);
        if self.classes.contains_key(&class_name) || self.types.contains_key(&class_name) {
            return Err(diag(line, 7, format!("type `{name}` is already defined")));
        }
        let mut fields = if let Some(parent_name) = &parent {
            self.classes
                .get(parent_name)
                .map(|class| class.fields.clone())
                .ok_or_else(|| diag(line, 7, format!("unknown parent class `{parent_name}`")))?
        } else {
            Vec::new()
        };
        let mut methods = HashMap::new();
        self.at += 1;
        while self.at < self.lines.len() {
            let (member_line, source) = self.lines[self.at].clone();
            if eq(&source, "ENDCLASS") {
                self.at += 1;
                let object = Value::Object {
                    class_name: class_name.clone(),
                    fields: fields
                        .iter()
                        .map(|field| (field.name.clone(), field.value.clone()))
                        .collect(),
                };
                self.types.insert(class_name.clone(), object);
                self.classes.insert(
                    class_name.clone(),
                    ClassDef {
                        parent,
                        fields,
                        methods,
                    },
                );
                return Ok(());
            }

            let (visibility, member) = if source.to_ascii_uppercase().starts_with("PRIVATE ") {
                (Visibility::Private, source[8..].trim())
            } else if source.to_ascii_uppercase().starts_with("PUBLIC ") {
                (Visibility::Public, source[7..].trim())
            } else {
                (Visibility::Public, source.as_str())
            };
            let upper_member = member.to_ascii_uppercase();
            if upper_member.starts_with("PROCEDURE ") || upper_member.starts_with("FUNCTION ") {
                let routine = self.routine(member_line, member)?;
                let key = normalize(&routine.name);
                if methods.contains_key(&key) {
                    return Err(diag(
                        member_line,
                        1,
                        format!("method `{}` is already defined in `{name}`", routine.name),
                    ));
                }
                methods.insert(
                    key,
                    ClassMethod {
                        routine,
                        owner: class_name.clone(),
                        visibility,
                    },
                );
                continue;
            }

            self.at += 1;
            let field_source = member.strip_prefix("DECLARE ").unwrap_or(member).trim();
            let (field_name, value) = parse_declaration(field_source, member_line, &self.types)?;
            if fields.iter().any(|field| field.name == field_name) {
                return Err(diag(
                    member_line,
                    1,
                    format!("field `{field_name}` is already declared in `{name}` or its parent"),
                ));
            }
            fields.push(ClassField {
                name: field_name,
                owner: class_name.clone(),
                visibility,
                value,
            });
        }
        Err(diag(line, 1, "class is missing ENDCLASS"))
    }

    fn define(&mut self, source: &str, line: usize) -> Result<(), Diagnostic> {
        let Some((name_and_values, type_name)) = source.split_once(':') else {
            return Err(diag(line, 8, "expected `DEFINE Name (values) : SetType`"));
        };
        let open = name_and_values
            .find('(')
            .ok_or_else(|| diag(line, 8, "DEFINE requires a parenthesized value list"))?;
        let close = name_and_values
            .rfind(')')
            .filter(|close| *close > open)
            .ok_or_else(|| diag(line, open + 9, "expected `)` after set values"))?;
        if !name_and_values[close + 1..].trim().is_empty() {
            return Err(diag(line, close + 9, "unexpected text after set values"));
        }
        let name = name_and_values[..open].trim();
        if !valid_identifier(name) {
            return Err(diag(line, 8, "invalid set name"));
        }
        let key = normalize(name);
        if self.variables.contains_key(&key) || self.constants.contains_key(&key) {
            return Err(diag(line, 8, format!("`{name}` is already declared")));
        }
        let type_name = normalize(type_name.trim());
        let Some(Value::Set {
            type_name: set_type,
            element_type,
            ..
        }) = self.types.get(&type_name).cloned()
        else {
            return Err(diag(line, 8, format!("`{type_name}` is not a set type")));
        };
        let mut values = Vec::new();
        for item in split_commas(&name_and_values[open + 1..close]) {
            let expression = ExprParser::parse(item.trim(), line)?;
            let value = match expression {
                Expr::Integer(n) => Value::Integer(n),
                Expr::Real(n) => Value::Real(n),
                Expr::Character(c) => Value::Character(c),
                Expr::String(s) => Value::String(s),
                Expr::Boolean(b) => Value::Boolean(b),
                Expr::Variable(member) => self
                    .constants
                    .get(&normalize(&member))
                    .cloned()
                    .ok_or_else(|| diag(line, 8, format!("unknown set member `{member}`")))?,
                _ => {
                    return Err(diag(
                        line,
                        8,
                        "set values must be literals or defined values",
                    ));
                }
            };
            let value = assign_target_type(&element_type, value).map_err(|error| {
                diag(
                    line,
                    8,
                    format!("set value has the wrong type: {}", error.message),
                )
            })?;
            if !values.contains(&value) {
                values.push(value);
            }
        }
        if values.is_empty() {
            return Err(diag(
                line,
                8,
                "set definition must include at least one value",
            ));
        }
        self.constants.insert(
            key,
            Value::Set {
                type_name: set_type,
                element_type,
                values,
            },
        );
        self.at += 1;
        Ok(())
    }

    fn constant(&mut self, source: &str, line: usize) -> Result<(), Diagnostic> {
        let (key, value) = parse_constant_declaration(source, line)?;
        if self.variables.contains_key(&key) || self.constants.contains_key(&key) {
            return Err(diag(line, 10, format!("`{key}` is already declared")));
        }
        self.constants.insert(key, value);
        Ok(())
    }

    fn routine(&mut self, line: usize, header: &str) -> Result<Routine, Diagnostic> {
        let upper = header.to_ascii_uppercase();
        let is_function = upper.starts_with("FUNCTION ");
        let start = if is_function { 9 } else { 10 };
        let open = header
            .find('(')
            .ok_or_else(|| diag(line, start, "routine parameters must use parentheses"))?;
        let close = header[open + 1..]
            .find(')')
            .map(|n| n + open + 1)
            .ok_or_else(|| diag(line, open + 1, "expected `)`"))?;
        let name = header[start..open].trim().to_owned();
        if !valid_identifier(&name) {
            return Err(diag(line, start, "invalid routine name"));
        }
        let mut parameters = Vec::new();
        let mut by_reference = false;
        for parameter in split_commas(&header[open + 1..close]) {
            if parameter.trim().is_empty() {
                continue;
            }
            let mut declaration = parameter.trim();
            let upper_declaration = declaration.to_ascii_uppercase();
            if upper_declaration.starts_with("BYREF ") {
                by_reference = true;
                declaration = declaration[6..].trim_start();
            } else if upper_declaration.starts_with("BYVAL ") {
                by_reference = false;
                declaration = declaration[6..].trim_start();
            }
            let Some((param, ty)) = declaration.split_once(':') else {
                return Err(diag(line, open + 1, "expected `parameter : type`"));
            };
            let param = param.trim().to_owned();
            if !valid_identifier(&param) {
                return Err(diag(line, open + 1, "invalid parameter name"));
            }
            parameters.push(RoutineParameter {
                name: param,
                type_name: ty.trim().to_owned(),
                by_reference,
            });
        }
        if is_function && parameters.iter().any(|parameter| parameter.by_reference) {
            return Err(diag(
                line,
                open + 1,
                "Cambridge pseudocode functions cannot have BYREF parameters",
            ));
        }
        let return_type = if is_function {
            let rest = header[close + 1..].trim();
            let upper_rest = rest.to_ascii_uppercase();
            let position = upper_rest
                .find("RETURNS")
                .ok_or_else(|| diag(line, close + 2, "function header requires RETURNS"))?;
            Some(rest[position + 7..].trim().to_owned())
        } else {
            None
        };
        let end = if is_function {
            "ENDFUNCTION"
        } else {
            "ENDPROCEDURE"
        };
        self.at += 1;
        self.routine_body_depth += 1;
        let body = self.block_until(&[end])?;
        self.routine_body_depth -= 1;
        if self.at >= self.lines.len() || !eq(&self.lines[self.at].1, end) {
            return Err(diag(line, 1, format!("routine is missing {end}")));
        }
        self.at += 1;
        Ok(Routine {
            name,
            parameters,
            return_type,
            body,
            line,
        })
    }

    fn block_until(&mut self, terminators: &[&str]) -> Result<Vec<Stmt>, Diagnostic> {
        let mut body = Vec::new();
        while self.at < self.lines.len() {
            let (line, source) = self.lines[self.at].clone();
            if terminators.iter().any(|end| is_terminator(&source, end)) {
                break;
            }
            self.at += 1;
            let upper = source.to_ascii_uppercase();
            if upper.starts_with("DECLARE ") {
                let (name, value) = parse_declaration(&source[8..], line, &self.types)?;
                body.push(Stmt::Declare(name, value));
            } else if upper.starts_with("CONSTANT ") {
                if self.routine_body_depth == 0 {
                    return Err(diag(
                        line,
                        1,
                        "CONSTANT inside a control block must be declared in a routine",
                    ));
                }
                body.push(
                    parse_constant_declaration(&source[9..], line)
                        .map(|(name, value)| Stmt::Constant(name, value))?,
                );
            } else {
                body.push(self.statement(line, &source)?);
            }
        }
        Ok(body)
    }

    fn statement(&mut self, line: usize, source: &str) -> Result<Stmt, Diagnostic> {
        let upper = source.to_ascii_uppercase();
        if upper.starts_with("IF ") {
            let then = upper
                .find(" THEN")
                .ok_or_else(|| diag(line, 1, "IF requires THEN"))?;
            let condition = ExprParser::parse(source[3..then].trim(), line)?;
            let yes = self.block_until(&["ELSE", "ENDIF"])?;
            let no = if self.at < self.lines.len() && eq(&self.lines[self.at].1, "ELSE") {
                self.at += 1;
                self.block_until(&["ENDIF"])?
            } else {
                Vec::new()
            };
            if self.at >= self.lines.len() || !eq(&self.lines[self.at].1, "ENDIF") {
                return Err(diag(line, 1, "IF is missing ENDIF"));
            }
            self.at += 1;
            return Ok(Stmt::If(condition, yes, no));
        }
        if upper.starts_with("CASE OF ") {
            let selector = ExprParser::parse(source[8..].trim(), line)?;
            let mut arms = Vec::new();
            let mut otherwise_seen = false;
            while self.at < self.lines.len() {
                let (arm_line, text) = self.lines[self.at].clone();
                if eq(&text, "ENDCASE") {
                    self.at += 1;
                    return Ok(Stmt::Case(selector, arms));
                }
                let Some(colon) = case_colon(&text) else {
                    return Err(diag(
                        arm_line,
                        1,
                        "CASE branch requires a label followed by `:`",
                    ));
                };
                let label = text[..colon].trim();
                if otherwise_seen {
                    return Err(diag(arm_line, 1, "OTHERWISE must be the final CASE branch"));
                }
                let (lower, upper_bound) = if eq(label, "OTHERWISE") {
                    otherwise_seen = true;
                    (None, None)
                } else if let Some(range) = find_word(label, "TO") {
                    (
                        Some(ExprParser::parse(label[..range].trim(), arm_line)?),
                        Some(ExprParser::parse(label[range + 2..].trim(), arm_line)?),
                    )
                } else {
                    (Some(ExprParser::parse(label, arm_line)?), None)
                };
                self.at += 1;
                let mut statements = Vec::new();
                let inline = text[colon + 1..].trim();
                if !inline.is_empty() {
                    statements.push(self.statement(arm_line, inline)?);
                }
                while self.at < self.lines.len() {
                    let (next_line, next) = self.lines[self.at].clone();
                    if eq(&next, "ENDCASE") || case_colon(&next).is_some() {
                        break;
                    }
                    self.at += 1;
                    statements.push(self.statement(next_line, &next)?);
                }
                arms.push(CaseArm {
                    lower,
                    upper: upper_bound,
                    body: statements,
                });
            }
            return Err(diag(line, 1, "CASE is missing ENDCASE"));
        }
        if upper.starts_with("FOR ") {
            let body_text = &source[4..];
            let arrow = body_text.find('←').or_else(|| body_text.find("<-"));
            let Some(arrow) = arrow else {
                return Err(diag(line, 1, "FOR requires an assignment arrow"));
            };
            let name = body_text[..arrow].trim().to_owned();
            let arrow_width = if body_text[arrow..].starts_with('←') {
                '←'.len_utf8()
            } else {
                2
            };
            let rest = body_text[arrow + arrow_width..].trim();
            let upper_rest = rest.to_ascii_uppercase();
            let to = upper_rest
                .find(" TO ")
                .ok_or_else(|| diag(line, arrow + 1, "FOR requires TO"))?;
            let start = ExprParser::parse(rest[..to].trim(), line)?;
            let after_to = &rest[to + 4..];
            let step_pos = after_to.to_ascii_uppercase().find(" STEP ");
            let (end_text, step_text) = if let Some(pos) = step_pos {
                (&after_to[..pos], &after_to[pos + 6..])
            } else {
                (after_to, "1")
            };
            let end = ExprParser::parse(end_text.trim(), line)?;
            let step = ExprParser::parse(step_text.trim(), line)?;
            let body = self.block_until(&["NEXT"])?;
            if self.at >= self.lines.len()
                || !self.lines[self.at]
                    .1
                    .to_ascii_uppercase()
                    .starts_with("NEXT")
            {
                return Err(diag(line, 1, "FOR is missing NEXT"));
            }
            let next_identifier = self.lines[self.at].1[4..].trim();
            if !next_identifier.is_empty() && !eq(next_identifier, &name) {
                return Err(diag(
                    self.lines[self.at].0,
                    5,
                    "NEXT identifier does not match FOR counter",
                ));
            }
            self.at += 1;
            return Ok(Stmt::For(name, start, end, step, body));
        }
        if upper.starts_with("WHILE ") {
            let condition_end = upper.find(" DO").unwrap_or(source.len());
            let condition = ExprParser::parse(source[6..condition_end].trim(), line)?;
            let body = self.block_until(&["ENDWHILE"])?;
            if self.at >= self.lines.len() {
                return Err(diag(line, 1, "WHILE is missing ENDWHILE"));
            }
            self.at += 1;
            return Ok(Stmt::While(condition, body));
        }
        if eq(source, "REPEAT") {
            let body = self.block_until(&["UNTIL"])?;
            if self.at >= self.lines.len() {
                return Err(diag(line, 1, "REPEAT is missing UNTIL"));
            }
            let condition_line = self.lines[self.at].0;
            let condition_text = self.lines[self.at].1[5..].trim();
            let condition = ExprParser::parse(condition_text, condition_line)?;
            self.at += 1;
            return Ok(Stmt::Repeat(body, condition));
        }
        if upper.starts_with("OPENFILE ") {
            let rest = source[9..].trim();
            let Some(for_at) = find_word(rest, "FOR") else {
                return Err(diag(line, 1, "OPENFILE requires FOR and a file mode"));
            };
            let path = ExprParser::parse(rest[..for_at].trim(), line)?;
            let mode = match rest[for_at + 3..].trim().to_ascii_uppercase().as_str() {
                "READ" => FileMode::Read,
                "WRITE" => FileMode::Write,
                "APPEND" => FileMode::Append,
                "RANDOM" => FileMode::Random,
                mode => return Err(diag(line, for_at + 4, format!("unknown file mode {mode}"))),
            };
            return Ok(Stmt::OpenFile(path, mode));
        }
        if upper.starts_with("READFILE ") {
            let parts = split_commas(&source[9..]);
            if parts.len() != 2 {
                return Err(diag(
                    line,
                    1,
                    "READFILE requires a file and destination variable",
                ));
            }
            return Ok(Stmt::ReadFile(
                ExprParser::parse(parts[0].trim(), line)?,
                parse_target(parts[1].trim(), line)?,
            ));
        }
        if upper.starts_with("WRITEFILE ") {
            let parts = split_commas(&source[10..]);
            if parts.len() != 2 {
                return Err(diag(
                    line,
                    1,
                    "WRITEFILE requires a file and one line of data",
                ));
            }
            return Ok(Stmt::WriteFile(
                ExprParser::parse(parts[0].trim(), line)?,
                ExprParser::parse(parts[1].trim(), line)?,
            ));
        }
        if upper.starts_with("CLOSEFILE ") {
            return Ok(Stmt::CloseFile(ExprParser::parse(
                source[10..].trim(),
                line,
            )?));
        }
        if upper.starts_with("SEEK ") {
            let parts = split_commas(&source[5..]);
            if parts.len() != 2 {
                return Err(diag(line, 1, "SEEK requires a file and record address"));
            }
            return Ok(Stmt::Seek(
                ExprParser::parse(parts[0].trim(), line)?,
                ExprParser::parse(parts[1].trim(), line)?,
            ));
        }
        if upper.starts_with("GETRECORD ") {
            let parts = split_commas(&source[10..]);
            if parts.len() != 2 {
                return Err(diag(
                    line,
                    1,
                    "GETRECORD requires a file and destination variable",
                ));
            }
            return Ok(Stmt::GetRecord(
                ExprParser::parse(parts[0].trim(), line)?,
                parse_target(parts[1].trim(), line)?,
            ));
        }
        if upper.starts_with("PUTRECORD ") {
            let parts = split_commas(&source[10..]);
            if parts.len() != 2 {
                return Err(diag(line, 1, "PUTRECORD requires a file and record value"));
            }
            return Ok(Stmt::PutRecord(
                ExprParser::parse(parts[0].trim(), line)?,
                ExprParser::parse(parts[1].trim(), line)?,
            ));
        }
        if upper.starts_with("INPUT ") {
            return Ok(Stmt::Input(parse_target(source[6..].trim(), line)?));
        }
        if upper.starts_with("OUTPUT ") {
            let expressions = split_commas(&source[7..])
                .into_iter()
                .map(|s| ExprParser::parse(s.trim(), line))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Stmt::Output(expressions));
        }
        if upper.starts_with("CALL ") {
            let (name, args) = call_parts(source[5..].trim(), line)?;
            return Ok(Stmt::Call(name, args));
        }
        if upper.starts_with("SUPER.") {
            let expression = ExprParser::parse(source, line)?;
            if let Expr::MethodCall(receiver, name, args) = expression {
                if matches!(*receiver, Expr::Variable(ref keyword) if eq(keyword, "SUPER")) {
                    return Ok(Stmt::SuperCall(name, args));
                }
            }
            return Err(diag(line, 1, "SUPER requires a parent-class method call"));
        }
        if upper == "RETURN" || upper.starts_with("RETURN ") {
            let rest = source[6..].trim();
            return Ok(Stmt::Return(if rest.is_empty() {
                None
            } else {
                Some(ExprParser::parse(rest, line)?)
            }));
        }
        if let Some(position) = assignment_arrow(source) {
            let width = if source[position..].starts_with('←') {
                '←'.len_utf8()
            } else {
                2
            };
            let target = parse_target(source[..position].trim(), line)?;
            let value = ExprParser::parse(source[position + width..].trim(), line)?;
            return Ok(Stmt::Assign(target, value));
        }
        if let Ok(expression) = ExprParser::parse(source, line) {
            if matches!(expression, Expr::MethodCall(_, _, _)) {
                return Ok(Stmt::MethodCall(expression));
            }
        }
        Err(diag(line, 1, format!("unsupported statement `{source}`")))
    }
}

fn parse_declaration(
    source: &str,
    line: usize,
    types: &HashMap<String, Value>,
) -> Result<(String, Value), Diagnostic> {
    let Some((name, type_text)) = source.split_once(':') else {
        return Err(diag(line, 9, "expected `name : type`"));
    };
    let name = name.trim();
    if !valid_identifier(name) {
        return Err(diag(line, 9, "invalid identifier"));
    }
    let value = default_value(type_text.trim(), types).ok_or_else(|| {
        diag(
            line,
            9,
            format!("unsupported data type `{}`", type_text.trim()),
        )
    })?;
    Ok((normalize(name), value))
}

fn parse_constant_declaration(source: &str, line: usize) -> Result<(String, Value), Diagnostic> {
    let Some((name, literal)) = source.split_once('=') else {
        return Err(diag(line, 10, "expected `CONSTANT name = literal`"));
    };
    let name = name.trim();
    if !valid_identifier(name) {
        return Err(diag(line, 10, "invalid constant identifier"));
    }
    let expression = ExprParser::parse(literal.trim(), line)?;
    let value = match expression {
        Expr::Integer(n) => Value::Integer(n),
        Expr::Real(n) => Value::Real(n),
        Expr::Character(c) => Value::Character(c),
        Expr::String(s) => Value::String(s),
        Expr::Boolean(b) => Value::Boolean(b),
        Expr::Date(date) => Value::Date(Some(date)),
        _ => return Err(diag(line, 10, "constants must use a literal value")),
    };
    Ok((normalize(name), value))
}

fn default_value(type_text: &str, types: &HashMap<String, Value>) -> Option<Value> {
    let ty = type_text.trim();
    let upper = ty.to_ascii_uppercase();
    match upper.as_str() {
        "INTEGER" => return Some(Value::Integer(0)),
        "REAL" => return Some(Value::Real(0.0)),
        "CHAR" => return Some(Value::Character('\0')),
        "STRING" => return Some(Value::String(String::new())),
        "BOOLEAN" => return Some(Value::Boolean(false)),
        "DATE" => return Some(Value::Date(None)),
        _ => {}
    }
    if let Some(value) = types.get(&normalize(ty)) {
        return Some(value.clone());
    }
    if !upper.starts_with("ARRAY[") {
        return None;
    }
    let close = ty.find(']')?;
    let mut bounds = Vec::new();
    for dimension in split_commas(&ty[6..close]) {
        let (lower, upper) = dimension.split_once(':')?;
        let lower: i64 = lower.trim().parse().ok()?;
        let upper: i64 = upper.trim().parse().ok()?;
        if upper < lower {
            return None;
        }
        bounds.push((lower, upper));
    }
    if bounds.is_empty() {
        return None;
    }
    let rest = ty[close + 1..].trim();
    let upper_rest = rest.to_ascii_uppercase();
    let of = upper_rest.strip_prefix("OF ")?;
    let element = default_value(of.trim(), types)?;
    let mut count = 1_usize;
    for (low, high) in &bounds {
        let width = *high as i128 - *low as i128 + 1;
        if !(1..=1_000_000).contains(&width) {
            return None;
        }
        count = count.checked_mul(width as usize)?;
        if count > 1_000_000 {
            return None;
        }
    }
    Some(Value::Array {
        bounds,
        values: vec![element; count],
    })
}

fn value_type_signature(value: &Value) -> String {
    match value {
        Value::Integer(_) => "INTEGER".into(),
        Value::Real(_) => "REAL".into(),
        Value::Character(_) => "CHAR".into(),
        Value::String(_) => "STRING".into(),
        Value::Boolean(_) => "BOOLEAN".into(),
        Value::Date(_) => "DATE".into(),
        Value::Enumeration { type_name, .. } => format!("ENUM:{type_name}"),
        Value::Array { bounds, values } => {
            let element = values.first().map(value_type_signature).unwrap_or_default();
            format!("ARRAY:{bounds:?}:{element}")
        }
        Value::Record { type_name, .. } => format!("RECORD:{type_name}"),
        Value::Object { class_name, .. } => format!("OBJECT:{class_name}"),
        Value::Set { type_name, .. } => format!("SET:{type_name}"),
        Value::Pointer { target_type, .. } => format!("POINTER:{target_type}"),
    }
}

fn valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn split_commas(source: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let (mut start, mut depth, mut quote) = (0, 0_i32, None);
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let mut index = 0;
    while index < chars.len() {
        let (byte, c) = chars[index];
        if c == '"' || c == '\'' {
            if quote == Some(c) && chars.get(index + 1).is_some_and(|(_, next)| *next == c) {
                index += 2;
                continue;
            }
            quote = if quote == Some(c) {
                None
            } else if quote.is_none() {
                Some(c)
            } else {
                quote
            };
        } else if quote.is_none() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                ',' if depth == 0 => {
                    result.push(&source[start..byte]);
                    start = byte + 1;
                }
                _ => {}
            }
        }
        index += 1;
    }
    result.push(&source[start..]);
    result
}

fn case_colon(source: &str) -> Option<usize> {
    let (mut depth, mut quote) = (0_i32, None);
    for (index, character) in source.char_indices() {
        if character == '"' || character == '\'' || character == 'ꞌ' {
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
        } else if quote.is_none() {
            match character {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                ':' if depth == 0 => return Some(index),
                _ => {}
            }
        }
    }
    None
}

fn find_word(source: &str, word: &str) -> Option<usize> {
    let upper = source.to_ascii_uppercase();
    let needle = format!(" {word} ");
    let chars = source.char_indices().collect::<Vec<_>>();
    let mut quote = None;
    let mut at = 0;
    while at < chars.len() {
        let (byte, character) = chars[at];
        if matches!(character, '"' | '\'' | 'ꞌ') {
            if quote == Some(character)
                && chars
                    .get(at + 1)
                    .is_some_and(|(_, next)| *next == character)
            {
                at += 2;
                continue;
            }
            quote = if quote == Some(character) {
                None
            } else if quote.is_none() {
                Some(character)
            } else {
                quote
            };
        } else if quote.is_none() && upper[byte..].starts_with(&needle) {
            return Some(byte + 1);
        }
        at += 1;
    }
    None
}

fn assignment_arrow(source: &str) -> Option<usize> {
    let (mut depth, mut quote) = (0_i32, None);
    let chars: Vec<(usize, char)> = source.char_indices().collect();
    let mut i = 0;
    while i < chars.len() {
        let (byte, c) = chars[i];
        if c == '"' || c == '\'' {
            quote = if quote == Some(c) {
                None
            } else if quote.is_none() {
                Some(c)
            } else {
                quote
            };
        } else if quote.is_none() {
            match c {
                '(' | '[' => depth += 1,
                ')' | ']' => depth -= 1,
                '←' if depth == 0 => return Some(byte),
                '<' if depth == 0 && chars.get(i + 1).is_some_and(|(_, next)| *next == '-') => {
                    return Some(byte);
                }
                _ => {}
            }
        }
        i += 1;
    }
    None
}

fn parse_target(source: &str, line: usize) -> Result<Target, Diagnostic> {
    let expression = ExprParser::parse(source.trim(), line)?;
    expression_target(&expression).map_err(|_| diag(line, 1, "invalid assignment target"))
}

fn call_parts(source: &str, line: usize) -> Result<(String, Vec<Expr>), Diagnostic> {
    let open = source
        .find('(')
        .ok_or_else(|| diag(line, 1, "CALL requires parentheses"))?;
    let close = source
        .rfind(')')
        .ok_or_else(|| diag(line, open + 1, "expected `)`"))?;
    if !source[close + 1..].trim().is_empty() {
        return Err(diag(line, close + 2, "unexpected text after call"));
    }
    let name = source[..open].trim();
    if !valid_identifier(name) {
        return Err(diag(line, 1, "invalid routine name"));
    }
    let args = split_commas(&source[open + 1..close])
        .into_iter()
        .filter(|s| !s.trim().is_empty())
        .map(|s| ExprParser::parse(s.trim(), line))
        .collect::<Result<Vec<_>, _>>()?;
    Ok((normalize(name), args))
}

pub trait Host {
    fn input(&mut self) -> Result<String, String>;
    /// Returns a value compatible with `expected`. Hosts may override this to
    /// collect arrays, records, sets, objects, or pointers as typed values.
    /// The default implementation parses the scalar value from `input()`.
    fn input_value(&mut self, expected: &Value) -> Result<Value, String> {
        let text = self.input()?;
        parse_input_value(expected, &text)
    }
    fn output(&mut self, text: &str) -> Result<(), String>;
    fn random(&mut self, upper_exclusive: f64) -> Result<f64, String>;
    /// Opens a named file. WRITE must replace existing content; APPEND preserves it.
    fn open_file(&mut self, _path: &str, _mode: FileMode) -> Result<(), String> {
        Err("file service is not configured".into())
    }
    /// Reads one UTF-8 text line, returning None at end of file.
    fn read_file_line(&mut self, _path: &str) -> Result<Option<String>, String> {
        Err("file service is not configured".into())
    }
    /// Writes one text line, including the host's line terminator.
    fn write_file_line(&mut self, _path: &str, _line: &str) -> Result<(), String> {
        Err("file service is not configured".into())
    }
    /// Returns true for an empty file or after the final text line was read.
    fn file_eof(&mut self, _path: &str) -> Result<bool, String> {
        Err("file service is not configured".into())
    }
    fn close_file(&mut self, _path: &str) -> Result<(), String> {
        Err("file service is not configured".into())
    }
    /// Moves to a one-based record address.
    fn seek_record(&mut self, _path: &str, _address: i64) -> Result<(), String> {
        Err("file service is not configured".into())
    }
    /// Reads a record at the current position in a random file.
    fn read_record(&mut self, _path: &str) -> Result<Value, String> {
        Err("file service is not configured".into())
    }
    /// Replaces the record at the current position in a random file.
    fn write_record(&mut self, _path: &str, _value: &Value) -> Result<(), String> {
        Err("file service is not configured".into())
    }
}

fn parse_input_value(expected: &Value, raw: &str) -> Result<Value, String> {
    match expected {
        Value::Integer(_) => raw
            .trim()
            .parse()
            .map(Value::Integer)
            .map_err(|_| "integer input expected".to_owned()),
        Value::Real(_) => raw
            .trim()
            .parse()
            .map(Value::Real)
            .map_err(|_| "real input expected".to_owned()),
        Value::Character(_) => {
            let mut chars = raw.trim_end_matches(['\r', '\n']).chars();
            let Some(character) = chars.next() else {
                return Err("character input expected".to_owned());
            };
            if chars.next().is_some() {
                return Err("character input expected".to_owned());
            }
            Ok(Value::Character(character))
        }
        Value::String(_) => Ok(Value::String(raw.trim_end_matches(['\r', '\n']).to_owned())),
        Value::Boolean(_) => match raw.trim().to_ascii_uppercase().as_str() {
            "TRUE" => Ok(Value::Boolean(true)),
            "FALSE" => Ok(Value::Boolean(false)),
            _ => Err("TRUE or FALSE input expected".to_owned()),
        },
        Value::Date(_) => parse_date_text(raw.trim())
            .map(|date| Value::Date(Some(date)))
            .ok_or_else(|| "valid dd/mm/yyyy input expected".to_owned()),
        Value::Enumeration {
            type_name, members, ..
        } => {
            let input = raw.trim();
            let Some((ordinal, member)) = members
                .iter()
                .enumerate()
                .find(|(_, member)| member.eq_ignore_ascii_case(input))
            else {
                return Err("unknown enumerated input value".to_owned());
            };
            Ok(Value::Enumeration {
                type_name: type_name.clone(),
                member: member.clone(),
                ordinal,
                members: members.clone(),
            })
        }
        Value::Array { .. }
        | Value::Record { .. }
        | Value::Object { .. }
        | Value::Set { .. }
        | Value::Pointer { .. } => Err(
            "typed input for this value requires a host that implements `input_value`".to_owned(),
        ),
    }
}

#[derive(Clone, Debug, Default)]
pub struct Execution {
    pub edition: Option<Edition>,
    pub variables: HashMap<String, Value>,
    pub executed_statements: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeError {
    pub message: String,
}
impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}
impl std::error::Error for RuntimeError {}

pub struct Interpreter<H: Host> {
    host: H,
    step_limit: usize,
    recursion_limit: usize,
}
impl<H: Host> Interpreter<H> {
    pub fn new(host: H) -> Self {
        Self {
            host,
            step_limit: 5_000_000,
            recursion_limit: 512,
        }
    }
    pub fn with_step_limit(mut self, limit: usize) -> Self {
        self.step_limit = limit.max(1);
        self
    }
    pub fn with_recursion_limit(mut self, limit: usize) -> Self {
        self.recursion_limit = limit.max(1);
        self
    }
    pub fn host(&self) -> &H {
        &self.host
    }
    pub fn host_mut(&mut self) -> &mut H {
        &mut self.host
    }
    pub fn run(&mut self, program: &Program) -> Result<Execution, RuntimeError> {
        let mut runtime = Runtime {
            program,
            host: &mut self.host,
            steps: 0,
            step_limit: self.step_limit,
            recursion_depth: 0,
            recursion_limit: self.recursion_limit,
            references: HashMap::new(),
            local_constants: Vec::new(),
            reference_sequence: 0,
            pointers: HashMap::new(),
            pointer_sequence: 0,
            open_files: HashMap::new(),
            current_class: None,
            current_self: None,
        };
        let mut variables = program.variables.clone();
        let outcome = runtime.execute_block(&mut variables, &program.body);
        let executed_statements = runtime.steps;
        let cleanup = runtime.close_open_files();
        match outcome {
            Err(error) => Err(error),
            Ok(Flow::Next) => {
                cleanup?;
                Ok(Execution {
                    edition: Some(program.edition),
                    variables,
                    executed_statements,
                })
            }
            Ok(Flow::Return(_)) => Err(runtime_error("RETURN used outside a function")),
        }
    }
}

enum Flow {
    Next,
    Return(Option<Value>),
}

struct Runtime<'a, H: Host> {
    program: &'a Program,
    host: &'a mut H,
    steps: usize,
    step_limit: usize,
    recursion_depth: usize,
    recursion_limit: usize,
    references: HashMap<String, Target>,
    local_constants: Vec<HashMap<String, Value>>,
    reference_sequence: u64,
    pointers: HashMap<u64, Target>,
    pointer_sequence: u64,
    open_files: HashMap<String, FileMode>,
    current_class: Option<String>,
    current_self: Option<String>,
}

impl<H: Host> Runtime<'_, H> {
    fn is_constant_name(&self, name: &str) -> bool {
        self.program.constants.contains_key(name)
            || self
                .local_constants
                .last()
                .is_some_and(|constants| constants.contains_key(name))
    }

    fn aliased_target(&self, target: &Target) -> Option<Target> {
        match target {
            Target::Variable(name) => self.references.get(name).cloned(),
            Target::Index(name, indices) => match self.references.get(name) {
                Some(Target::Variable(variable)) => {
                    Some(Target::Index(variable.clone(), indices.clone()))
                }
                Some(base) => Some(Target::NestedIndex(Box::new(base.clone()), indices.clone())),
                _ => None,
            },
            Target::Field(base, field) => self
                .aliased_target(base)
                .map(|base| Target::Field(Box::new(base), field.clone())),
            Target::NestedIndex(base, indices) => self
                .aliased_target(base)
                .map(|base| Target::NestedIndex(Box::new(base), indices.clone())),
            Target::Dereference(_) => None,
        }
    }

    fn pointer_target(
        &mut self,
        variables: &mut HashMap<String, Value>,
        pointer_expression: &Expr,
    ) -> Result<Target, RuntimeError> {
        let pointer = self.evaluate(variables, pointer_expression)?;
        let Value::Pointer {
            target_type,
            address: Some(address),
            ..
        } = pointer
        else {
            return Err(runtime_error("cannot dereference a null pointer"));
        };
        let target = self
            .pointers
            .get(&address)
            .cloned()
            .ok_or_else(|| runtime_error("pointer address is no longer available"))?;
        let value = self.target_value(&target, variables)?;
        if value_type_signature(&value) != target_type {
            return Err(runtime_error(
                "pointer target no longer has its declared type",
            ));
        }
        Ok(target)
    }

    fn file_path(
        &mut self,
        variables: &mut HashMap<String, Value>,
        expression: &Expr,
    ) -> Result<String, RuntimeError> {
        match self.evaluate(variables, expression)? {
            Value::String(path) if !path.is_empty() => Ok(path),
            Value::String(_) => Err(runtime_error("file name cannot be empty")),
            _ => Err(runtime_error("file identifier must evaluate to a STRING")),
        }
    }

    fn open_file(&mut self, path: String, mode: FileMode) -> Result<(), RuntimeError> {
        if self.open_files.contains_key(&path) {
            return Err(runtime_error(format!("file {path} is already open")));
        }
        self.host.open_file(&path, mode).map_err(runtime_error)?;
        self.open_files.insert(path, mode);
        Ok(())
    }

    fn require_field_access(&self, class_name: &str, field_name: &str) -> Result<(), RuntimeError> {
        let field = self
            .program
            .classes
            .get(class_name)
            .and_then(|class| class.fields.iter().find(|field| field.name == field_name))
            .ok_or_else(|| runtime_error(format!("undefined object property `{field_name}`")))?;
        if field.visibility == Visibility::Private
            && self.current_class.as_deref() != Some(field.owner.as_str())
        {
            return Err(runtime_error(format!(
                "private property `{field_name}` is not accessible here"
            )));
        }
        Ok(())
    }

    fn require_file_mode(&self, path: &str, expected: &[FileMode]) -> Result<(), RuntimeError> {
        let Some(mode) = self.open_files.get(path) else {
            return Err(runtime_error(format!("file {path} is not open")));
        };
        if expected.contains(mode) {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "file {path} is open in the wrong mode"
            )))
        }
    }

    fn file_eof(&mut self, path: &str) -> Result<bool, RuntimeError> {
        self.require_file_mode(path, &[FileMode::Read])?;
        self.host.file_eof(path).map_err(runtime_error)
    }

    fn close_open_files(&mut self) -> Result<(), RuntimeError> {
        let paths = self.open_files.keys().cloned().collect::<Vec<_>>();
        let mut first_error = None;
        for path in paths {
            if let Err(error) = self.host.close_file(&path) {
                first_error.get_or_insert_with(|| runtime_error(error));
            }
            self.open_files.remove(&path);
        }
        first_error.map_or(Ok(()), Err)
    }

    fn execute_block(
        &mut self,
        variables: &mut HashMap<String, Value>,
        body: &[Stmt],
    ) -> Result<Flow, RuntimeError> {
        for statement in body {
            self.steps += 1;
            if self.steps > self.step_limit {
                return Err(runtime_error("execution step limit exceeded"));
            }
            match statement {
                Stmt::Declare(name, value) => {
                    let key = normalize(name);
                    if let Some(Target::Variable(storage)) = self.references.get(&key) {
                        variables.insert(storage.clone(), value.clone());
                    } else {
                        variables.insert(key, value.clone());
                    }
                }
                Stmt::Constant(_, _) => {}
                Stmt::Assign(target, expression) => {
                    let value = self.evaluate(variables, expression)?;
                    if let Some(name) = target_root_name(target) {
                        if self.is_constant_name(name) {
                            return Err(runtime_error(format!(
                                "constant `{name}` cannot be changed"
                            )));
                        }
                    }
                    self.assign_target(target, value, variables)?;
                }
                Stmt::Input(target) => {
                    if let Some(name) = target_root_name(target)
                        && self.is_constant_name(name)
                    {
                        return Err(runtime_error(format!(
                            "constant `{name}` cannot be changed"
                        )));
                    }
                    let current = self.target_value(target, variables)?;
                    let value = self.host.input_value(&current).map_err(runtime_error)?;
                    let value = assign_target_type(&current, value)?;
                    self.assign_target(target, value, variables)?;
                }
                Stmt::Output(expressions) => {
                    let mut text = String::new();
                    for expression in expressions {
                        text.push_str(&self.evaluate(variables, expression)?.to_string());
                    }
                    self.host.output(&text).map_err(runtime_error)?;
                }
                Stmt::If(condition, yes, no) => {
                    let branch = if as_bool(self.evaluate(variables, condition)?)? {
                        yes
                    } else {
                        no
                    };
                    match self.execute_block(variables, branch)? {
                        Flow::Next => {}
                        flow @ Flow::Return(_) => return Ok(flow),
                    }
                }
                Stmt::Case(selector, arms) => {
                    let selected = self.evaluate(variables, selector)?;
                    let mut default = None;
                    let mut match_body = None;
                    for arm in arms {
                        let Some(lower) = &arm.lower else {
                            default = Some(&arm.body);
                            continue;
                        };
                        let lower = self.evaluate(variables, lower)?;
                        let matches = if let Some(upper) = &arm.upper {
                            let upper = self.evaluate(variables, upper)?;
                            let above = as_bool(binary(selected.clone(), Op::Ge, lower)?)?;
                            let below = as_bool(binary(selected.clone(), Op::Le, upper)?)?;
                            above && below
                        } else {
                            values_equal(&selected, &lower)
                        };
                        if matches {
                            match_body = Some(&arm.body);
                            break;
                        }
                    }
                    if let Some(body) = match_body.or(default) {
                        match self.execute_block(variables, body)? {
                            Flow::Next => {}
                            flow @ Flow::Return(_) => return Ok(flow),
                        }
                    }
                }
                Stmt::For(name, start, end, step, body) => {
                    let key = normalize(name);
                    if !variables.contains_key(&key) {
                        return Err(runtime_error(format!("undeclared FOR counter `{name}`")));
                    }
                    let mut value = as_integer(self.evaluate(variables, start)?)?;
                    let end = as_integer(self.evaluate(variables, end)?)?;
                    let step = as_integer(self.evaluate(variables, step)?)?;
                    if step == 0 {
                        return Err(runtime_error("FOR step cannot be zero"));
                    }
                    while if step > 0 { value <= end } else { value >= end } {
                        self.loop_tick()?;
                        self.assign_target(
                            &Target::Variable(key.clone()),
                            Value::Integer(value),
                            variables,
                        )?;
                        match self.execute_block(variables, body)? {
                            Flow::Next => {}
                            flow @ Flow::Return(_) => return Ok(flow),
                        }
                        let Some(next) = value.checked_add(step) else {
                            break;
                        };
                        value = next;
                    }
                }
                Stmt::While(condition, body) => {
                    while as_bool(self.evaluate(variables, condition)?)? {
                        self.loop_tick()?;
                        match self.execute_block(variables, body)? {
                            Flow::Next => {}
                            flow @ Flow::Return(_) => return Ok(flow),
                        }
                    }
                }
                Stmt::Repeat(body, condition) => loop {
                    self.loop_tick()?;
                    match self.execute_block(variables, body)? {
                        Flow::Next => {}
                        flow @ Flow::Return(_) => return Ok(flow),
                    }
                    if as_bool(self.evaluate(variables, condition)?)? {
                        break;
                    }
                },
                Stmt::Call(name, args) => {
                    self.call_routine(variables, name, args, false)?;
                }
                Stmt::MethodCall(expression) => {
                    let Expr::MethodCall(receiver, name, args) = expression else {
                        return Err(runtime_error("method call expected"));
                    };
                    self.call_method_expression(variables, receiver, name, args, false)?;
                }
                Stmt::SuperCall(name, args) => {
                    let current_class = self.current_class.clone().ok_or_else(|| {
                        runtime_error("SUPER is only valid inside a class method")
                    })?;
                    let parent = self
                        .program
                        .classes
                        .get(&current_class)
                        .and_then(|class| class.parent.clone())
                        .ok_or_else(|| runtime_error("SUPER has no parent class"))?;
                    let receiver = self
                        .current_self
                        .clone()
                        .ok_or_else(|| runtime_error("SUPER has no current object"))?;
                    self.call_method_target(
                        variables,
                        Target::Variable(receiver),
                        &parent,
                        name,
                        args,
                        false,
                    )?;
                }
                Stmt::Return(expression) => {
                    let value = expression
                        .as_ref()
                        .map(|e| self.evaluate(variables, e))
                        .transpose()?;
                    return Ok(Flow::Return(value));
                }
                Stmt::OpenFile(path, mode) => {
                    let path = self.file_path(variables, path)?;
                    self.open_file(path, *mode)?;
                }
                Stmt::ReadFile(path, target) => {
                    let path = self.file_path(variables, path)?;
                    self.require_file_mode(&path, &[FileMode::Read])?;
                    if !matches!(self.target_value(target, variables)?, Value::String(_)) {
                        return Err(runtime_error("READFILE destination must be a STRING"));
                    }
                    let line = self
                        .host
                        .read_file_line(&path)
                        .map_err(runtime_error)?
                        .ok_or_else(|| runtime_error("READFILE reached the end of file"))?;
                    self.assign_target(target, Value::String(line), variables)?;
                }
                Stmt::WriteFile(path, data) => {
                    let path = self.file_path(variables, path)?;
                    self.require_file_mode(&path, &[FileMode::Write, FileMode::Append])?;
                    let Value::String(line) = self.evaluate(variables, data)? else {
                        return Err(runtime_error("WRITEFILE data must be a STRING"));
                    };
                    self.host
                        .write_file_line(&path, &line)
                        .map_err(runtime_error)?;
                }
                Stmt::CloseFile(path) => {
                    let path = self.file_path(variables, path)?;
                    if !self.open_files.contains_key(&path) {
                        return Err(runtime_error(format!("file {path} is not open")));
                    }
                    self.host.close_file(&path).map_err(runtime_error)?;
                    self.open_files.remove(&path);
                }
                Stmt::Seek(path, address) => {
                    let path = self.file_path(variables, path)?;
                    self.require_file_mode(&path, &[FileMode::Random])?;
                    let address = as_integer(self.evaluate(variables, address)?)?;
                    if address < 1 {
                        return Err(runtime_error("SEEK address must be at least 1"));
                    }
                    self.host
                        .seek_record(&path, address)
                        .map_err(runtime_error)?;
                }
                Stmt::GetRecord(path, target) => {
                    let path = self.file_path(variables, path)?;
                    self.require_file_mode(&path, &[FileMode::Random])?;
                    if !matches!(self.target_value(target, variables)?, Value::Record { .. }) {
                        return Err(runtime_error("GETRECORD destination must be a record"));
                    }
                    let record = self.host.read_record(&path).map_err(runtime_error)?;
                    self.assign_target(target, record, variables)?;
                }
                Stmt::PutRecord(path, expression) => {
                    let path = self.file_path(variables, path)?;
                    self.require_file_mode(&path, &[FileMode::Random])?;
                    let record = self.evaluate(variables, expression)?;
                    if !matches!(record, Value::Record { .. }) {
                        return Err(runtime_error("PUTRECORD value must be a record"));
                    }
                    self.host
                        .write_record(&path, &record)
                        .map_err(runtime_error)?;
                }
            }
        }
        Ok(Flow::Next)
    }

    fn loop_tick(&mut self) -> Result<(), RuntimeError> {
        self.steps += 1;
        if self.steps > self.step_limit {
            Err(runtime_error("execution step limit exceeded"))
        } else {
            Ok(())
        }
    }

    fn target_value(
        &mut self,
        target: &Target,
        variables: &mut HashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        if let Some(aliased) = self.aliased_target(target) {
            return self.target_value(&aliased, variables);
        }
        match target {
            Target::Variable(name) => variables
                .get(name)
                .cloned()
                .ok_or_else(|| runtime_error(format!("undefined variable `{name}`"))),
            Target::Index(name, indices) => {
                let value = variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| runtime_error(format!("undefined array `{name}`")))?;
                let Value::Array { bounds, values } = value else {
                    return Err(runtime_error(format!("`{name}` is not an array")));
                };
                let offset = array_offset(&bounds, indices, variables, self)?;
                values
                    .get(offset)
                    .cloned()
                    .ok_or_else(|| runtime_error("array index out of bounds"))
            }
            Target::Field(base, field) => {
                let value = self.target_value(base, variables)?;
                let fields = match value {
                    Value::Record { fields, .. } => fields,
                    Value::Object { class_name, fields } => {
                        self.require_field_access(&class_name, field)?;
                        fields
                    }
                    _ => {
                        return Err(runtime_error(
                            "record or object expected before field access",
                        ));
                    }
                };
                fields
                    .into_iter()
                    .find(|(name, _)| name == field)
                    .map(|(_, value)| value)
                    .ok_or_else(|| runtime_error(format!("undefined record field `{field}`")))
            }
            Target::NestedIndex(base, indices) => {
                let value = self.target_value(base, variables)?;
                let Value::Array { bounds, values } = value else {
                    return Err(runtime_error("array value expected before indexing"));
                };
                let offset = array_offset(&bounds, indices, variables, self)?;
                values
                    .get(offset)
                    .cloned()
                    .ok_or_else(|| runtime_error("array index out of bounds"))
            }
            Target::Dereference(pointer_expression) => {
                let target = self.pointer_target(variables, pointer_expression)?;
                self.target_value(&target, variables)
            }
        }
    }

    fn assign_target(
        &mut self,
        target: &Target,
        value: Value,
        variables: &mut HashMap<String, Value>,
    ) -> Result<(), RuntimeError> {
        if let Some(aliased) = self.aliased_target(target) {
            return self.assign_target(&aliased, value, variables);
        }
        match target {
            Target::Variable(name) => {
                let current = variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| runtime_error(format!("undefined variable `{name}`")))?;
                let coerced = coerce_assignment(&current, value)?;
                variables.insert(name.clone(), coerced);
                Ok(())
            }
            Target::Index(name, indices) => {
                let current = variables
                    .get(name)
                    .cloned()
                    .ok_or_else(|| runtime_error(format!("undefined array `{name}`")))?;
                let Value::Array { bounds, .. } = current else {
                    return Err(runtime_error(format!("`{name}` is not an array")));
                };
                let offset = array_offset(&bounds, indices, variables, self)?;
                let Some(Value::Array { values, .. }) = variables.get_mut(name) else {
                    return Err(runtime_error(format!("`{name}` is not an array")));
                };
                let slot = values
                    .get_mut(offset)
                    .ok_or_else(|| runtime_error("array index out of bounds"))?;
                *slot = coerce_assignment(slot, value)?;
                Ok(())
            }
            Target::Field(base, field) => {
                let mut record = self.target_value(base, variables)?;
                let fields = match &mut record {
                    Value::Record { fields, .. } => fields,
                    Value::Object { class_name, fields } => {
                        self.require_field_access(class_name, field)?;
                        fields
                    }
                    _ => {
                        return Err(runtime_error(
                            "record or object expected before field assignment",
                        ));
                    }
                };
                let Some((_, slot)) = fields.iter_mut().find(|(name, _)| name == field) else {
                    return Err(runtime_error(format!("undefined record field `{field}`")));
                };
                *slot = coerce_assignment(slot, value)?;
                self.assign_target(base, record, variables)
            }
            Target::NestedIndex(base, indices) => {
                let mut array = self.target_value(base, variables)?;
                let Value::Array { bounds, values } = &mut array else {
                    return Err(runtime_error(
                        "array value expected before indexed assignment",
                    ));
                };
                let offset = array_offset(bounds, indices, variables, self)?;
                let slot = values
                    .get_mut(offset)
                    .ok_or_else(|| runtime_error("array index out of bounds"))?;
                *slot = coerce_assignment(slot, value)?;
                self.assign_target(base, array, variables)
            }
            Target::Dereference(pointer_expression) => {
                let target = self.pointer_target(variables, pointer_expression)?;
                self.assign_target(&target, value, variables)
            }
        }
    }

    fn freeze_target(
        &mut self,
        variables: &mut HashMap<String, Value>,
        target: Target,
    ) -> Result<Target, RuntimeError> {
        if let Some(aliased) = self.aliased_target(&target) {
            return self.freeze_target(variables, aliased);
        }
        match target {
            Target::Variable(name) => Ok(Target::Variable(name)),
            Target::Index(name, indices) => {
                let indices = indices
                    .iter()
                    .map(|index| {
                        self.evaluate(variables, index)
                            .and_then(as_integer)
                            .map(Expr::Integer)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Target::Index(name, indices))
            }
            Target::Field(base, field) => Ok(Target::Field(
                Box::new(self.freeze_target(variables, *base)?),
                field,
            )),
            Target::NestedIndex(base, indices) => {
                let base = self.freeze_target(variables, *base)?;
                let indices = indices
                    .iter()
                    .map(|index| {
                        self.evaluate(variables, index)
                            .and_then(as_integer)
                            .map(Expr::Integer)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(Target::NestedIndex(Box::new(base), indices))
            }
            Target::Dereference(pointer_expression) => {
                let target = self.pointer_target(variables, &pointer_expression)?;
                self.freeze_target(variables, target)
            }
        }
    }

    fn evaluate(
        &mut self,
        variables: &mut HashMap<String, Value>,
        expression: &Expr,
    ) -> Result<Value, RuntimeError> {
        match expression {
            Expr::Integer(n) => Ok(Value::Integer(*n)),
            Expr::Real(n) => Ok(Value::Real(*n)),
            Expr::Character(c) => Ok(Value::Character(*c)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Date(date) => Ok(Value::Date(Some(*date))),
            Expr::Variable(name) => {
                let key = normalize(name);
                if let Some(value) = self
                    .local_constants
                    .last()
                    .and_then(|constants| constants.get(&key))
                {
                    return Ok(value.clone());
                }
                if variables.contains_key(&key) || self.references.contains_key(&key) {
                    self.target_value(&Target::Variable(key), variables)
                } else {
                    self.program
                        .constants
                        .get(&key)
                        .cloned()
                        .ok_or_else(|| runtime_error(format!("undefined identifier `{name}`")))
                }
            }
            Expr::Index(name, indices) => {
                self.target_value(&Target::Index(normalize(name), indices.clone()), variables)
            }
            Expr::Field(_, _) | Expr::NestedIndex(_, _) => {
                let target = expression_target(expression)?;
                self.target_value(&target, variables)
            }
            Expr::Dereference(_) => {
                let target = expression_target(expression)?;
                self.target_value(&target, variables)
            }
            Expr::AddressOf(expression) => {
                let target = self.freeze_target(variables, expression_target(expression)?)?;
                if target_root_name(&target).is_some_and(|name| self.is_constant_name(name)) {
                    return Err(runtime_error("a constant cannot be addressed"));
                }
                let value = self.target_value(&target, variables)?;
                let target_type = value_type_signature(&value);
                let address = if let Some((address, _)) = self
                    .pointers
                    .iter()
                    .find(|(_, existing)| *existing == &target)
                {
                    *address
                } else {
                    self.pointer_sequence = self.pointer_sequence.wrapping_add(1);
                    let address = self.pointer_sequence;
                    self.pointers.insert(address, target);
                    address
                };
                Ok(Value::Pointer {
                    type_name: String::new(),
                    target_type,
                    address: Some(address),
                })
            }
            Expr::Unary(op, value) => {
                let value = self.evaluate(variables, value)?;
                match op {
                    Unary::Negate => negate(value),
                    Unary::Not => Ok(Value::Boolean(!as_bool(value)?)),
                }
            }
            Expr::Binary(left, op, right) => {
                let left = self.evaluate(variables, left)?;
                if matches!(op, Op::And) && !as_bool(left.clone())? {
                    return Ok(Value::Boolean(false));
                }
                if matches!(op, Op::Or) && as_bool(left.clone())? {
                    return Ok(Value::Boolean(true));
                }
                let right = self.evaluate(variables, right)?;
                binary(left, *op, right)
            }
            Expr::Call(name, args) => {
                if let Some(value) = builtin(name, args, variables, self)? {
                    Ok(value)
                } else {
                    self.call_routine(variables, &normalize(name), args, true)?
                        .ok_or_else(|| {
                            runtime_error(format!("function `{name}` returned no value"))
                        })
                }
            }
            Expr::MethodCall(receiver, name, args) => self
                .call_method_expression(variables, receiver, name, args, true)?
                .ok_or_else(|| runtime_error(format!("method `{name}` returned no value"))),
            Expr::New(class_name, args) => self.create_object(variables, class_name, args),
        }
    }

    fn call_method_expression(
        &mut self,
        caller: &mut HashMap<String, Value>,
        receiver: &Expr,
        method_name: &str,
        args: &[Expr],
        expect_function: bool,
    ) -> Result<Option<Value>, RuntimeError> {
        let target = expression_target(receiver)?;
        let target = self.freeze_target(caller, target)?;
        let object = self.target_value(&target, caller)?;
        let Value::Object { class_name, .. } = object else {
            return Err(runtime_error("method receiver must be an object"));
        };
        self.call_method_target(
            caller,
            target,
            &class_name,
            method_name,
            args,
            expect_function,
        )
    }

    fn call_method_target(
        &mut self,
        caller: &mut HashMap<String, Value>,
        target: Target,
        lookup_class: &str,
        method_name: &str,
        args: &[Expr],
        expect_function: bool,
    ) -> Result<Option<Value>, RuntimeError> {
        let method = self.method_from_class(lookup_class, method_name)?;
        if method.visibility == Visibility::Private
            && self.current_class.as_deref() != Some(method.owner.as_str())
        {
            return Err(runtime_error(format!(
                "private method `{method_name}` is not accessible here"
            )));
        }
        let routine = method.routine;
        if routine.return_type.is_some() != expect_function {
            return Err(runtime_error(format!(
                "method `{}` called in the wrong context",
                routine.name
            )));
        }
        if args.len() != routine.parameters.len() {
            return Err(runtime_error(format!(
                "`{}` expects {} argument(s), received {}",
                routine.name,
                routine.parameters.len(),
                args.len()
            )));
        }
        if self.recursion_depth >= self.recursion_limit {
            return Err(runtime_error("routine recursion limit exceeded"));
        }

        let object = self.target_value(&target, caller)?;
        let Value::Object {
            class_name: actual_class,
            ..
        } = object.clone()
        else {
            return Err(runtime_error("method receiver must be an object"));
        };
        let mut evaluated_args = Vec::with_capacity(args.len());
        let mut reference_bindings = Vec::new();
        for (parameter, argument) in routine.parameters.iter().zip(args) {
            let (value, reference_target) = if parameter.by_reference {
                let argument_target = expression_target(argument)?;
                let argument_target = self.freeze_target(caller, argument_target)?;
                let argument_target = self
                    .aliased_target(&argument_target)
                    .unwrap_or(argument_target);
                if target_root_name(&argument_target)
                    .is_some_and(|root| self.is_constant_name(root))
                {
                    return Err(runtime_error("BYREF argument cannot refer to a constant"));
                }
                let value = self.target_value(&argument_target, caller)?;
                (value, Some(argument_target))
            } else {
                (self.evaluate(caller, argument)?, None)
            };
            if !value_matches_type(&value, &parameter.type_name, &self.program.types) {
                return Err(runtime_error(format!(
                    "argument '{}' does not match {}",
                    parameter.name, parameter.type_name
                )));
            }
            if let Some(target) = reference_target {
                reference_bindings.push((normalize(&parameter.name), target));
            }
            evaluated_args.push(value);
        }

        let local_declarations = declared_local_initializers(&routine.body);
        let local_constants = declared_local_constants(&routine.body)
            .into_iter()
            .collect::<HashMap<_, _>>();
        let locals = local_declarations
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let parameter_names = routine
            .parameters
            .iter()
            .map(|parameter| normalize(&parameter.name))
            .collect::<Vec<_>>();
        let mut frame = caller.clone();
        self.reference_sequence = self.reference_sequence.wrapping_add(1);
        let self_storage = format!("\0cambridge_self_{}", self.reference_sequence);
        frame.insert(self_storage.clone(), object);
        let mut frame_scoped_storage = vec![self_storage.clone()];
        let mut references = self.references.clone();
        for name in parameter_names.iter().chain(locals.iter()) {
            references.remove(name);
        }
        let shadowed = parameter_names
            .iter()
            .chain(locals.iter())
            .cloned()
            .collect::<std::collections::HashSet<_>>();
        let mut visible_owner_classes = std::collections::HashSet::new();
        let mut owner_cursor = Some(method.owner.clone());
        while let Some(owner) = owner_cursor {
            visible_owner_classes.insert(owner.clone());
            owner_cursor = self
                .program
                .classes
                .get(&owner)
                .and_then(|class| class.parent.clone());
        }
        if let Some(class) = self.program.classes.get(&actual_class) {
            for field in &class.fields {
                let visible_to_method = visible_owner_classes.contains(&field.owner)
                    && (field.visibility == Visibility::Public || field.owner == method.owner);
                if visible_to_method && !shadowed.contains(&field.name) {
                    references.insert(
                        field.name.clone(),
                        Target::Field(
                            Box::new(Target::Variable(self_storage.clone())),
                            field.name.clone(),
                        ),
                    );
                }
            }
        }
        for (parameter, value) in routine.parameters.iter().zip(evaluated_args) {
            let name = normalize(&parameter.name);
            let storage = if parameter.by_reference {
                let actual = reference_bindings
                    .iter()
                    .find(|(bound_name, _)| bound_name == &name)
                    .map(|(_, target)| target.clone())
                    .ok_or_else(|| runtime_error("BYREF parameter binding was removed"))?;
                self.reference_sequence = self.reference_sequence.wrapping_add(1);
                let alias = format!("\0cambridge_reference_{}", self.reference_sequence);
                references.insert(alias.clone(), actual);
                Target::Variable(alias)
            } else {
                self.reference_sequence = self.reference_sequence.wrapping_add(1);
                let hidden = format!("\0cambridge_local_{}", self.reference_sequence);
                frame.insert(hidden.clone(), value);
                frame_scoped_storage.push(hidden.clone());
                Target::Variable(hidden)
            };
            references.insert(name, storage);
        }
        for (local, value) in &local_declarations {
            self.reference_sequence = self.reference_sequence.wrapping_add(1);
            let hidden = format!("\0cambridge_local_{}", self.reference_sequence);
            frame.insert(hidden.clone(), value.clone());
            frame_scoped_storage.push(hidden.clone());
            references.insert(local.clone(), Target::Variable(hidden));
        }

        let previous_references = std::mem::replace(&mut self.references, references);
        let previous_class = self.current_class.replace(method.owner.clone());
        let previous_self = self.current_self.replace(self_storage.clone());
        self.local_constants.push(local_constants);
        self.recursion_depth += 1;
        let outcome = self.execute_block(&mut frame, &routine.body);
        self.recursion_depth -= 1;
        self.local_constants.pop();
        self.references = previous_references;
        self.current_class = previous_class;
        self.current_self = previous_self;
        self.pointers.retain(|_, target| {
            !target_root_name(target)
                .is_some_and(|root| frame_scoped_storage.iter().any(|name| name == root))
        });
        let outcome = outcome?;
        for (key, value) in &frame {
            if caller.contains_key(key) {
                caller.insert(key.clone(), value.clone());
            }
        }
        let updated_object = frame
            .get(&self_storage)
            .cloned()
            .ok_or_else(|| runtime_error("method lost its current object"))?;
        self.assign_target(&target, updated_object, caller)?;

        match (routine.return_type, outcome) {
            (Some(type_name), Flow::Return(Some(value)))
                if value_matches_type(&value, &type_name, &self.program.types) =>
            {
                Ok(Some(value))
            }
            (Some(type_name), Flow::Return(Some(_))) => Err(runtime_error(format!(
                "return value does not match {type_name}"
            ))),
            (Some(_), _) => Err(runtime_error(format!(
                "function `{}` did not return a value",
                routine.name
            ))),
            (None, Flow::Return(_)) => Err(runtime_error(format!(
                "procedure `{}` cannot return a value",
                routine.name
            ))),
            (None, Flow::Next) => Ok(None),
        }
    }

    fn method_from_class(
        &self,
        class_name: &str,
        method_name: &str,
    ) -> Result<ClassMethod, RuntimeError> {
        let mut current = Some(normalize(class_name));
        while let Some(name) = current {
            let class = self
                .program
                .classes
                .get(&name)
                .ok_or_else(|| runtime_error(format!("unknown class `{name}`")))?;
            if let Some(method) = class.methods.get(&normalize(method_name)) {
                return Ok(method.clone());
            }
            current = class.parent.clone();
        }
        Err(runtime_error(format!(
            "class `{class_name}` has no method `{method_name}`"
        )))
    }

    fn create_object(
        &mut self,
        variables: &mut HashMap<String, Value>,
        class_name: &str,
        args: &[Expr],
    ) -> Result<Value, RuntimeError> {
        let object = self
            .program
            .types
            .get(&normalize(class_name))
            .cloned()
            .ok_or_else(|| runtime_error(format!("unknown class `{class_name}`")))?;
        if !matches!(object, Value::Object { .. }) {
            return Err(runtime_error(format!("`{class_name}` is not a class")));
        }
        self.reference_sequence = self.reference_sequence.wrapping_add(1);
        let temporary = format!("\0cambridge_new_{}", self.reference_sequence);
        variables.insert(temporary.clone(), object);
        let constructor = self.method_from_class(class_name, "NEW").is_ok();
        let construction = if constructor {
            self.call_method_target(
                variables,
                Target::Variable(temporary.clone()),
                &normalize(class_name),
                "NEW",
                args,
                false,
            )
            .map(|_| ())
        } else if args.is_empty() {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "class `{class_name}` has no constructor accepting arguments"
            )))
        };
        let result = variables
            .remove(&temporary)
            .ok_or_else(|| runtime_error("object construction lost its temporary storage"));
        construction?;
        result
    }

    fn call_routine(
        &mut self,
        caller: &mut HashMap<String, Value>,
        name: &str,
        args: &[Expr],
        expect_function: bool,
    ) -> Result<Option<Value>, RuntimeError> {
        let routine = self
            .program
            .routines
            .get(&normalize(name))
            .cloned()
            .ok_or_else(|| runtime_error(format!("undefined routine `{name}`")))?;
        if routine.return_type.is_some() != expect_function {
            return Err(runtime_error(format!(
                "routine `{}` called in the wrong context",
                routine.name
            )));
        }
        if args.len() != routine.parameters.len() {
            return Err(runtime_error(format!(
                "`{}` expects {} argument(s), received {}",
                routine.name,
                routine.parameters.len(),
                args.len()
            )));
        }
        if self.recursion_depth >= self.recursion_limit {
            return Err(runtime_error("routine recursion limit exceeded"));
        }
        let mut evaluated_args = Vec::with_capacity(args.len());
        let mut reference_bindings = Vec::new();
        for (parameter, argument) in routine.parameters.iter().zip(args) {
            let (value, reference_target) = if parameter.by_reference {
                let target = expression_target(argument)?;
                let target = self.freeze_target(caller, target)?;
                let target = self.aliased_target(&target).unwrap_or(target);
                if target_root_name(&target).is_some_and(|root| self.is_constant_name(root)) {
                    return Err(runtime_error("BYREF argument cannot refer to a constant"));
                }
                let value = self.target_value(&target, caller)?;
                (value, Some(target))
            } else {
                (self.evaluate(caller, argument)?, None)
            };
            if !value_matches_type(&value, &parameter.type_name, &self.program.types) {
                return Err(runtime_error(format!(
                    "argument '{}' does not match {}",
                    parameter.name, parameter.type_name
                )));
            }
            if let Some(target) = reference_target {
                reference_bindings.push((normalize(&parameter.name), target));
            }
            evaluated_args.push(value);
        }
        let local_declarations = declared_local_initializers(&routine.body);
        let local_constants = declared_local_constants(&routine.body)
            .into_iter()
            .collect::<HashMap<_, _>>();
        let locals = local_declarations
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>();
        let mut frame = caller.clone();
        let parameter_names: Vec<String> = routine
            .parameters
            .iter()
            .map(|parameter| normalize(&parameter.name))
            .collect();
        let mut references = self.references.clone();
        for parameter in &parameter_names {
            references.remove(parameter);
        }
        for local in &locals {
            references.remove(local);
        }
        let mut frame_scoped_storage = Vec::new();
        for (parameter, value) in routine.parameters.iter().zip(evaluated_args) {
            let name = normalize(&parameter.name);
            let storage = if parameter.by_reference {
                let actual = reference_bindings
                    .iter()
                    .find(|(bound_name, _)| bound_name == &name)
                    .map(|(_, target)| target.clone())
                    .ok_or_else(|| runtime_error("BYREF parameter binding was removed"))?;
                self.reference_sequence = self.reference_sequence.wrapping_add(1);
                let alias = format!("\0cambridge_reference_{}", self.reference_sequence);
                references.insert(alias.clone(), actual);
                Target::Variable(alias)
            } else {
                self.reference_sequence = self.reference_sequence.wrapping_add(1);
                let hidden = format!("\0cambridge_local_{}", self.reference_sequence);
                frame.insert(hidden.clone(), value);
                frame_scoped_storage.push(hidden.clone());
                Target::Variable(hidden)
            };
            references.insert(name, storage);
        }
        for (local, value) in &local_declarations {
            self.reference_sequence = self.reference_sequence.wrapping_add(1);
            let hidden = format!("\0cambridge_local_{}", self.reference_sequence);
            frame.insert(hidden.clone(), value.clone());
            frame_scoped_storage.push(hidden.clone());
            references.insert(local.clone(), Target::Variable(hidden));
        }
        let previous_references = std::mem::replace(&mut self.references, references);
        self.local_constants.push(local_constants);
        self.recursion_depth += 1;
        let outcome = self.execute_block(&mut frame, &routine.body);
        self.recursion_depth -= 1;
        self.local_constants.pop();
        self.references = previous_references;
        self.pointers.retain(|_, target| {
            !target_root_name(target)
                .is_some_and(|root| frame_scoped_storage.iter().any(|name| name == root))
        });
        let outcome = outcome?;
        for (key, value) in &frame {
            if caller.contains_key(key) {
                caller.insert(key.clone(), value.clone());
            }
        }
        match (routine.return_type, outcome) {
            (Some(ty), Flow::Return(Some(value)))
                if value_matches_type(&value, &ty, &self.program.types) =>
            {
                Ok(Some(value))
            }
            (Some(ty), Flow::Return(Some(_))) => {
                Err(runtime_error(format!("return value does not match {ty}")))
            }
            (Some(_), _) => Err(runtime_error(format!(
                "function `{}` did not return a value",
                routine.name
            ))),
            (None, Flow::Return(_)) => Err(runtime_error(format!(
                "procedure `{}` cannot return a value",
                routine.name
            ))),
            (None, Flow::Next) => Ok(None),
        }
    }
}

fn expression_target(expression: &Expr) -> Result<Target, RuntimeError> {
    match expression {
        Expr::Variable(name) => Ok(Target::Variable(normalize(name))),
        Expr::Index(name, indices) => Ok(Target::Index(normalize(name), indices.clone())),
        Expr::Field(base, field) => Ok(Target::Field(
            Box::new(expression_target(base)?),
            normalize(field),
        )),
        Expr::Dereference(pointer_expression) => {
            Ok(Target::Dereference(pointer_expression.clone()))
        }
        Expr::NestedIndex(base, indices) => Ok(Target::NestedIndex(
            Box::new(expression_target(base)?),
            indices.clone(),
        )),
        _ => Err(runtime_error(
            "target must be a variable, record field, or array element",
        )),
    }
}

fn target_root_name(target: &Target) -> Option<&str> {
    match target {
        Target::Variable(name) | Target::Index(name, _) => Some(name),
        Target::Field(base, _) | Target::NestedIndex(base, _) => target_root_name(base),
        Target::Dereference(_) => None,
    }
}

fn array_offset<H: Host>(
    bounds: &[(i64, i64)],
    indices: &[Expr],
    variables: &mut HashMap<String, Value>,
    runtime: &mut Runtime<'_, H>,
) -> Result<usize, RuntimeError> {
    if bounds.len() != indices.len() {
        return Err(runtime_error(format!(
            "array expects {} index value(s)",
            bounds.len()
        )));
    }
    let mut offset = 0_usize;
    for ((lower, upper), expression) in bounds.iter().zip(indices) {
        let index = as_integer(runtime.evaluate(variables, expression)?)?;
        if index < *lower || index > *upper {
            return Err(runtime_error(format!(
                "array index {index} is outside {lower}..{upper}"
            )));
        }
        let width = (*upper as i128 - *lower as i128 + 1) as usize;
        offset = offset
            .checked_mul(width)
            .and_then(|n| n.checked_add((index - lower) as usize))
            .ok_or_else(|| runtime_error("array index overflow"))?;
    }
    Ok(offset)
}

fn declared_local_initializers(body: &[Stmt]) -> Vec<(String, Value)> {
    fn visit(body: &[Stmt], output: &mut Vec<(String, Value)>) {
        for statement in body {
            match statement {
                Stmt::Declare(name, value) => output.push((normalize(name), value.clone())),
                Stmt::If(_, yes, no) => {
                    visit(yes, output);
                    visit(no, output);
                }
                Stmt::Case(_, arms) => {
                    for arm in arms {
                        visit(&arm.body, output);
                    }
                }
                Stmt::For(_, _, _, _, nested)
                | Stmt::While(_, nested)
                | Stmt::Repeat(nested, _) => visit(nested, output),
                _ => {}
            }
        }
    }
    let mut output = Vec::new();
    visit(body, &mut output);
    output
}

fn declared_local_constants(body: &[Stmt]) -> Vec<(String, Value)> {
    fn visit(body: &[Stmt], output: &mut Vec<(String, Value)>) {
        for statement in body {
            match statement {
                Stmt::Constant(name, value) => output.push((normalize(name), value.clone())),
                Stmt::If(_, yes, no) => {
                    visit(yes, output);
                    visit(no, output);
                }
                Stmt::Case(_, arms) => {
                    for arm in arms {
                        visit(&arm.body, output);
                    }
                }
                Stmt::For(_, _, _, _, nested)
                | Stmt::While(_, nested)
                | Stmt::Repeat(nested, _) => visit(nested, output),
                _ => {}
            }
        }
    }
    let mut output = Vec::new();
    visit(body, &mut output);
    output
}

fn builtin<H: Host>(
    name: &str,
    args: &[Expr],
    variables: &mut HashMap<String, Value>,
    runtime: &mut Runtime<'_, H>,
) -> Result<Option<Value>, RuntimeError> {
    let key = normalize(name);
    if !matches!(
        key.as_str(),
        "length" | "right" | "mid" | "lcase" | "ucase" | "int" | "rand" | "eof"
    ) {
        return Ok(None);
    }
    let values = args
        .iter()
        .map(|arg| runtime.evaluate(variables, arg))
        .collect::<Result<Vec<_>, _>>()?;
    let value = match key.as_str() {
        "eof" => {
            if values.len() != 1 {
                return Err(runtime_error("EOF expects one file identifier"));
            }
            let Value::String(path) = &values[0] else {
                return Err(runtime_error("EOF expects a STRING file identifier"));
            };
            Some(Value::Boolean(runtime.file_eof(path)?))
        }
        "length" => {
            if values.len() != 1 {
                return Err(runtime_error("LENGTH expects one argument"));
            }
            let Value::String(text) = &values[0] else {
                return Err(runtime_error("LENGTH expects a STRING"));
            };
            Some(Value::Integer(text.chars().count() as i64))
        }
        "right" => {
            if values.len() != 2 {
                return Err(runtime_error("RIGHT expects two arguments"));
            }
            let Value::String(text) = &values[0] else {
                return Err(runtime_error("RIGHT expects a STRING and an INTEGER"));
            };
            let count = as_integer(values[1].clone())?;
            let chars: Vec<char> = text.chars().collect();
            if count < 0 || count as usize > chars.len() {
                return Err(runtime_error("RIGHT length is outside the string"));
            }
            Some(Value::String(
                chars[chars.len() - count as usize..].iter().collect(),
            ))
        }
        "mid" => {
            if values.len() != 3 {
                return Err(runtime_error("MID expects three arguments"));
            }
            let Value::String(text) = &values[0] else {
                return Err(runtime_error("MID expects a STRING"));
            };
            let start = as_integer(values[1].clone())?;
            let count = as_integer(values[2].clone())?;
            let chars: Vec<char> = text.chars().collect();
            if start < 1
                || count < 0
                || start as usize - 1 > chars.len()
                || start as usize - 1 + count as usize > chars.len()
            {
                return Err(runtime_error("MID range is outside the string"));
            }
            Some(Value::String(
                chars[start as usize - 1..start as usize - 1 + count as usize]
                    .iter()
                    .collect(),
            ))
        }
        "lcase" | "ucase" => {
            if values.len() != 1 {
                return Err(runtime_error(format!(
                    "{} expects one argument",
                    key.to_ascii_uppercase()
                )));
            }
            let Value::Character(character) = values[0] else {
                return Err(runtime_error(format!(
                    "{} expects a CHAR",
                    key.to_ascii_uppercase()
                )));
            };
            let result = if key == "lcase" {
                character.to_lowercase().next().unwrap_or(character)
            } else {
                character.to_uppercase().next().unwrap_or(character)
            };
            Some(Value::Character(result))
        }
        "int" => {
            if values.len() != 1 {
                return Err(runtime_error("INT expects one argument"));
            }
            Some(Value::Integer(as_number(values[0].clone())?.trunc() as i64))
        }
        "rand" => {
            if values.len() != 1 {
                return Err(runtime_error("RAND expects one argument"));
            }
            let upper = as_number(values[0].clone())?;
            if upper <= 0.0 || !upper.is_finite() {
                return Err(runtime_error(
                    "RAND upper limit must be positive and finite",
                ));
            }
            let result = runtime.host.random(upper).map_err(runtime_error)?;
            if !result.is_finite() || result < 0.0 || result >= upper {
                return Err(runtime_error("host RAND value must be in [0, upper)"));
            }
            Some(Value::Real(result))
        }
        _ => unreachable!("recognized built-in name"),
    };
    Ok(value)
}

fn assign_target_type(expected: &Value, value: Value) -> Result<Value, RuntimeError> {
    match (expected, value) {
        (Value::Integer(_), Value::Integer(n)) => Ok(Value::Integer(n)),
        (Value::Real(_), Value::Integer(n)) => Ok(Value::Real(n as f64)),
        (Value::Real(_), Value::Real(n)) => Ok(Value::Real(n)),
        (Value::Character(_), Value::Character(c)) => Ok(Value::Character(c)),
        (Value::String(_), Value::String(s)) => Ok(Value::String(s)),
        (Value::Boolean(_), Value::Boolean(b)) => Ok(Value::Boolean(b)),
        (Value::Date(_), Value::Date(date)) => Ok(Value::Date(date)),
        (
            Value::Pointer {
                type_name: expected_type,
                target_type: expected_target,
                ..
            },
            Value::Pointer {
                target_type,
                address,
                ..
            },
        ) if expected_target == &target_type => Ok(Value::Pointer {
            type_name: expected_type.clone(),
            target_type,
            address,
        }),
        (
            Value::Set {
                type_name: expected_type,
                element_type,
                ..
            },
            Value::Set {
                type_name, values, ..
            },
        ) if expected_type == &type_name => {
            let mut coerced = Vec::new();
            for value in values {
                let value = assign_target_type(element_type, value)?;
                if !coerced.contains(&value) {
                    coerced.push(value);
                }
            }
            Ok(Value::Set {
                type_name,
                element_type: element_type.clone(),
                values: coerced,
            })
        }
        (
            Value::Enumeration {
                type_name: expected_type,
                members: expected_members,
                ..
            },
            Value::Enumeration {
                type_name,
                member,
                ordinal,
                members,
            },
        ) if expected_type == &type_name && expected_members == &members => {
            Ok(Value::Enumeration {
                type_name,
                member,
                ordinal,
                members,
            })
        }
        (
            Value::Record {
                type_name: expected_name,
                fields: expected_fields,
            },
            Value::Record { type_name, fields },
        ) => {
            if expected_name != &type_name || expected_fields.len() != fields.len() {
                return Err(runtime_error("record assignment requires matching types"));
            }
            let mut coerced = Vec::with_capacity(fields.len());
            for ((expected_name, expected), (field_name, value)) in
                expected_fields.iter().zip(fields)
            {
                if expected_name != &field_name {
                    return Err(runtime_error(
                        "record fields do not match their declared order",
                    ));
                }
                coerced.push((field_name, assign_target_type(expected, value)?));
            }
            Ok(Value::Record {
                type_name,
                fields: coerced,
            })
        }
        (
            Value::Object {
                class_name: expected_class,
                fields: expected_fields,
            },
            Value::Object { class_name, fields },
        ) => {
            if expected_class != &class_name || expected_fields.len() != fields.len() {
                return Err(runtime_error("object assignment requires matching classes"));
            }
            let mut coerced = Vec::with_capacity(fields.len());
            for ((expected_name, expected), (field_name, value)) in
                expected_fields.iter().zip(fields)
            {
                if expected_name != &field_name {
                    return Err(runtime_error(
                        "object properties do not match their declaration",
                    ));
                }
                coerced.push((field_name, assign_target_type(expected, value)?));
            }
            Ok(Value::Object {
                class_name,
                fields: coerced,
            })
        }
        (
            Value::Array {
                bounds: expected_bounds,
                values: expected_values,
            },
            Value::Array { bounds, values },
        ) => {
            if expected_bounds != &bounds || expected_values.len() != values.len() {
                return Err(runtime_error("array assignment requires matching bounds"));
            }
            let mut coerced = Vec::with_capacity(values.len());
            for (expected, value) in expected_values.iter().zip(values) {
                coerced.push(assign_target_type(expected, value)?);
            }
            Ok(Value::Array {
                bounds,
                values: coerced,
            })
        }
        _ => Err(runtime_error("assigned value has the wrong data type")),
    }
}

fn value_matches_type(value: &Value, type_name: &str, types: &HashMap<String, Value>) -> bool {
    default_value(type_name, types)
        .is_some_and(|expected| assign_target_type(&expected, value.clone()).is_ok())
}

fn as_number(value: Value) -> Result<f64, RuntimeError> {
    match value {
        Value::Integer(n) => Ok(n as f64),
        Value::Real(n) => Ok(n),
        _ => Err(runtime_error("numeric value expected")),
    }
}
fn as_integer(value: Value) -> Result<i64, RuntimeError> {
    match value {
        Value::Integer(n) => Ok(n),
        _ => Err(runtime_error("INTEGER value expected")),
    }
}
fn as_bool(value: Value) -> Result<bool, RuntimeError> {
    if let Value::Boolean(b) = value {
        Ok(b)
    } else {
        Err(runtime_error("BOOLEAN value expected"))
    }
}
fn negate(value: Value) -> Result<Value, RuntimeError> {
    match value {
        Value::Integer(n) => n
            .checked_neg()
            .map(Value::Integer)
            .ok_or_else(|| runtime_error("integer negation overflow")),
        Value::Real(n) => Ok(Value::Real(-n)),
        _ => Err(runtime_error("numeric value expected")),
    }
}

fn coerce_assignment(expected: &Value, value: Value) -> Result<Value, RuntimeError> {
    assign_target_type(expected, value)
}

fn binary(left: Value, op: Op, right: Value) -> Result<Value, RuntimeError> {
    use Op::*;
    match op {
        Add => match (&left, &right) {
            (value @ Value::Enumeration { .. }, Value::Integer(offset)) => {
                shift_enumeration(value, *offset)
            }
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_add(*b)
                .map(Value::Integer)
                .ok_or_else(|| runtime_error("integer addition overflow")),
            (Value::Integer(_) | Value::Real(_), Value::Integer(_) | Value::Real(_)) => {
                Ok(Value::Real(as_number(left)? + as_number(right)?))
            }
            _ => Err(runtime_error("`+` expects numeric values")),
        },
        Sub => match (&left, &right) {
            (value @ Value::Enumeration { .. }, Value::Integer(offset)) => offset
                .checked_neg()
                .ok_or_else(|| runtime_error("enumerated value offset overflow"))
                .and_then(|offset| shift_enumeration(value, offset)),
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_sub(*b)
                .map(Value::Integer)
                .ok_or_else(|| runtime_error("integer subtraction overflow")),
            (Value::Integer(_) | Value::Real(_), Value::Integer(_) | Value::Real(_)) => {
                Ok(Value::Real(as_number(left)? - as_number(right)?))
            }
            _ => Err(runtime_error("`-` expects numeric values")),
        },
        Mul => match (&left, &right) {
            (Value::Integer(a), Value::Integer(b)) => a
                .checked_mul(*b)
                .map(Value::Integer)
                .ok_or_else(|| runtime_error("integer multiplication overflow")),
            (Value::Integer(_) | Value::Real(_), Value::Integer(_) | Value::Real(_)) => {
                Ok(Value::Real(as_number(left)? * as_number(right)?))
            }
            _ => Err(runtime_error("`*` expects numeric values")),
        },
        Div => {
            let divisor = as_number(right)?;
            if divisor == 0.0 {
                Err(runtime_error("division by zero"))
            } else {
                Ok(Value::Real(as_number(left)? / divisor))
            }
        }
        IntDiv | Mod => {
            let a = as_integer(left)?;
            let b = as_integer(right)?;
            if b == 0 {
                return Err(runtime_error("integer division by zero"));
            }
            if op == IntDiv {
                a.checked_div(b)
                    .map(Value::Integer)
                    .ok_or_else(|| runtime_error("integer division overflow"))
            } else {
                a.checked_rem(b)
                    .map(Value::Integer)
                    .ok_or_else(|| runtime_error("integer remainder overflow"))
            }
        }
        Concat => match (left, right) {
            (Value::String(mut a), Value::String(b)) => {
                a.push_str(&b);
                Ok(Value::String(a))
            }
            _ => Err(runtime_error("`&` concatenates two STRING values")),
        },
        Eq => Ok(Value::Boolean(values_equal(&left, &right))),
        Ne => Ok(Value::Boolean(!values_equal(&left, &right))),
        Lt | Le | Gt | Ge => {
            let ordering = match (&left, &right) {
                (Value::Integer(a), Value::Integer(b)) => Some(a.cmp(b)),
                (Value::Integer(_) | Value::Real(_), Value::Integer(_) | Value::Real(_)) => {
                    as_number(left)?.partial_cmp(&as_number(right)?)
                }
                (Value::Character(a), Value::Character(b)) => Some(a.cmp(b)),
                (Value::String(a), Value::String(b)) => Some(a.cmp(b)),
                (Value::Date(Some(a)), Value::Date(Some(b))) => Some(a.cmp(b)),
                (
                    Value::Enumeration {
                        type_name: left_type,
                        ordinal: left_ordinal,
                        ..
                    },
                    Value::Enumeration {
                        type_name: right_type,
                        ordinal: right_ordinal,
                        ..
                    },
                ) if left_type == right_type => Some(left_ordinal.cmp(right_ordinal)),
                _ => None,
            }
            .ok_or_else(|| runtime_error("comparison requires compatible scalar values"))?;
            Ok(Value::Boolean(match op {
                Lt => ordering.is_lt(),
                Le => !ordering.is_gt(),
                Gt => ordering.is_gt(),
                _ => !ordering.is_lt(),
            }))
        }
        And => Ok(Value::Boolean(as_bool(left)? && as_bool(right)?)),
        Or => Ok(Value::Boolean(as_bool(left)? || as_bool(right)?)),
    }
}

fn shift_enumeration(value: &Value, offset: i64) -> Result<Value, RuntimeError> {
    let Value::Enumeration {
        type_name,
        ordinal,
        members,
        ..
    } = value
    else {
        return Err(runtime_error("enumerated value expected"));
    };
    let target = (*ordinal as i128) + (offset as i128);
    if target < 0 || target >= members.len() as i128 {
        return Err(runtime_error(
            "enumerated value offset is outside its declared range",
        ));
    }
    let ordinal = target as usize;
    Ok(Value::Enumeration {
        type_name: type_name.clone(),
        member: members[ordinal].clone(),
        ordinal,
        members: members.clone(),
    })
}

fn values_equal(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Integer(a), Value::Real(b)) | (Value::Real(b), Value::Integer(a)) => {
            (*a as f64) == *b
        }
        (
            Value::Set {
                type_name: left_type,
                values: left_values,
                ..
            },
            Value::Set {
                type_name: right_type,
                values: right_values,
                ..
            },
        ) => {
            left_type == right_type
                && left_values.len() == right_values.len()
                && left_values.iter().all(|value| right_values.contains(value))
        }
        (
            Value::Pointer {
                target_type: left_type,
                address: left_address,
                ..
            },
            Value::Pointer {
                target_type: right_type,
                address: right_address,
                ..
            },
        ) => left_type == right_type && left_address == right_address,
        _ => left == right,
    }
}

fn runtime_error(message: impl ToString) -> RuntimeError {
    RuntimeError {
        message: message.to_string(),
    }
}
