use crate::csharp::ast::*;

pub trait Fold {
    fn fold_expr(&mut self, expr: Expr) -> Expr;
    fn fold_stmt(&mut self, stmt: Stmt) -> Stmt;
    fn fold_decl(&mut self, decl: Decl) -> Decl;
}

fn fold_block(folder: &mut dyn Fold, block: Block) -> Block {
    Block {
        stmts: block
            .stmts
            .into_iter()
            .map(|s| folder.fold_stmt(s))
            .collect(),
        span: block.span,
    }
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
        Expr::Call(callee, args, span) => Expr::Call(
            Box::new(folder.fold_expr(*callee)),
            args.into_iter().map(|a| folder.fold_expr(a)).collect(),
            span,
        ),
        Expr::Index(arr, idx, span) => Expr::Index(
            Box::new(folder.fold_expr(*arr)),
            Box::new(folder.fold_expr(*idx)),
            span,
        ),
        Expr::Member(obj, name, span) => Expr::Member(Box::new(folder.fold_expr(*obj)), name, span),
        Expr::Conditional(cond, then, els, span) => Expr::Conditional(
            Box::new(folder.fold_expr(*cond)),
            Box::new(folder.fold_expr(*then)),
            Box::new(folder.fold_expr(*els)),
            span,
        ),
        Expr::NullCoalesce(left, right, span) => Expr::NullCoalesce(
            Box::new(folder.fold_expr(*left)),
            Box::new(folder.fold_expr(*right)),
            span,
        ),
        Expr::NullConditional(obj, name, span) => {
            Expr::NullConditional(Box::new(folder.fold_expr(*obj)), name, span)
        }
        Expr::Lambda(l, span) => Expr::Lambda(l, span),
        Expr::Paren(e, span) => Expr::Paren(Box::new(folder.fold_expr(*e)), span),
        Expr::Assign(left, right, span) => Expr::Assign(
            Box::new(folder.fold_expr(*left)),
            Box::new(folder.fold_expr(*right)),
            span,
        ),
        Expr::Await(e, span) => Expr::Await(Box::new(folder.fold_expr(*e)), span),
        Expr::Throw(e, span) => Expr::Throw(Box::new(folder.fold_expr(*e)), span),
        other => other,
    }
}

pub fn fold_stmt(folder: &mut dyn Fold, stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(e, span) => Stmt::Expr(folder.fold_expr(e), span),
        Stmt::Decl(d, span) => Stmt::Decl(folder.fold_decl(d), span),
        Stmt::If(cond, body, alt, span) => Stmt::If(
            folder.fold_expr(cond),
            Box::new(folder.fold_stmt(*body)),
            alt.map(|a| Box::new(folder.fold_stmt(*a))),
            span,
        ),
        Stmt::While(cond, body, span) => Stmt::While(
            folder.fold_expr(cond),
            Box::new(folder.fold_stmt(*body)),
            span,
        ),
        Stmt::Do(body, test, span) => Stmt::Do(
            Box::new(folder.fold_stmt(*body)),
            folder.fold_expr(test),
            span,
        ),
        Stmt::For(init, test, update, body, span) => Stmt::For(
            init.map(|i| Box::new(folder.fold_stmt(*i))),
            test.map(|t| folder.fold_expr(t)),
            update.map(|u| Box::new(folder.fold_stmt(*u))),
            Box::new(folder.fold_stmt(*body)),
            span,
        ),
        Stmt::Block(b, span) => Stmt::Block(fold_block(folder, b), span),
        Stmt::Return(e, span) => Stmt::Return(e.map(|e| folder.fold_expr(e)), span),
        Stmt::Throw(e, span) => Stmt::Throw(e.map(|e| folder.fold_expr(e)), span),
        other => other,
    }
}

pub fn fold_decl(folder: &mut dyn Fold, decl: Decl) -> Decl {
    match decl {
        Decl::Namespace(name, members, span) => Decl::Namespace(
            name,
            members.into_iter().map(|d| folder.fold_decl(d)).collect(),
            span,
        ),
        Decl::Method(f, span) => Decl::Method(
            FuncDecl {
                name: f.name,
                return_type: f.return_type,
                params: f.params,
                body: f.body.map(|b| fold_block(folder, b)),
                is_async: f.is_async,
                is_static: f.is_static,
                is_virtual: f.is_virtual,
                is_override: f.is_override,
                is_abstract: f.is_abstract,
                is_sealed: f.is_sealed,
                is_unsafe: f.is_unsafe,
                is_extern: f.is_extern,
                is_partial: f.is_partial,
                visibility: f.visibility,
                type_params: f.type_params,
                span,
            },
            span,
        ),
        _ => decl,
    }
}
