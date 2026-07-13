use crate::c::ast::*;

pub trait Fold {
    fn fold_expr(&mut self, expr: Expr) -> Expr;
    fn fold_stmt(&mut self, stmt: Stmt) -> Stmt;
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
        Expr::Arrow(obj, name, span) => Expr::Arrow(Box::new(folder.fold_expr(*obj)), name, span),
        Expr::Deref(e, span) => Expr::Deref(Box::new(folder.fold_expr(*e)), span),
        Expr::Ref(e, span) => Expr::Ref(Box::new(folder.fold_expr(*e)), span),
        Expr::Cast(e, t, span) => Expr::Cast(Box::new(folder.fold_expr(*e)), t, span),
        Expr::Sizeof(e, span) => Expr::Sizeof(Box::new(folder.fold_expr(*e)), span),
        Expr::Alignof(e, span) => Expr::Alignof(Box::new(folder.fold_expr(*e)), span),
        Expr::Ternary(cond, then, els, span) => Expr::Ternary(
            Box::new(folder.fold_expr(*cond)),
            Box::new(folder.fold_expr(*then)),
            Box::new(folder.fold_expr(*els)),
            span,
        ),
        Expr::Comma(exprs, span) => Expr::Comma(
            exprs.into_iter().map(|e| folder.fold_expr(e)).collect(),
            span,
        ),
        Expr::StmtExpr(stmts, span) => Expr::StmtExpr(
            stmts.into_iter().map(|s| folder.fold_stmt(s)).collect(),
            span,
        ),
        Expr::Paren(e, span) => Expr::Paren(Box::new(folder.fold_expr(*e)), span),
        Expr::Assign(left, right, span) => Expr::Assign(
            Box::new(folder.fold_expr(*left)),
            Box::new(folder.fold_expr(*right)),
            span,
        ),
        other => other,
    }
}

pub fn fold_stmt(folder: &mut dyn Fold, stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(e, span) => Stmt::Expr(folder.fold_expr(e), span),
        Stmt::Decl(f, span) => Stmt::Decl(f, span),
        Stmt::VarDecl(v, span) => Stmt::VarDecl(v, span),
        Stmt::If(cond, body, alt, span) => Stmt::If(
            folder.fold_expr(cond),
            Box::new(folder.fold_stmt(*body)),
            alt.map(|a| Box::new(folder.fold_stmt(*a))),
            span,
        ),
        Stmt::Switch(test, cases, span) => Stmt::Switch(folder.fold_expr(test), cases, span),
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
        Stmt::Return(e, span) => Stmt::Return(e.map(|e| folder.fold_expr(e)), span),
        Stmt::Block(b, span) => Stmt::Block(fold_block(folder, b), span),
        other => other,
    }
}
