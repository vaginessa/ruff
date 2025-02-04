use rustpython_ast::{Expr, ExprKind};

use crate::ast::types::{Range, ScopeKind};
use crate::checkers::ast::Checker;
use crate::registry::Diagnostic;
use crate::violations;

fn is_cache_func(checker: &Checker, expr: &Expr) -> bool {
    checker.resolve_call_path(expr).map_or(false, |call_path| {
        call_path == ["functools", "lru_cache"] || call_path == ["functools", "cache"]
    })
}

/// B019
pub fn cached_instance_method(checker: &mut Checker, decorator_list: &[Expr]) {
    if !matches!(checker.current_scope().kind, ScopeKind::Class(_)) {
        return;
    }
    for decorator in decorator_list {
        // TODO(charlie): This should take into account `classmethod-decorators` and
        // `staticmethod-decorators`.
        if let ExprKind::Name { id, .. } = &decorator.node {
            if id == "classmethod" || id == "staticmethod" {
                return;
            }
        }
    }
    for decorator in decorator_list {
        if is_cache_func(
            checker,
            match &decorator.node {
                ExprKind::Call { func, .. } => func,
                _ => decorator,
            },
        ) {
            checker.diagnostics.push(Diagnostic::new(
                violations::CachedInstanceMethod,
                Range::from_located(decorator),
            ));
        }
    }
}
