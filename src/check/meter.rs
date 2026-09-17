//! RFC-033 §9 — expansion metering: deterministic budgets so a parameterized
//! design cannot expand unboundedly before the compiler notices.
//!
//! Three limits, charged BEFORE any push/insert so a tripped meter leaves no
//! partially materialized IR: 100,000 cumulative entered iterations,
//! 1,000,000 work items, 64 active loop frames. Metering activates ONLY when
//! the reachable expansion graph contains M2 syntax — a purely legacy graph
//! keeps the pre-RFC behavior (no counting, no limits, bytes unchanged).

use crate::ast::{Expr, ForStmt, GenericArg, GenericBound, LayoutBlock, Stmt};
use crate::diag::{Diagnostic, Diagnostics};
use crate::resolve::World;
use crate::span::Span;

pub const MAX_ITERATIONS: u64 = 100_000;
pub const MAX_WORK_ITEMS: u64 = 1_000_000;
pub const MAX_FRAMES: usize = 64;

/// True when the syntactic expansion graph rooted at `design` contains any
/// M2 construct — a `const`, a `for`, a layout const/loop, any non-literal
/// expression in a length/selector/placement/generic-arg position, or
/// `.len`. Device/part references contribute nothing.
pub fn metering_needed(world: &World, design: &crate::ast::DesignDef) -> bool {
    let mut visited: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    body_has_m2(world, &design.body, &mut visited)
}

