use rustpython_parser::ast::{ExprKind, Stmt, StmtKind};

use crate::checks::{Check, CheckKind};
use crate::visitor::SingleNodeVisitor;

pub struct IfTupleChecker {
    checks: Vec<Check>,
}

impl IfTupleChecker {
    pub fn new() -> IfTupleChecker {
        IfTupleChecker { checks: vec![] }
    }
}

impl SingleNodeVisitor for IfTupleChecker {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::If { test, .. } = &stmt.node {
            if let ExprKind::Tuple { elts, .. } = &test.node {
                if !elts.is_empty() {
                    self.checks
                        .push(Check::new(CheckKind::IfTuple, stmt.location));
                }
            }
        }
    }
}
