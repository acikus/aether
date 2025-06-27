use std::{env, fs};

fn main() {
    // expect: aethc parse <file>
    let mut args = env::args().skip(1);
    if args.next().as_deref() != Some("parse") {
        eprintln!("usage: aethc parse <file>");
        std::process::exit(1);
    }
    let path = args
        .next()
        .unwrap_or_else(|| { eprintln!("missing <file>"); std::process::exit(1) });

    let src = fs::read_to_string(&path).expect("failed to read file");
    let module = aethc_core::parser::Parser::new(&src).parse_module();

    println!("{:#?}", module);
}
