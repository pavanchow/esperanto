//! Runtime values and lexical scopes. Closures capture their defining scope by
//! reference (Rc), which is what makes both closures and recursion work.
use crate::ast::Expr;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Env = Rc<RefCell<Scope>>;

pub struct Scope {
    pub vars: HashMap<String, Value>,
    pub parent: Option<Env>,
}

#[derive(Clone)]
pub enum Value {
    Int(i64),
    Bool(bool),
    Str(String),
    Closure {
        params: Vec<String>,
        body: Rc<Expr>,
        env: Env,
    },
}

impl Scope {
    pub fn root() -> Env {
        Rc::new(RefCell::new(Scope {
            vars: HashMap::new(),
            parent: None,
        }))
    }
    pub fn child(parent: &Env) -> Env {
        Rc::new(RefCell::new(Scope {
            vars: HashMap::new(),
            parent: Some(parent.clone()),
        }))
    }
}

pub fn get(env: &Env, name: &str) -> Option<Value> {
    let s = env.borrow();
    if let Some(v) = s.vars.get(name) {
        Some(v.clone())
    } else if let Some(p) = &s.parent {
        get(p, name)
    } else {
        None
    }
}

pub fn define(env: &Env, name: &str, v: Value) {
    env.borrow_mut().vars.insert(name.to_string(), v);
}

// Manual Debug: never descend into a closure's captured scope, which for a
// recursive function contains the closure itself and would recurse forever.
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "Int({n})"),
            Value::Bool(b) => write!(f, "Bool({b})"),
            Value::Str(s) => write!(f, "Str({s:?})"),
            Value::Closure { params, .. } => write!(f, "Closure({params:?})"),
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Closure { .. } => write!(f, "<function>"),
        }
    }
}
