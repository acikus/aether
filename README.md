# Aether

Aether is an experimental compiler written in Rust.  The project currently
implements the early stages of a language front‑end: a lexer, a recursive
parser, name resolution and a small borrow checker.  It is intended as a
playground for exploring compiler construction techniques and a typed language
with borrow‑checked semantics.

## Building

The workspace uses the nightly Rust toolchain with the 2024 edition.  Build all
crates with:

```bash
cargo build
```

Run the test suite with:

```bash
cargo test
```

## Running

The `aethc_cli` crate provides a simple command line interface that parses a
source file and prints the resulting abstract syntax tree.  Invoke it with:

```bash
cargo run -p aethc_cli -- parse path/to/file.aeth
```

`parse` is currently the only subcommand and is useful for inspecting how the
parser understands your source code.

## Goals

Aether is in a very early stage.  The near‑term goals are:

- A complete lexer supporting nested comments and string escapes.
- A recursive descent parser for functions and `let` bindings.
- Basic name resolution and type inference.
- An elementary borrow checker used in the tests.

The project is not yet a full language implementation, but a foundation for
experimenting with these ideas.

See the `LICENSE` file in this repository for license information.
