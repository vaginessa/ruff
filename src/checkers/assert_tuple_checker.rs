use rustpython_parser::ast::{ExprKind, Stmt, StmtKind};

use crate::checks::{Check, CheckKind};
use crate::visitor::SingleNodeVisitor;

pub struct AssertTupleChecker {
    checks: Vec<Check>,
}

impl AssertTupleChecker {
    pub fn new() -> AssertTupleChecker {
        AssertTupleChecker { checks: vec![] }
    }
}

impl SingleNodeVisitor for AssertTupleChecker {
    fn visit_stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::Assert { test, .. } = &stmt.node {
            if let ExprKind::Tuple { elts, .. } = &test.node {
                if !elts.is_empty() {
                    self.checks
                        .push(Check::new(CheckKind::IfTuple, stmt.location));
                }
            }
        }
    }
}
