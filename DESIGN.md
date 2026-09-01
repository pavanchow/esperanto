# Design

Esperanto is a statically-typed language whose whole implementation is meant to be read. The design choices all serve that: small surface, one concept per file, and a type checker that is the interesting part rather than the hidden part.

## Pipeline

```
source ──lex──> tokens ──parse──> AST ──check──> AST (well-typed) ──eval──> value
```

`run(src)` (in `lib.rs`) runs all four; `typecheck(src)` stops after stage three. Because checking is a separate, total pass, a type error is reported without executing anything.

## The type system

The goal is a real static check that still feels light to write. The rules:

- **Literals and operators** have fixed types. Arithmetic is `Int -> Int`, comparison is `Int -> Bool`, `++` is `Str -> Str`, `&& || !` are `Bool`, and `== / !=` compare two values of the same non-function type.
- **`let` inference.** A binding takes the inferred type of its value. An optional annotation is checked against it. No annotation is needed for ordinary values, so most code carries no types.
- **Functions** annotate their parameters; the return type is optional and inferred from the body when omitted. A function's type is `(P1, ..., Pn) -> R`.
- **`if`** requires a `Bool` condition and both branches to share a type, which becomes the type of the expression.
- **Recursion** is the one place an annotation is mandatory. To check the body of `let rec f = fn(...) -> R => ...`, `f` must already be in scope with a known type. We get that type from the parameter and return annotations before descending into the body. This is why a recursive binding must be a function with a declared return type; the checker says so explicitly rather than guessing.

This is deliberately not full Hindley-Milner. There is no unification variable and no let-generalization. The trade is honesty about scope: you can hold the entire checker (`types.rs`) in your head, and it still infers the common cases and rejects the real errors.

## The interpreter

A tree-walking evaluator over the same AST. It runs only after checking, so it assumes well-typedness and does not re-check types at runtime. The one thing types cannot rule out is division or modulo by zero, so those are the only runtime errors the evaluator raises for arithmetic.

Scopes are `Rc<RefCell<Scope>>` with a parent link. A closure captures the scope it was defined in by reference, which is what makes both closures and recursion work: a recursive function is defined into the same scope its closure captured, so the name is visible when the body runs.

## Non-goals

No modules, no mutation, no loops (recursion is the iteration construct), no user-defined types or generics. Each of those is a real feature and each would blur the "read it in one sitting" line. The point of Esperanto is the checked core, not the size of the language.
