pub mod lexer;
pub mod type_;
pub mod ast;
pub mod hir;
pub mod parser;
pub mod resolver;
pub mod borrowck;
pub mod borrow;
pub mod infer_ctx;
pub mod type_inference;
pub mod test_harness;
pub mod mir;
pub mod codegen;

use lexer::Span;

#[derive(Debug, Clone)]
pub struct LexError {
    pub span: Span,
    pub msg: String,
}

pub fn parse(src: &str) -> (ast::Module, Vec<LexError>) {
    // Current lexer does not report errors, so return empty list
    let module = parser::Parser::new(src).parse_module();
    (module, Vec::new())
}

pub fn lower_to_hir(ast: &ast::Module, _src: &str) -> (hir::HirModule, Vec<resolver::ResolveError>) {
    resolver::resolve(ast)
}
