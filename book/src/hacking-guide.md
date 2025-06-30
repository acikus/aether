# Hacking Guide

This guide describes how to build and contribute to the Aether compiler. It summarizes the repository layout and basic workflows.

## Repository layout

- **`aethc_core`** – core compiler library. Modules include lexer, parser, name resolver, borrow checker and code generation.
- **`aethc_cli`** – command line frontend. `build.rs` compiles `runtime.c` and links it when producing executables.
- **`book/`** – documentation built with [mdBook](https://rust-lang.github.io/mdBook/).
- **`samples/`** – small example programs such as [`hello.ae`](../samples/hello.ae).

## Building

Aether requires a nightly Rust toolchain and LLVM with clang. After installing the prerequisites from the [README](../../README.md), build the workspace with:

```bash
cargo build --release
```

## Running the compiler

The CLI exposes several subcommands:

- `parse FILE [--emit-hir]` – print the AST and optionally the HIR.
- `check FILE` – run all front-end checks.
- `build FILE [-o OUTPUT] [--emit hir|mir|llvm]` – produce an executable via LLVM and clang.

For example, to parse the sample program:

```bash
cargo run -p aethc_cli -- parse samples/hello.ae
```

## Tests

Run the full test suite with:

```bash
cargo test
```

Both crates contain unit tests and integration tests under `aethc_core/tests` and `aethc_cli/tests`.
