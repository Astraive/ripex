use crate::python::ast::*;

pub trait Fold {
    fn fold_expr(&mut self, expr: Expr) -> Expr;
    fn fold_stmt(&mut self, stmt: Stmt) -> Stmt;
    fn fold_pattern(&mut self, pat: Pattern) -> Pattern;
}

pub fn fold_expr(folder: &mut dyn Fold, expr: Expr) -> Expr {
    match expr {
        Expr::Binary(left, op, right, span) => Expr::Binary(
            Box::new(folder.fold_expr(*left)),
            op,
            Box::new(folder.fold_expr(*right)),
            span,
        ),
        Expr::Unary(op, operand, span) => {
            Expr::Unary(op, Box::new(folder.fold_expr(*operand)), span)
        }
        Expr::Call(callee, args, keywords, span) => Expr::Call(
            Box::new(folder.fold_expr(*callee)),
            args.into_iter().map(|a| folder.fold_expr(a)).collect(),
            keywords,
            span,
        ),
        Expr::IfElse(cond, then, else_, span) => Expr::IfElse(
            Box::new(folder.fold_expr(*cond)),
            Box::new(folder.fold_expr(*then)),
            Box::new(folder.fold_expr(*else_)),
            span,
        ),
        Expr::Lambda(params, body, span) => {
            Expr::Lambda(params, Box::new(folder.fold_expr(*body)), span)
        }
        Expr::List(elems, span) => Expr::List(
            elems.into_iter().map(|e| folder.fold_expr(e)).collect(),
            span,
        ),
        Expr::Tuple(elems, span) => Expr::Tuple(
            elems.into_iter().map(|e| folder.fold_expr(e)).collect(),
            span,
        ),
        Expr::Dict(entries, span) => Expr::Dict(
            entries
                .into_iter()
                .map(|(k, v)| (folder.fold_expr(k), folder.fold_expr(v)))
                .collect(),
            span,
        ),
        Expr::Await(e, span) => Expr::Await(Box::new(folder.fold_expr(*e)), span),
        Expr::Starred(e, span) => Expr::Starred(Box::new(folder.fold_expr(*e)), span),
        Expr::Walrus(target, value, span) => Expr::Walrus(
            Box::new(folder.fold_expr(*target)),
            Box::new(folder.fold_expr(*value)),
            span,
        ),
        Expr::Compare(left, ops, comparators, span) => Expr::Compare(
            Box::new(folder.fold_expr(*left)),
            ops,
            comparators
                .into_iter()
                .map(|c| Box::new(folder.fold_expr(*c)))
                .collect(),
            span,
        ),
        Expr::Paren(e, span) => Expr::Paren(Box::new(folder.fold_expr(*e)), span),
        Expr::Attribute(obj, name, span) => {
            Expr::Attribute(Box::new(folder.fold_expr(*obj)), name, span)
        }
        Expr::Subscript(obj, idx, span) => Expr::Subscript(
            Box::new(folder.fold_expr(*obj)),
            Box::new(folder.fold_expr(*idx)),
            span,
        ),
        Expr::Yield(e, span) => Expr::Yield(e.map(|e| Box::new(folder.fold_expr(*e))), span),
        Expr::YieldFrom(e, span) => Expr::YieldFrom(Box::new(folder.fold_expr(*e)), span),
        other => other,
    }
}

fn fold_body(folder: &mut dyn Fold, body: Vec<Stmt>) -> Vec<Stmt> {
    body.into_iter().map(|s| folder.fold_stmt(s)).collect()
}

