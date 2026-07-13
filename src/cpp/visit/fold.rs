use crate::cpp::ast::*;

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
        Expr::Arrow(obj, name, span) => Expr::Arrow(Box::new(folder.fold_expr(*obj)), name, span),
        Expr::Deref(e, span) => Expr::Deref(Box::new(folder.fold_expr(*e)), span),
        Expr::Ref(e, span) => Expr::Ref(Box::new(folder.fold_expr(*e)), span),
        Expr::Cast(e, t, span) => Expr::Cast(Box::new(folder.fold_expr(*e)), t, span),
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
        Expr::Lambda(l, span) => Expr::Lambda(
            LambdaExpr {
                captures: l.captures,
                params: l.params,
                return_type: l.return_type,
                body: Box::new(fold_block(folder, *l.body)),
                span: l.span,
            },
            span,
        ),
        Expr::New(t, args, span) => Expr::New(
            Box::new(folder.fold_expr(*t)),
            args.into_iter().map(|a| folder.fold_expr(a)).collect(),
            span,
        ),
        Expr::Delete(e, span) => Expr::Delete(Box::new(folder.fold_expr(*e)), span),
        Expr::Paren(e, span) => Expr::Paren(Box::new(folder.fold_expr(*e)), span),
        Expr::Assign(left, right, span) => Expr::Assign(
            Box::new(folder.fold_expr(*left)),
            Box::new(folder.fold_expr(*right)),
            span,
        ),
        Expr::Template(t, args, span) => Expr::Template(
            Box::new(folder.fold_expr(*t)),
            args.into_iter().map(|a| folder.fold_expr(a)).collect(),
            span,
        ),
        Expr::BraceInit(exprs, span) => Expr::BraceInit(
            exprs.into_iter().map(|e| folder.fold_expr(e)).collect(),
            span,
        ),
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
        Stmt::Return(e, span) => Stmt::Return(e.map(|e| folder.fold_expr(e)), span),
        Stmt::Throw(e, span) => Stmt::Throw(e.map(|e| folder.fold_expr(e)), span),
        Stmt::Block(b, span) => Stmt::Block(fold_block(folder, b), span),
        other => other,
    }
}

pub fn fold_decl(folder: &mut dyn Fold, decl: Decl) -> Decl {
    match decl {
        Decl::Func(f, span) => Decl::Func(
            FuncDecl {
                name: f.name,
                return_type: f.return_type,
                params: f.params,
                is_variadic: f.is_variadic,
                body: f.body.map(|b| fold_block(folder, b)),
                is_virtual: f.is_virtual,
                is_override: f.is_override,
                is_const: f.is_const,
                is_pure: f.is_pure,
                is_constexpr: f.is_constexpr,
                is_inline: f.is_inline,
                is_explicit: f.is_explicit,
                is_static: f.is_static,
                is_friend: f.is_friend,
                span,
            },
            span,
        ),
        Decl::Namespace(name, members, span) => Decl::Namespace(
            name,
            members.into_iter().map(|d| folder.fold_decl(d)).collect(),
            span,
        ),
        _ => decl,
    }
}
