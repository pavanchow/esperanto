//! Recursive-descent parser. One method per precedence level, top to bottom.
use crate::ast::*;
use crate::error::{Error, Result};
use crate::lexer::{Tok, Token};

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
}

pub fn parse(toks: Vec<Token>) -> Result<Program> {
    let mut p = Parser { toks, pos: 0 };
    let mut prog = Vec::new();
    while !p.check(&Tok::Eof) {
        prog.push(p.stmt()?);
    }
    Ok(prog)
}

impl Parser {
    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }
    fn line(&self) -> usize {
        self.toks[self.pos].line
    }
    fn check(&self, t: &Tok) -> bool {
        self.peek() == t
    }
    fn advance(&mut self) -> Tok {
        let t = self.toks[self.pos].tok.clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }
    fn eat(&mut self, t: &Tok) -> bool {
        if self.check(t) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect(&mut self, t: &Tok, what: &str) -> Result<()> {
        if self.eat(t) {
            Ok(())
        } else {
            Err(Error::Parse {
                msg: format!("expected {what}, found {:?}", self.peek()),
                line: self.line(),
            })
        }
    }

    fn stmt(&mut self) -> Result<Stmt> {
        if self.eat(&Tok::Let) {
            let rec = self.eat(&Tok::Rec);
            let name = self.ident("binding name")?;
            let ty = if self.eat(&Tok::Colon) {
                Some(self.parse_type()?)
            } else {
                None
            };
            self.expect(&Tok::Eq, "'='")?;
            let value = self.expr()?;
            self.expect(&Tok::Semi, "';'")?;
            Ok(Stmt::Let {
                name,
                rec,
                ty,
                value,
            })
        } else {
            let e = self.expr()?;
            self.expect(&Tok::Semi, "';'")?;
            Ok(Stmt::Expr(e))
        }
    }

    fn ident(&mut self, what: &str) -> Result<String> {
        match self.advance() {
            Tok::Ident(s) => Ok(s),
            other => Err(Error::Parse {
                msg: format!("expected {what}, found {other:?}"),
                line: self.line(),
            }),
        }
    }

    fn parse_type(&mut self) -> Result<Type> {
        if self.eat(&Tok::LParen) {
            let mut params = Vec::new();
            if !self.check(&Tok::RParen) {
                loop {
                    params.push(self.parse_type()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
            }
            self.expect(&Tok::RParen, "')'")?;
            self.expect(&Tok::Arrow, "'->' in function type")?;
            let ret = self.parse_type()?;
            return Ok(Type::Fn(params, Box::new(ret)));
        }
        let name = self.ident("type name")?;
        match name.as_str() {
            "Int" => Ok(Type::Int),
            "Bool" => Ok(Type::Bool),
            "Str" => Ok(Type::Str),
            other => Err(Error::Parse {
                msg: format!("unknown type '{other}'"),
                line: self.line(),
            }),
        }
    }

    fn expr(&mut self) -> Result<Expr> {
        self.or()
    }

    fn or(&mut self) -> Result<Expr> {
        let mut left = self.and()?;
        while self.eat(&Tok::PipePipe) {
            let right = self.and()?;
            left = Expr::Binary(BinOp::Or, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn and(&mut self) -> Result<Expr> {
        let mut left = self.equality()?;
        while self.eat(&Tok::AmpAmp) {
            let right = self.equality()?;
            left = Expr::Binary(BinOp::And, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn equality(&mut self) -> Result<Expr> {
        let mut left = self.comparison()?;
        loop {
            let op = match self.peek() {
                Tok::EqEq => BinOp::Eq,
                Tok::NotEq => BinOp::Ne,
                _ => break,
            };
            self.advance();
            let right = self.comparison()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn comparison(&mut self) -> Result<Expr> {
        let mut left = self.concat()?;
        loop {
            let op = match self.peek() {
                Tok::Lt => BinOp::Lt,
                Tok::Le => BinOp::Le,
                Tok::Gt => BinOp::Gt,
                Tok::Ge => BinOp::Ge,
                _ => break,
            };
            self.advance();
            let right = self.concat()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn concat(&mut self) -> Result<Expr> {
        let mut left = self.add()?;
        while self.eat(&Tok::PlusPlus) {
            let right = self.add()?;
            left = Expr::Binary(BinOp::Concat, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn add(&mut self) -> Result<Expr> {
        let mut left = self.mul()?;
        loop {
            let op = match self.peek() {
                Tok::Plus => BinOp::Add,
                Tok::Minus => BinOp::Sub,
                _ => break,
            };
            self.advance();
            let right = self.mul()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn mul(&mut self) -> Result<Expr> {
        let mut left = self.unary()?;
        loop {
            let op = match self.peek() {
                Tok::Star => BinOp::Mul,
                Tok::Slash => BinOp::Div,
                Tok::Percent => BinOp::Mod,
                _ => break,
            };
            self.advance();
            let right = self.unary()?;
            left = Expr::Binary(op, Box::new(left), Box::new(right));
        }
        Ok(left)
    }
    fn unary(&mut self) -> Result<Expr> {
        if self.eat(&Tok::Bang) {
            return Ok(Expr::Unary(UnOp::Not, Box::new(self.unary()?)));
        }
        if self.eat(&Tok::Minus) {
            return Ok(Expr::Unary(UnOp::Neg, Box::new(self.unary()?)));
        }
        self.call()
    }
    fn call(&mut self) -> Result<Expr> {
        let mut e = self.primary()?;
        while self.eat(&Tok::LParen) {
            let mut args = Vec::new();
            if !self.check(&Tok::RParen) {
                loop {
                    args.push(self.expr()?);
                    if !self.eat(&Tok::Comma) {
                        break;
                    }
                }
            }
            self.expect(&Tok::RParen, "')' after arguments")?;
            e = Expr::Call(Box::new(e), args);
        }
        Ok(e)
    }
    fn primary(&mut self) -> Result<Expr> {
        match self.peek().clone() {
            Tok::Int(n) => {
                self.advance();
                Ok(Expr::Int(n))
            }
            Tok::True => {
                self.advance();
                Ok(Expr::Bool(true))
            }
            Tok::False => {
                self.advance();
                Ok(Expr::Bool(false))
            }
            Tok::Str(s) => {
                self.advance();
                Ok(Expr::Str(s))
            }
            Tok::Ident(s) => {
                self.advance();
                Ok(Expr::Var(s))
            }
            Tok::LParen => {
                self.advance();
                let e = self.expr()?;
                self.expect(&Tok::RParen, "')'")?;
                Ok(e)
            }
            Tok::If => {
                self.advance();
                let cond = self.expr()?;
                self.expect(&Tok::Then, "'then'")?;
                let then = self.expr()?;
                self.expect(&Tok::Else, "'else'")?;
                let els = self.expr()?;
                Ok(Expr::If(Box::new(cond), Box::new(then), Box::new(els)))
            }
            Tok::Fn => {
                self.advance();
                self.expect(&Tok::LParen, "'(' after fn")?;
                let mut params = Vec::new();
                if !self.check(&Tok::RParen) {
                    loop {
                        let name = self.ident("parameter name")?;
                        self.expect(&Tok::Colon, "':' and a type for the parameter")?;
                        let ty = self.parse_type()?;
                        params.push((name, ty));
                        if !self.eat(&Tok::Comma) {
                            break;
                        }
                    }
                }
                self.expect(&Tok::RParen, "')'")?;
                let ret = if self.eat(&Tok::Arrow) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                self.expect(&Tok::FatArrow, "'=>' before the function body")?;
                let body = self.expr()?;
                Ok(Expr::Lambda {
                    params,
                    ret,
                    body: Box::new(body),
                })
            }
            other => Err(Error::Parse {
                msg: format!("unexpected {other:?}"),
                line: self.line(),
            }),
        }
    }
}
