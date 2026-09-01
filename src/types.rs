//! Static type checker with inference. Every program is fully checked before it
//! runs, so a type error is reported without executing a single expression.
//!
//! Inference is deliberately small and readable: let bindings and every
//! expression infer their own type, function parameters carry annotations, and a
//! recursive binding needs a return-type annotation so the function is in scope
//! (with a known type) while its own body is checked.
use crate::ast::*;
use crate::error::{Error, Result};
use std::collections::HashMap;

pub struct Checker {
    scopes: Vec<HashMap<String, Type>>,
}

const BUILTINS: &[&str] = &["print", "len", "str"];

pub fn check(prog: &Program) -> Result<()> {
    let mut c = Checker {
        scopes: vec![HashMap::new()],
    };
    for stmt in prog {
        c.stmt(stmt)?;
    }
    Ok(())
}

impl Checker {
    fn lookup(&self, name: &str) -> Option<Type> {
        for scope in self.scopes.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t.clone());
            }
        }
        None
    }
    fn define(&mut self, name: &str, t: Type) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), t);
    }

    fn stmt(&mut self, s: &Stmt) -> Result<()> {
        match s {
            Stmt::Let {
                name,
                rec,
                ty,
                value,
            } => {
                if *rec {
                    let fn_ty = match value {
                        Expr::Lambda { params, ret: Some(ret), .. } => Type::Fn(
                            params.iter().map(|(_, t)| t.clone()).collect(),
                            Box::new(ret.clone()),
                        ),
                        _ => {
                            return Err(Error::Type(format!(
                                "recursive binding '{name}' must be a function with a declared return type"
                            )))
                        }
                    };
                    self.define(name, fn_ty.clone());
                    let inferred = self.infer(value)?;
                    if inferred != fn_ty {
                        return Err(Error::Type(format!(
                            "recursive binding '{name}' declared {fn_ty} but body has type {inferred}"
                        )));
                    }
                    if let Some(ann) = ty {
                        self.unify(&inferred, ann, &format!("binding '{name}'"))?;
                    }
                } else {
                    let inferred = self.infer(value)?;
                    if let Some(ann) = ty {
                        self.unify(&inferred, ann, &format!("binding '{name}'"))?;
                        self.define(name, ann.clone());
                    } else {
                        self.define(name, inferred);
                    }
                }
                Ok(())
            }
            Stmt::Expr(e) => {
                self.infer(e)?;
                Ok(())
            }
        }
    }

    fn unify(&self, actual: &Type, expected: &Type, ctx: &str) -> Result<()> {
        if actual == expected {
            Ok(())
        } else {
            Err(Error::Type(format!(
                "{ctx}: expected {expected}, found {actual}"
            )))
        }
    }

    fn infer(&mut self, e: &Expr) -> Result<Type> {
        match e {
            Expr::Int(_) => Ok(Type::Int),
            Expr::Bool(_) => Ok(Type::Bool),
            Expr::Str(_) => Ok(Type::Str),
            Expr::Var(name) => self
                .lookup(name)
                .ok_or_else(|| Error::Type(format!("undefined variable '{name}'"))),
            Expr::Unary(op, x) => {
                let t = self.infer(x)?;
                match op {
                    UnOp::Not => {
                        self.unify(&t, &Type::Bool, "operator '!'")?;
                        Ok(Type::Bool)
                    }
                    UnOp::Neg => {
                        self.unify(&t, &Type::Int, "unary '-'")?;
                        Ok(Type::Int)
                    }
                }
            }
            Expr::Binary(op, l, r) => {
                let lt = self.infer(l)?;
                let rt = self.infer(r)?;
                use BinOp::*;
                match op {
                    Add | Sub | Mul | Div | Mod => {
                        self.unify(&lt, &Type::Int, "arithmetic operator")?;
                        self.unify(&rt, &Type::Int, "arithmetic operator")?;
                        Ok(Type::Int)
                    }
                    Concat => {
                        self.unify(&lt, &Type::Str, "operator '++'")?;
                        self.unify(&rt, &Type::Str, "operator '++'")?;
                        Ok(Type::Str)
                    }
                    Lt | Le | Gt | Ge => {
                        self.unify(&lt, &Type::Int, "comparison operator")?;
                        self.unify(&rt, &Type::Int, "comparison operator")?;
                        Ok(Type::Bool)
                    }
                    Eq | Ne => {
                        if matches!(lt, Type::Fn(..)) {
                            return Err(Error::Type("cannot compare functions".into()));
                        }
                        self.unify(&rt, &lt, "operands of '==' / '!='")?;
                        Ok(Type::Bool)
                    }
                    And | Or => {
                        self.unify(&lt, &Type::Bool, "boolean operator")?;
                        self.unify(&rt, &Type::Bool, "boolean operator")?;
                        Ok(Type::Bool)
                    }
                }
            }
            Expr::If(c, t, e) => {
                let ct = self.infer(c)?;
                self.unify(&ct, &Type::Bool, "if condition")?;
                let tt = self.infer(t)?;
                let et = self.infer(e)?;
                if tt != et {
                    return Err(Error::Type(format!(
                        "if branches disagree: then is {tt}, else is {et}"
                    )));
                }
                Ok(tt)
            }
            Expr::Lambda { params, ret, body } => {
                self.scopes.push(HashMap::new());
                for (n, t) in params {
                    self.define(n, t.clone());
                }
                let body_ty = self.infer(body)?;
                self.scopes.pop();
                if let Some(ret) = ret {
                    self.unify(&body_ty, ret, "function body")?;
                }
                let param_tys = params.iter().map(|(_, t)| t.clone()).collect();
                Ok(Type::Fn(
                    param_tys,
                    Box::new(ret.clone().unwrap_or(body_ty)),
                ))
            }
            Expr::Call(callee, args) => {
                // builtins: only when the name is not shadowed by a real binding
                if let Expr::Var(name) = callee.as_ref() {
                    if BUILTINS.contains(&name.as_str()) && self.lookup(name).is_none() {
                        return self.builtin(name, args);
                    }
                }
                let ct = self.infer(callee)?;
                let (ps, r) = match ct {
                    Type::Fn(ps, r) => (ps, r),
                    other => {
                        return Err(Error::Type(format!("cannot call a value of type {other}")))
                    }
                };
                if args.len() != ps.len() {
                    return Err(Error::Type(format!(
                        "function expects {} argument(s), got {}",
                        ps.len(),
                        args.len()
                    )));
                }
                for (i, a) in args.iter().enumerate() {
                    let at = self.infer(a)?;
                    self.unify(&at, &ps[i], &format!("argument {}", i + 1))?;
                }
                Ok(*r)
            }
        }
    }

    fn builtin(&mut self, name: &str, args: &[Expr]) -> Result<Type> {
        let one = |args: &[Expr]| -> Result<()> {
            if args.len() != 1 {
                Err(Error::Type(format!("{name} takes exactly one argument")))
            } else {
                Ok(())
            }
        };
        match name {
            "print" => {
                one(args)?;
                let t = self.infer(&args[0])?;
                if matches!(t, Type::Fn(..)) {
                    return Err(Error::Type("cannot print a function".into()));
                }
                Ok(t) // print returns its argument, so it composes
            }
            "len" => {
                one(args)?;
                let t = self.infer(&args[0])?;
                self.unify(&t, &Type::Str, "len")?;
                Ok(Type::Int)
            }
            "str" => {
                one(args)?;
                let t = self.infer(&args[0])?;
                match t {
                    Type::Int | Type::Bool => Ok(Type::Str),
                    other => Err(Error::Type(format!(
                        "str expects Int or Bool, found {other}"
                    ))),
                }
            }
            _ => unreachable!(),
        }
    }
}