pub fn fold_stmt(folder: &mut dyn Fold, stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(e, span) => Stmt::Expr(folder.fold_expr(e), span),
        Stmt::Assign(target, value, span) => Stmt::Assign(
            Box::new(folder.fold_expr(*target)),
            Box::new(folder.fold_expr(*value)),
            span,
        ),
        Stmt::AugAssign(target, op, value, span) => Stmt::AugAssign(
            Box::new(folder.fold_expr(*target)),
            op,
            Box::new(folder.fold_expr(*value)),
            span,
        ),
        Stmt::If(cond, body, else_, span) => Stmt::If(
            Box::new(folder.fold_expr(*cond)),
            fold_body(folder, body),
            fold_body(folder, else_),
            span,
        ),
        Stmt::While(cond, body, else_, span) => Stmt::While(
            Box::new(folder.fold_expr(*cond)),
            fold_body(folder, body),
            else_.map(|e| fold_body(folder, e)),
            span,
        ),
        Stmt::For(target, iter, body, else_, span) => Stmt::For(
            Box::new(folder.fold_expr(*target)),
            Box::new(folder.fold_expr(*iter)),
            fold_body(folder, body),
            else_.map(|e| fold_body(folder, e)),
            span,
        ),
        Stmt::With(items, body, span) => Stmt::With(items, fold_body(folder, body), span),
        Stmt::Return(e, span) => Stmt::Return(e.map(|e| folder.fold_expr(e)), span),
        Stmt::Raise(e, _, span) => Stmt::Raise(e.map(|e| folder.fold_expr(e)), None, span),
        Stmt::Assert(test, msg, span) => Stmt::Assert(folder.fold_expr(test), msg, span),
        Stmt::Delete(e, span) => Stmt::Delete(folder.fold_expr(e), span),
        Stmt::Block(body, span) => Stmt::Block(fold_body(folder, body), span),
        Stmt::Async(s, span) => Stmt::Async(Box::new(folder.fold_stmt(*s)), span),
        Stmt::FuncDef(f, span) => Stmt::FuncDef(
            FuncDef {
                name: f.name,
                args: f.args,
                body: fold_body(folder, f.body),
                decorators: f
                    .decorators
                    .into_iter()
                    .map(|d| folder.fold_expr(d))
                    .collect(),
                returns: f.returns.map(|r| Box::new(folder.fold_expr(*r))),
                is_async: f.is_async,
                is_generator: f.is_generator,
                defaults: f
                    .defaults
                    .into_iter()
                    .map(|d| folder.fold_expr(d))
                    .collect(),
                kw_defaults: f.kw_defaults,
                vararg: f.vararg,
                kwarg: f.kwarg,
                span,
            },
            span,
        ),
        Stmt::ClassDef(c, span) => Stmt::ClassDef(
            ClassDef {
                name: c.name,
                bases: c.bases.into_iter().map(|b| folder.fold_expr(b)).collect(),
                keywords: c.keywords,
                body: fold_body(folder, c.body),
                decorators: c
                    .decorators
                    .into_iter()
                    .map(|d| folder.fold_expr(d))
                    .collect(),
                span,
            },
            span,
        ),
        _ => stmt,
    }
}

pub fn fold_pattern(folder: &mut dyn Fold, pat: Pattern) -> Pattern {
    match pat {
        Pattern::Sequence(pats, span) => Pattern::Sequence(
            pats.into_iter().map(|p| folder.fold_pattern(p)).collect(),
            span,
        ),
        Pattern::Mapping(items, rest, span) => Pattern::Mapping(
            items
                .into_iter()
                .map(|(k, v)| (folder.fold_pattern(k), folder.fold_pattern(v)))
                .collect(),
            rest,
            span,
        ),
        Pattern::Class(name, args, kwargs, span) => Pattern::Class(
            name,
            args.into_iter().map(|a| folder.fold_pattern(a)).collect(),
            kwargs,
            span,
        ),
        Pattern::Or(pats, span) => Pattern::Or(
            pats.into_iter().map(|p| folder.fold_pattern(p)).collect(),
            span,
        ),
        Pattern::As(pat, name, span) => {
            Pattern::As(Box::new(folder.fold_pattern(*pat)), name, span)
        }
        Pattern::Guard(pat, guard, span) => Pattern::Guard(
            Box::new(folder.fold_pattern(*pat)),
            Box::new(folder.fold_expr(*guard)),
            span,
        ),
        Pattern::Group(pat, span) => Pattern::Group(Box::new(folder.fold_pattern(*pat)), span),
        other => other,
    }
}
