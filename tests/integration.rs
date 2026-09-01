use esperanto::{run, typecheck, Error, Value};

fn int(src: &str) -> i64 {
    match run(src).expect("should run") {
        Value::Int(n) => n,
        v => panic!("expected Int, got {v}"),
    }
}
fn boolean(src: &str) -> bool {
    match run(src).expect("should run") {
        Value::Bool(b) => b,
        v => panic!("expected Bool, got {v}"),
    }
}
fn string(src: &str) -> String {
    match run(src).expect("should run") {
        Value::Str(s) => s,
        v => panic!("expected Str, got {v}"),
    }
}
fn type_error(src: &str) -> String {
    match typecheck(src) {
        Err(Error::Type(m)) => m,
        Err(other) => panic!("expected Type error, got {other}"),
        Ok(()) => panic!("expected a type error, but it type-checked: {src}"),
    }
}

// ---- evaluation ----

#[test]
fn arithmetic_and_precedence() {
    assert_eq!(int("1 + 2 * 3;"), 7);
    assert_eq!(int("(1 + 2) * 3;"), 9);
    assert_eq!(int("10 - 2 - 3;"), 5); // left assoc
    assert_eq!(int("17 % 5;"), 2);
    assert_eq!(int("-4 + 10;"), 6);
}

#[test]
fn let_inference_and_use() {
    assert_eq!(int("let x = 2 + 3; let y = x * x; y;"), 25);
    assert_eq!(string("let g = \"hi\"; g ++ \" there\";"), "hi there");
}

#[test]
fn booleans_and_comparison() {
    assert!(boolean("3 < 5;"));
    assert!(boolean("(3 < 5) && (2 == 2);"));
    assert!(!boolean("(3 > 5) || false;"));
    assert!(boolean("!(1 == 2);"));
    assert!(boolean("\"a\" == \"a\";"));
    assert!(!boolean("\"a\" == \"b\";"));
}

#[test]
fn if_expression() {
    assert_eq!(int("if 2 < 3 then 10 else 20;"), 10);
    assert_eq!(int("if false then 10 else 20;"), 20);
    assert_eq!(int("1 + (if true then 2 else 3);"), 3);
}

#[test]
fn functions_and_closures() {
    assert_eq!(int("let add = fn(a: Int, b: Int) => a + b; add(4, 5);"), 9);
    // curried closure: add(5) returns a function
    let src = "let add = fn(x: Int) -> (Int) -> Int => fn(y: Int) -> Int => x + y; \
               let add5 = add(5); add5(10);";
    assert_eq!(int(src), 15);
}

#[test]
fn recursion() {
    let fact = "let rec f = fn(n: Int) -> Int => if n <= 1 then 1 else n * f(n - 1); f(5);";
    assert_eq!(int(fact), 120);
    let fib =
        "let rec fib = fn(n: Int) -> Int => if n < 2 then n else fib(n-1) + fib(n-2); fib(10);";
    assert_eq!(int(fib), 55);
}

#[test]
fn strings_and_builtins() {
    assert_eq!(int("len(\"hello\");"), 5);
    assert_eq!(string("\"n=\" ++ str(6 * 7);"), "n=42");
    assert_eq!(string("str(true);"), "true");
}

#[test]
fn print_returns_its_argument() {
    // print(x) has the type and value of x, so it composes inside arithmetic
    assert_eq!(int("1 + print(41);"), 42);
}

// ---- type checking: rejections ----

#[test]
fn rejects_undefined_variable() {
    assert!(type_error("x + 1;").contains("undefined variable"));
}

#[test]
fn rejects_arithmetic_on_bool() {
    type_error("true + 1;");
}

#[test]
fn rejects_concat_on_int() {
    type_error("1 ++ 2;");
}

#[test]
fn rejects_if_branch_mismatch() {
    assert!(type_error("if true then 1 else \"x\";").contains("branches disagree"));
}

#[test]
fn rejects_calling_non_function() {
    assert!(type_error("let x = 5; x(3);").contains("cannot call"));
}

#[test]
fn rejects_wrong_arg_type() {
    type_error("let f = fn(n: Int) => n + 1; f(true);");
}

#[test]
fn rejects_wrong_arg_count() {
    assert!(type_error("let f = fn(a: Int, b: Int) => a + b; f(1);").contains("argument"));
}

#[test]
fn rejects_return_annotation_mismatch() {
    type_error("let f = fn(n: Int) -> Bool => n + 1;");
}

#[test]
fn rejects_rec_without_return_annotation() {
    assert!(type_error("let rec f = fn(n: Int) => n; f(1);").contains("declared return type"));
}

#[test]
fn rejects_comparing_functions() {
    assert!(type_error("let f = fn(n: Int) => n; f == f;").contains("compare functions"));
}

#[test]
fn well_typed_program_does_not_error() {
    assert!(
        typecheck("let rec f = fn(n: Int) -> Int => if n <= 0 then 0 else n + f(n-1); f(4);")
            .is_ok()
    );
}

// ---- runtime guard types cannot catch ----

#[test]
fn division_by_zero_is_a_runtime_error() {
    match run("10 / 0;") {
        Err(Error::Runtime(m)) => assert!(m.contains("division by zero")),
        other => panic!("expected runtime error, got {other:?}"),
    }
}