fn body_has_m2(
    world: &World,
    body: &[Stmt],
    visited: &mut std::collections::BTreeSet<String>,
) -> bool {
    for stmt in body {
        match stmt {
            Stmt::Const(_) | Stmt::For(_) => return true,
            Stmt::For(ForStmt {
                start, end, body, ..
            }) => {
                if expr_is_m2(start) || expr_is_m2(end) {
                    return true;
                }
                if body_has_m2(world, body, visited) {
                    return true;
                }
            }
            Stmt::Inst(i) => {
                if let Some((e, _)) = &i.array_len {
                    if expr_is_m2(e) {
                        return true;
                    }
                }
                if i.ty.generic_args.iter().any(|a| match a {
                    GenericArg::Expr(_) => true,
                    _ => false,
                }) {
                    return true;
                }
            }
            Stmt::SubdesignUse(u) => {
                if let Some((e, _)) = &u.array_len {
                    if expr_is_m2(e) {
                        return true;
                    }
                }
                if let Some(sd) = world.subdesigns.get(&u.ty.name.name) {
                    if generics_have_int(&sd.generics) {
                        return true;
                    }
                    if visited.insert(u.ty.name.name.clone()) {
                        if body_has_m2(world, &sd.body, visited) {
                            return true;
                        }
                    }
                }
            }
            Stmt::Call(call) => {
                if call
                    .generic_args
                    .iter()
                    .any(|a| matches!(a, GenericArg::Expr(_)))
                {
                    return true;
                }
                if let Some(fn_def) = world.fns.get(&call.callee.name) {
                    if generics_have_int(&fn_def.generics) {
                        return true;
                    }
                    if visited.insert(call.callee.name.clone()) {
                        if body_has_m2(world, &fn_def.body, visited) {
                            return true;
                        }
                    }
                }
            }
            Stmt::Layout(block) => {
                if layout_has_m2(world, block, visited) {
                    return true;
                }
            }
            Stmt::Net(n) => {
                for m in &n.members {
                    if let Some(sel) = &m.index {
                        if index_sel_is_m2(sel) {
                            return true;
                        }
                    }
                }
            }
            Stmt::Nc(nc) => {
                for m in &nc.members {
                    if let Some(sel) = &m.index {
                        if index_sel_is_m2(sel) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn layout_has_m2(
    world: &World,
    block: &LayoutBlock,
    visited: &mut std::collections::BTreeSet<String>,
) -> bool {
    if !block.consts.is_empty() {
        return true;
    }
    if body_has_m2_layout_for(world, &block.loops, visited) {
        return true;
    }
    for p in &block.placements {
        if expr_is_m2(&p.at.0) || expr_is_m2(&p.at.1) || p.rotate.as_ref().is_some_and(expr_is_m2) {
            return true;
        }
        for seg in &p.path {
            if let Some((e, _)) = &seg.index {
                if expr_is_m2(e) {
                    return true;
                }
            }
        }
    }
    false
}

fn body_has_m2_layout_for(
    world: &World,
    loops: &[crate::ast::LayoutFor],
    visited: &mut std::collections::BTreeSet<String>,
) -> bool {
    for f in loops {
        if expr_is_m2(&f.start) || expr_is_m2(&f.end) {
            return true;
        }
        if !f.consts.is_empty() {
            return true;
        }
        for p in &f.placements {
            if expr_is_m2(&p.at.0) || expr_is_m2(&p.at.1) {
                return true;
            }
            if let Some(r) = &p.rotate {
                if expr_is_m2(r) {
                    return true;
                }
            }
        }
        if body_has_m2_layout_for(world, &f.loops, visited) {
            return true;
        }
    }
    false
}

fn generics_have_int(generics: &[crate::ast::GenericParam]) -> bool {
    generics
        .iter()
        .any(|g| matches!(g.bound, GenericBound::Int(_)))
}

/// An expression is M2 when it is anything but a bare literal (int or
/// parenthesized int) — names, `.len`, unary, binary all count.
fn expr_is_m2(e: &Expr) -> bool {
    match e {
        Expr::Int(_, _) => false,
        Expr::Paren(inner, _) => expr_is_m2(inner),
        _ => true,
    }
}

fn index_sel_is_m2(sel: &crate::ast::IndexSel) -> bool {
    match sel {
        crate::ast::IndexSel::Single(e, _) => expr_is_m2(e),
        crate::ast::IndexSel::Range {
            start, end, step, ..
        } => expr_is_m2(start) || expr_is_m2(end) || step.as_ref().is_some_and(expr_is_m2),
        crate::ast::IndexSel::List(items, _) => items.iter().any(expr_is_m2),
    }
}

/// The budget ledger. Every method returns `true` while within budget; the
/// FIRST overflow pushes one E1405 (with the site) and returns `false` —
/// every subsequent call also returns `false` so no caller continues
/// materializing after a trip.
pub struct Meter {
    pub active: bool,
    pub iterations: u64,
    pub work: u64,
    pub frames: usize,
    tripped: bool,
}

impl Meter {
    pub fn new(active: bool) -> Meter {
        Meter {
            active,
            iterations: 0,
            work: 0,
            frames: 0,
            tripped: false,
        }
    }

    /// Whether a budget has been exceeded (assembly is skipped after a trip).
    pub fn tripped(&self) -> bool {
        self.tripped
    }

    /// Charge `n` work items; on the first overflow push E1405 (once) and
    /// return false.
    pub fn charge(&mut self, n: u64, what: &str, span: Span, diags: &mut Diagnostics) -> bool {
        if !self.active || self.tripped {
            return !self.tripped;
        }
        let (next, over) = self.work.overflowing_add(n);
        if over || next > MAX_WORK_ITEMS {
            self.tripped = true;
            diags.push(Diagnostic::error(
                "E1405",
                span,
                format!(
                    "{} would exceed the deterministic expansion budget: {} work items (limit {})",
                    what,
                    next.min(u64::MAX / 2),
                    fmt_limit(MAX_WORK_ITEMS)
                ),
            ));
            return false;
        }
        self.work = next;
        true
    }

    /// One more entered iteration; the FIRST overflow (the MAX+1-th entry)
    /// trips E1405 naming the failing count.
    pub fn enter_iteration(&mut self, span: Span, diags: &mut Diagnostics) -> bool {
        if !self.active || self.tripped {
            return !self.tripped;
        }
        let next = self.iterations + 1;
        if next > MAX_ITERATIONS {
            self.tripped = true;
            diags.push(Diagnostic::error(
                "E1405",
                span,
                format!(
                    "iteration {} would exceed the deterministic expansion budget: {} iterations (limit {})",
                    next,
                    fmt_count(next),
                    fmt_limit(MAX_ITERATIONS)
                ),
            ));
            return false;
        }
        self.iterations = next;
        true
    }

    /// Enter a loop frame; the depth (64 active frames) is part of the
    /// budget.
    pub fn enter_frame(&mut self, span: Span, diags: &mut Diagnostics) -> bool {
        if !self.active || self.tripped {
            return !self.tripped;
        }
        if self.frames + 1 > MAX_FRAMES {
            self.tripped = true;
            diags.push(Diagnostic::error(
                "E1405",
                span,
                format!(
                    "loop nesting would exceed the deterministic expansion budget: {} active frames (limit {})",
                    self.frames + 1,
                    MAX_FRAMES
                ),
            ));
            return false;
        }
        self.frames += 1;
        true
    }

    pub fn leave_frame(&mut self) {
        if self.frames > 0 {
            self.frames -= 1;
        }
    }
}

fn fmt_limit(n: u64) -> String {
    fmt_count(n)
}

fn fmt_count(n: u64) -> String {
    // 100000 → "100,000" — matches the RFC's phrasing ("E1405 naming
    // 100,001 and the limit").
    let s = n.to_string();
    let mut out = String::new();
    let bytes = s.as_bytes();
    for (i, c) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*c as char);
    }
    out
}
