//! Source text to a flat token stream. Hand-written, one pass, tracks line numbers.
use crate::error::{Error, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Int(i64),
    Str(String),
    Ident(String),
    // keywords
    Let,
    Rec,
    Fn,
    If,
    Then,
    Else,
    True,
    False,
    // operators and punctuation
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusPlus,
    EqEq,
    NotEq,
    Lt,
    Le,
    Gt,
    Ge,
    AmpAmp,
    PipePipe,
    Bang,
    Eq,
    Arrow,    // ->
    FatArrow, // =>
    Colon,
    Semi,
    Comma,
    LParen,
    RParen,
    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
}

pub fn lex(src: &str) -> Result<Vec<Token>> {
    let chars: Vec<char> = src.chars().collect();
    let mut i = 0;
    let mut line = 1;
    let mut out = Vec::new();
    let push = |out: &mut Vec<Token>, tok: Tok, line: usize| out.push(Token { tok, line });

    while i < chars.len() {
        let c = chars[i];
        match c {
            '\n' => {
                line += 1;
                i += 1;
            }
            c if c.is_whitespace() => i += 1,
            '#' => {
                while i < chars.len() && chars[i] != '\n' {
                    i += 1;
                }
            }
            '"' => {
                i += 1;
                let mut s = String::new();
                loop {
                    if i >= chars.len() {
                        return Err(Error::Lex {
                            msg: "unterminated string".into(),
                            line,
                        });
                    }
                    let ch = chars[i];
                    if ch == '"' {
                        i += 1;
                        break;
                    }
                    if ch == '\\' && i + 1 < chars.len() {
                        i += 1;
                        s.push(match chars[i] {
                            'n' => '\n',
                            't' => '\t',
                            '\\' => '\\',
                            '"' => '"',
                            other => other,
                        });
                        i += 1;
                    } else {
                        if ch == '\n' {
                            line += 1;
                        }
                        s.push(ch);
                        i += 1;
                    }
                }
                push(&mut out, Tok::Str(s), line);
            }
            c if c.is_ascii_digit() => {
                let mut n = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    n.push(chars[i]);
                    i += 1;
                }
                let v: i64 = n.parse().map_err(|_| Error::Lex {
                    msg: format!("bad integer '{n}'"),
                    line,
                })?;
                push(&mut out, Tok::Int(v), line);
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut s = String::new();
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    s.push(chars[i]);
                    i += 1;
                }
                let tok = match s.as_str() {
                    "let" => Tok::Let,
                    "rec" => Tok::Rec,
                    "fn" => Tok::Fn,
                    "if" => Tok::If,
                    "then" => Tok::Then,
                    "else" => Tok::Else,
                    "true" => Tok::True,
                    "false" => Tok::False,
                    _ => Tok::Ident(s),
                };
                push(&mut out, tok, line);
            }
            _ => {
                // two-char operators first
                let two = if i + 1 < chars.len() {
                    Some((chars[i], chars[i + 1]))
                } else {
                    None
                };
                let (tok, len) = match two {
                    Some(('+', '+')) => (Tok::PlusPlus, 2),
                    Some(('=', '=')) => (Tok::EqEq, 2),
                    Some(('!', '=')) => (Tok::NotEq, 2),
                    Some(('<', '=')) => (Tok::Le, 2),
                    Some(('>', '=')) => (Tok::Ge, 2),
                    Some(('&', '&')) => (Tok::AmpAmp, 2),
                    Some(('|', '|')) => (Tok::PipePipe, 2),
                    Some(('-', '>')) => (Tok::Arrow, 2),
                    Some(('=', '>')) => (Tok::FatArrow, 2),
                    _ => {
                        let t = match c {
                            '+' => Tok::Plus,
                            '-' => Tok::Minus,
                            '*' => Tok::Star,
                            '/' => Tok::Slash,
                            '%' => Tok::Percent,
                            '<' => Tok::Lt,
                            '>' => Tok::Gt,
                            '!' => Tok::Bang,
                            '=' => Tok::Eq,
                            ':' => Tok::Colon,
                            ';' => Tok::Semi,
                            ',' => Tok::Comma,
                            '(' => Tok::LParen,
                            ')' => Tok::RParen,
                            other => {
                                return Err(Error::Lex {
                                    msg: format!("unexpected character '{other}'"),
                                    line,
                                })
                            }
                        };
                        (t, 1)
                    }
                };
                push(&mut out, tok, line);
                i += len;
            }
        }
    }
    out.push(Token {
        tok: Tok::Eof,
        line,
    });
    Ok(out)
}
