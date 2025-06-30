# Language Reference

This document describes the currently implemented features of the Aether
language as reflected in the compiler source. The language is intentionally
small and the reference will grow as more constructs are added.

## Lexical structure

* **Comments** – line comments start with `//` and block comments use
  `/*...*/`. Nested block comments are supported as seen in the lexer
  implementation.
* **Keywords** – `fn`, `let`, `mut` and `return` are recognised keywords.
  Additional tokens such as `if` or `while` are reserved for future use.
* **Literals** – integer, floating point, boolean, string and byte string
  literals are tokenised by the lexer.

## Types

Aether currently defines a handful of built-in types:

* `Int`
* `Float`
* `Bool`
* `Str`
* `()` – the unit type

These correspond to the variants of `Type` used throughout the resolver
and later compilation stages.

## Expressions

Expressions form the core of the language. Supported primary expressions are
identifiers and the various literals. Compound expressions include:

* Unary operations `-expr` and `!expr`.
* Binary arithmetic operators `+`, `-`, `*`, `/` and `%`.
* Comparison operators `==`, `!=`, `<`, `<=`, `>` and `>=`.
* Logical operators `&&` and `||`.
* Function calls written as `callee(arg1, arg2, ...)`.

Parentheses can be used to group expressions and the empty tuple `()` denotes
the unit value.

## Statements

The parser recognises the following forms of statements:

* **`let` bindings** – `let [mut] name = expr;` introduces a new local
  variable. The optional `mut` keyword allows the variable to be reassigned.
* **Assignment** – `name = expr;` updates a mutable binding.
* **Expression statements** – any expression followed by a semicolon.
* **Return** – `return expr;` or `return;` to return the unit value.

## Functions and modules

A source file is parsed as a module containing function definitions and optional
global `let` bindings. A function is declared with:

```text
fn name(param1: Type, param2, ...) -> ReturnType {
    // body as a list of statements
}
```

Parameter and return type annotations are optional; omitted types default to the
unit type. If a function body does not end with an explicit `return` statement
the parser automatically appends `return ();` so that every function returns a
value.

## Built-in functionality

The runtime exposes a single builtin function `print` which accepts either an
`Int` or `Str` and writes it to standard output. The compiler recognises `print`
as a special identifier and generates calls into the runtime library.

```text
fn main() {
    print(42);
    print("hello");
}
```

Further details on the runtime are described in the [Runtime](runtime.md)
chapter.
