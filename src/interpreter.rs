//! Tree-walking interpreter. Runs only after the type checker has passed, so it
//! trusts types and guards just the things types cannot catch (division by zero).
use crate::ast::*;
use crate::error::{Error, Result};
use crate::value::{define, get, Env, Scope, Value};
use std::rc::Rc;

const BUILTINS: &[&str] = &["print", "len", "str"];

pub fn run_program(prog: &Program) -> Result<Value> {
    let env = Scope::root();
    let mut last = Value::Int(0);
    for s in prog {
        match s {
            Stmt::Let { name, value, .. } => {
                let v = eval(value, &env)?;
                define(&env, name, v.clone());
                last = v;
            }
            Stmt::Expr(e) => last = eval(e, &env)?,
        }
    }
    Ok(last)
}

fn eval(e: &Expr, env: &Env) -> Result<Value> {
    match e {
        Expr::Int(n) => Ok(Value::Int(*n)),
        Expr::Bool(b) => Ok(Value::Bool(*b)),
        Expr::Str(s) => Ok(Value::Str(s.clone())),
        Expr::Var(name) => {
            get(env, name).ok_or_else(|| Error::Runtime(format!("undefined variable '{name}'")))
        }
        Expr::Unary(op, x) => {
            let v = eval(x, env)?;
            match (op, v) {
                (UnOp::Neg, Value::Int(n)) => Ok(Value::Int(-n)),
                (UnOp::Not, Value::Bool(b)) => Ok(Value::Bool(!b)),
                _ => Err(Error::Runtime("bad operand for unary operator".into())),
            }
        }
        Expr::Binary(op, l, r) => {
            let a = eval(l, env)?;
            let b = eval(r, env)?;
            eval_binary(*op, a, b)
        }
        Expr::If(c, t, e) => match eval(c, env)? {
            Value::Bool(true) => eval(t, env),
            Value::Bool(false) => eval(e, env),
            _ => Err(Error::Runtime("if condition is not a boolean".into())),
        },
        Expr::Lambda { params, body, .. } => Ok(Value::Closure {
            params: params.iter().map(|(n, _)| n.clone()).collect(),
            body: Rc::new(body.as_ref().clone()),
            env: env.clone(),
        }),
        Expr::Call(callee, args) => {
            if let Expr::Var(name) = callee.as_ref() {
                if BUILTINS.contains(&name.as_str()) && get(env, name).is_none() {
                    let vals: Result<Vec<Value>> = args.iter().map(|a| eval(a, env)).collect();
                    return call_builtin(name, vals?);
                }
            }
            let callee_v = eval(callee, env)?;
            let (params, body, closure_env) = match callee_v {
                Value::Closure { params, body, env } => (params, body, env),
                _ => return Err(Error::Runtime("attempt to call a non-function".into())),
            };
            let call_env = Scope::child(&closure_env);
            for (p, a) in params.iter().zip(args.iter()) {
                let v = eval(a, env)?;
                define(&call_env, p, v);
            }
            eval(&body, &call_env)
        }
    }
}

fn eval_binary(op: BinOp, a: Value, b: Value) -> Result<Value> {
    use BinOp::*;
    use Value::*;
    match (op, a, b) {
        (Add, Int(x), Int(y)) => Ok(Int(x + y)),
        (Sub, Int(x), Int(y)) => Ok(Int(x - y)),
        (Mul, Int(x), Int(y)) => Ok(Int(x * y)),
        (Div, Int(_), Int(0)) => Err(Error::Runtime("division by zero".into())),
        (Div, Int(x), Int(y)) => Ok(Int(x / y)),
        (Mod, Int(_), Int(0)) => Err(Error::Runtime("modulo by zero".into())),
        (Mod, Int(x), Int(y)) => Ok(Int(x % y)),
        (Concat, Str(x), Str(y)) => Ok(Str(x + &y)),
        (Lt, Int(x), Int(y)) => Ok(Bool(x < y)),
        (Le, Int(x), Int(y)) => Ok(Bool(x <= y)),
        (Gt, Int(x), Int(y)) => Ok(Bool(x > y)),
        (Ge, Int(x), Int(y)) => Ok(Bool(x >= y)),
        (And, Bool(x), Bool(y)) => Ok(Bool(x && y)),
        (Or, Bool(x), Bool(y)) => Ok(Bool(x || y)),
        (Eq, x, y) => Ok(Bool(values_equal(&x, &y))),
        (Ne, x, y) => Ok(Bool(!values_equal(&x, &y))),
        _ => Err(Error::Runtime("bad operands for binary operator".into())),
    }
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Int(x), Value::Int(y)) => x == y,
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Str(x), Value::Str(y)) => x == y,
        _ => false,
    }
}

fn call_builtin(name: &str, args: Vec<Value>) -> Result<Value> {
    match name {
        "print" => {
            println!("{}", args[0]);
            Ok(args.into_iter().next().unwrap())
        }
        "len" => match &args[0] {
            Value::Str(s) => Ok(Value::Int(s.chars().count() as i64)),
            _ => Err(Error::Runtime("len expects a string".into())),
        },
        "str" => Ok(Value::Str(args[0].to_string())),
        _ => Err(Error::Runtime(format!("unknown builtin '{name}'"))),
    }
}
