use aethc_core::{self, lexer::Span};
use ariadne::{Label, Report, ReportKind, Source};
use clap::{Parser, Subcommand};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

#[derive(Parser)]
#[command(name = "aethc", version)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Dump AST (and optionally HIR) for a source file.
    Parse {
        file: PathBuf,
        #[arg(long)]
        emit_hir: bool,
    },
    /// Run lexer, parser, resolver, borrow, type-infer checks.
    Check { file: PathBuf },
    /// Build executable: full pipeline → LLVM → clang/ld
    Build {
        file: PathBuf,
        #[arg(short, long, default_value = "a.out")]
        output: PathBuf,
        #[arg(long, value_parser = ["hir", "mir", "llvm"])]
        emit: Option<String>,
    },
}

fn main() -> std::process::ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Parse { file, emit_hir } => {
            let src = fs::read_to_string(&file).expect("read");
            let (ast, lex_errs) = aethc_core::parse(&src);
            report_errors(&lex_errs, &src);
            println!("{:#?}", ast);
            if emit_hir {
                let (hir, _res_errs) = aethc_core::lower_to_hir(&ast, &src);
                println!("{:#?}", hir);
            }
            exit_code(lex_errs.is_empty())
        }
        Cmd::Check { file } => {
            let ok = run_full_frontend(&file, None).is_ok();
            exit_code(ok)
        }
        Cmd::Build { file, output, emit } => match run_full_frontend(&file, emit.as_deref()) {
            Ok(mut llvm_module) => {
                let bitcode_path = output.with_extension("bc");
                llvm_module.module.write_bitcode_to_path(&bitcode_path);
                link_with_clang(&bitcode_path, &output);
                println!("built {}", output.display());
                exit_code(true)
            }
            Err(_) => exit_code(false),
        },
    }
}

fn report_errors<E>(errs: &[E], src: &str)
where
    E: SpannedError,
{
    for e in errs {
        let span = e.span();
        let msg = e.msg();
        Report::build(ReportKind::Error, (), span.start)
            .with_message(&msg)
            .with_label(Label::new(span.start..span.end).with_message(msg))
            .finish()
            .print(Source::from(src))
            .unwrap();
    }
}

type LlvmModule<'ctx> = aethc_core::codegen::LlvmCtx<'ctx>;

fn run_full_frontend(path: &PathBuf, emit: Option<&str>) -> Result<LlvmModule<'static>, ()> {
    let src = fs::read_to_string(path).expect("read");
    let (ast, lex_errs) = aethc_core::parse(&src);
    report_errors(&lex_errs, &src);
    if !lex_errs.is_empty() {
        return Err(());
    }

    let (hir, res_errs) = aethc_core::lower_to_hir(&ast, &src);
    report_errors(&res_errs, &src);
    if !res_errs.is_empty() {
        return Err(());
    }

    if let Some("hir") = emit {
        println!("{:#?}", hir);
    }

    if let Some("mir") | Some("llvm") | Some(_) = emit {
        // continue to MIR/LLVM
    }

    // Lower to MIR
    let mir = match &hir.items[0] {
        aethc_core::hir::Item::Fn(f) => aethc_core::mir::lower_fn(f),
        _ => return Err(()),
    };

    if let Some("mir") = emit {
        println!("{:#?}", mir);
    }

    // Codegen
    let mut llcx = aethc_core::codegen::LlvmCtx::new("app");
    aethc_core::codegen::codegen_fn(&mut llcx, "main", &mir);

    if let Some("llvm") = emit {
        let txt = llcx.module.print_to_string();
        println!("{}", txt.to_string());
    }

    Ok(llcx)
}

// fn link_with_clang(bc: &std::path::Path, out: &std::path::Path) {
//     let target_dir = std::env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());
//     let status = std::process::Command::new("clang")
//         .arg("-O0")
//         .arg(bc)
//         .arg("-L")
//         .arg(format!("{target_dir}/debug"))
//         .arg("-laethc_runtime")
//         .arg("-o")
//         .arg(out)
//         .status()
//         .expect("failed to invoke clang");
//     if !status.success() {
//         eprintln!("link failed");
//         std::process::exit(1);
//     }
// }
// fn link_with_clang(bc: &Path, exe: &Path) {
//     // ── 1. target dir (пада на „target“ ако пром. нема) ───────────
//     let target_root = env::var("CARGO_TARGET_DIR").unwrap_or_else(|_| "target".into());

//     // ── 2. профил: runtime→PROFILE  ∥  compile-time→debug_assertions ─
//     let profile = env::var("PROFILE").unwrap_or_else(|_| {
//         if cfg!(debug_assertions) {
//             "debug".into()
//         } else {
//             "release".into()
//         }
//     });

//     // ── 3. triple: runtime→CARGO_CFG_TARGET_TRIPLE ∥ HOST/вг. компилера ─
//     let triple = env::var("CARGO_CFG_TARGET_TRIPLE")
//         .or_else(|_| env::var("HOST"))
//         .unwrap_or_else(|_| "x86_64-pc-windows-msvc".into());

//     // …\target\<triple>\<profile>
//     let lib_dir: PathBuf = [target_root.as_str(), &triple, &profile].iter().collect();

//     // MSVC линкер преко clang-а: −LIBPATH:<dir>
//     let libpath = format!("-LIBPATH:{}", lib_dir.display());

//     let status = Command::new("clang")
//         .arg("-O0")
//         .arg(bc)
//         .arg(libpath)
//         .arg("aethc_runtime.lib") // директно .lib
//         .arg("-o")
//         .arg(exe)
//         .status()
//         .expect("failed to invoke clang");

//     if !status.success() {
//         eprintln!("link failed (clang exit {})", status);
//         std::process::exit(1);
//     }
// }
fn link_with_clang(bc: &Path, exe: &Path) {
    // директоријум који је build.rs уписао
    let lib_dir = std::path::PathBuf::from(env!("AETHC_RUNTIME_DIR"));

    // пун пут до .lib
    let lib_path = lib_dir.join("aethc_runtime.lib");

    let status = Command::new("clang")
        .args([
            "-O0",
            bc.to_str().unwrap(),
            &format!("-LIBPATH:{}", lib_dir.display()),
            lib_path.to_str().unwrap(),
            "-o",
            exe.to_str().unwrap(),
        ])
        .status()
        .expect("failed to invoke clang");

    if !status.success() {
        eprintln!("link failed ({status})");
        std::process::exit(1);
    }
}
fn exit_code(ok: bool) -> std::process::ExitCode {
    if ok { 0.into() } else { 1.into() }
}

trait SpannedError {
    fn span(&self) -> Span;
    fn msg(&self) -> String;
}

impl SpannedError for aethc_core::LexError {
    fn span(&self) -> Span {
        self.span
    }
    fn msg(&self) -> String {
        self.msg.clone()
    }
}

impl SpannedError for aethc_core::resolver::ResolveError {
    fn span(&self) -> Span {
        self.span
    }
    fn msg(&self) -> String {
        self.msg.clone()
    }
}

impl SpannedError for aethc_core::borrow::BorrowError {
    fn span(&self) -> Span {
        self.span
    }
    fn msg(&self) -> String {
        format!("{:?}", self.kind)
    }
}
