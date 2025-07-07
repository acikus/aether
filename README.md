# Aether

Aether is an experimental language compiler written in Rust. The workspace
includes a core library and a command line interface (`aethc`).

## Prerequisites

Aether targets LLVM for code generation and uses `clang` to link the produced
bitcode. A matching version of LLVM and clang must be installed along with the
LLVM development headers. The compiler also uses Rust's upcoming 2024 edition,
so a recent nightly toolchain is required.

To set up the Rust toolchain with [rustup](https://rustup.rs):

```bash
rustup install nightly
rustup override set nightly  # inside this repository
```

### Installing LLVM

Install the LLVM packages that correspond to your clang version. Below are
common commands for popular platforms:

- **Debian/Ubuntu**: `sudo apt install clang llvm-dev libclang-dev`
- **Fedora**: `sudo dnf install clang llvm-devel clang-devel`
- **Arch Linux**: `sudo pacman -S clang llvm`
- **macOS (Homebrew)**: `brew install llvm` (ensure `$(brew --prefix llvm)/bin`
  is in `PATH`)
- **Windows**: download the installer from the [LLVM website](https://releases.llvm.org/)
  or install via Chocolatey: `choco install llvm`

## Building

```bash
cargo build --release
```

## Usage Example

```bash
cargo run -p aethc_cli -- parse path/to/file.aeth
```


## Routing Service

See [`routing_service/`](routing_service/) for a simple Flask application that proxies requests to a Valhalla server.
