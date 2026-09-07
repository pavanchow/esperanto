# Esperanto

<img src="docs/logo.svg" alt="Esperanto logo" width="96">

**A small statically-typed programming language with type inference, built from scratch in Rust.** Lexer, parser, type checker, and tree-walking interpreter, each short enough to read in one sitting. By **Pavan Nallamothu** ([`pavanchow`](https://github.com/pavanchow)).

Most "build your own language" projects are dynamically typed, because a type checker is the part tutorials skip. Esperanto keeps that part and makes it the readable centrepiece: **a program that does not type-check never runs**, and the checker infers the types you did not write down.

- **Why use it.** To read a real, static type system end to end: inference for `let` bindings and every expression, a bidirectional-style check for functions, and clear type errors reported before any code executes. It is a teaching-grade reference, not another dynamic toy.
- **What is different.** The distinguishing feature is the type checker in [`src/types.rs`](src/types.rs). Sister projects like a dynamic scripting language stop at the interpreter; Esperanto adds the pass that turns "runs until it crashes" into "rejected before it runs."

## Quickstart

```sh
cargo run -- examples/factorial.esp     # run a program
cargo run -- --check examples/fib.esp   # type-check without running
cargo run                               # REPL
cargo test                              # 20 tests: evaluation, inference, and rejections
```

## The language in one screen

```esperanto
# comments start with '#'. statements end with ';'.
let answer = 6 * 7;                 # type Int, inferred (no annotation needed)
let greeting: Str = "hello";        # optional annotation, checked against the value

# functions are first-class; parameters are annotated, the return type is optional
let add = fn(a: Int, b: Int) => a + b;

# a recursive binding needs a declared return type (so it is in scope while checked)
let rec factorial = fn(n: Int) -> Int =>
  if n <= 1 then 1 else n * factorial(n - 1);

# closures capture their scope; 'add5' remembers x
let add5 = (fn(x: Int) -> (Int) -> Int => fn(y: Int) -> Int => x + y)(5);

print(factorial(10));               # 3628800
print(greeting ++ ", " ++ str(add5(add(1, 1))));   # hello, 7
```

**Types:** `Int`, `Bool`, `Str`, and function types like `(Int, Int) -> Int`.
**Operators:** `+ - * / %`, `++` (string concat), `== != < <= > >=`, `&& || !`.
**Forms:** `let` / `let rec`, `fn(...) -> T => body`, `if c then a else b`, calls `f(x)`.
**Builtins:** `print(x)` (returns `x`, so it composes), `len(Str) -> Int`, `str(Int|Bool) -> Str`.

## What the checker catches (before running)

```
true + 1;                       ->  type error: arithmetic operator: expected Int, found Bool
if true then 1 else "x";        ->  type error: if branches disagree: then is Int, else is Str
let f = fn(n: Int) => n; f(true);  -> type error: argument 1: expected Int, found Bool
let x = 5; x(3);                ->  type error: cannot call a value of type Int
```

## How it works

Four stages, one file each, wired together in [`src/lib.rs`](src/lib.rs):

1. [`lexer.rs`](src/lexer.rs) — source text to tokens, one pass, tracks line numbers.
2. [`parser.rs`](src/parser.rs) — recursive descent, one method per precedence level.
3. [`types.rs`](src/types.rs) — infers and checks types over the AST; rejects ill-typed programs.
4. [`interpreter.rs`](src/interpreter.rs) — walks the AST; trusts types, guards only what types cannot (division by zero).

See [DESIGN.md](DESIGN.md) for the type system and the choices behind it.

## License

MIT.
