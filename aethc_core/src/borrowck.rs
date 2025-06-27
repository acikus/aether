//! borrowck.rs – минимални borrow-checker α
//! Правило: име се сме поново дефинисати у истом блоку
//!          само ако је претходна дефиниција имала `mutable: true`.

use crate::{hir, resolver::ResolveError};
use crate::lexer::Span;
use std::collections::HashMap;

/*────────── јавни улаз ──────────*/
pub fn borrow_check(m: &hir::HirModule) -> Vec<ResolveError> {
    let mut errs = Vec::new();
    for it in &m.items {
        if let hir::Item::Fn(f) = it {
            check_block(&f.body, &mut errs, &mut HashMap::new());
        }
    }
    errs
}

/*────────── рекурзивна провера блока ──────────*/
fn check_block(
    blk: &hir::Block,
    errs: &mut Vec<ResolveError>,
    defined: &mut HashMap<String, bool>, // name → mutable?
) {
    for st in &blk.stmts {
        match st {
            hir::Stmt::Let(l) => {
                match defined.get(&l.name) {
                    Some(prev_mut) if !prev_mut => {
                        // већ постоји immutable – грешка
                        errs.push(ResolveError {
                            span: Span { start: 0, end: 0, line: 0, column: 0 },
                            msg:  format!("cannot reassign immutable binding `{}`", l.name),
                        });
                    }
                    _ => {
                        // уносимо/ажурирамо запис
                        defined.insert(l.name.clone(), l.mutable);
                    }
                };
            }
            hir::Stmt::Expr(_) | hir::Stmt::Semi(_) | hir::Stmt::Return(_) => {}
        }
    }
}
