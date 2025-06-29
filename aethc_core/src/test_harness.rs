use crate::borrow::{BorrowCtx, BorrowState, BorrowError};
use crate::hir;
use std::collections::HashMap;

pub struct BorrowOutput {
    pub errors: Vec<BorrowError>,
}

/// Very small parser for borrow-checker tests.
/// Supports only statements used in the integration tests.
pub fn compile_and_borrow(src: &str) -> BorrowOutput {
    let mut ids = HashMap::new();
    let mut next_id: u32 = 0;

    // dummy block required by BorrowCtx but never used
    let dummy = hir::Block { id: 0, stmts: vec![] };
    let mut cx = BorrowCtx::new(&dummy);

    for line in src.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("fn ") || line == "{" || line == "}" {
            continue;
        }

        if line.starts_with("let ") {
            let rest = line.strip_prefix("let ").unwrap();
            let rest = rest.trim_end_matches(';').trim();
            let mut_parts: Vec<&str> = rest.splitn(2, '=').collect();
            let left = mut_parts[0].trim();
            let expr = mut_parts.get(1).map(|s| s.trim());
            let name = if left.starts_with("mut ") {
                left[4..].trim()
            } else {
                left
            };
            let id = next_id;
            next_id += 1;
            ids.insert(name.to_string(), id);
            if let Some(expr) = expr {
                if expr.starts_with("&mut ") {
                    if let Some(&target) = ids.get(expr[5..].trim()) {
                        cx.borrow_var(target);
                    }
                } else if let Some(&src_id) = ids.get(expr) {
                    cx.move_var(src_id);
                }
            }
            cx.states.insert(id, BorrowState::Live);
            if expr.map_or(true, |e| !e.starts_with("&mut ")) {
                cx.cleanup();
            }
        } else if line.starts_with('*') {
            // use through deref: *y = ...
            let name = line[1..].split('=').next().unwrap().trim();
            if let Some(&id) = ids.get(name) {
                cx.use_var(id);
            }
            cx.cleanup();
        } else {
            cx.cleanup();
        }
    }

    BorrowOutput { errors: cx.errors }
}
