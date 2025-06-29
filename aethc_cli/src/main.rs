use std::{env, fs, process::Command};

fn main() {
    let mut args = env::args().skip(1);
    let cmd = args.next().unwrap_or_else(|| {
        eprintln!("usage: aethc <parse|build> <file> [-o <out>]");
        std::process::exit(1);
    });

    match cmd.as_str() {
        "parse" => {
            let path = args
                .next()
                .unwrap_or_else(|| {
                    eprintln!("missing <file>");
                    std::process::exit(1)
                });
            let src = fs::read_to_string(&path).expect("failed to read file");
            let module = aethc_core::parser::Parser::new(&src).parse_module();
            println!("{:#?}", module);
        }
        "build" => {
            let path = args
                .next()
                .unwrap_or_else(|| {
                    eprintln!("missing <file>");
                    std::process::exit(1)
                });
            let mut out = "a.out".to_string();
            if args.next().as_deref() == Some("-o") {
                out = args
                    .next()
                    .unwrap_or_else(|| {
                        eprintln!("missing output after -o");
                        std::process::exit(1)
                    });
            }
            let src = fs::read_to_string(&path).expect("failed to read file");
            let ast_mod = aethc_core::parser::Parser::new(&src).parse_module();
            let (hir_mod, errs) = aethc_core::resolver::resolve(&ast_mod);
            if !errs.is_empty() {
                for e in errs {
                    eprintln!("{:?}", e);
                }
                std::process::exit(1);
            }
            let mir = match &hir_mod.items[0] {
                aethc_core::hir::Item::Fn(f) => aethc_core::mir::lower_fn(f),
                _ => {
                    eprintln!("expected a function");
                    std::process::exit(1);
                }
            };
            let mut llcx = aethc_core::codegen::new_module("app");
            aethc_core::codegen::codegen_fn(&mut llcx, "main", &mir);
            let bc_path = "temp.bc";
            aethc_core::codegen::write_ir(&llcx, bc_path);
            let _ = Command::new("clang")
                .arg("-O0")
                .arg("-no-pie")
                .arg(bc_path)
                .arg("-o")
                .arg(&out)
                .status();
        }
        _ => {
            eprintln!("usage: aethc <parse|build> <file> [-o <out>]");
            std::process::exit(1);
        }
    }
}
