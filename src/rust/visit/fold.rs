use crate::rust::ast::*;

pub trait Fold {
    fn fold_expr(&mut self, expr: Expr) -> Expr;
    fn fold_stmt(&mut self, stmt: Stmt) -> Stmt;
    fn fold_item(&mut self, item: Item) -> Item;
    fn fold_pattern(&mut self, pat: Pattern) -> Pattern;
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
        Expr::MethodCall(obj, name, args, span) => Expr::MethodCall(
            Box::new(folder.fold_expr(*obj)),
            name,
            args.into_iter().map(|a| folder.fold_expr(a)).collect(),
            span,
        ),
        Expr::Index(arr, idx, span) => Expr::Index(
            Box::new(folder.fold_expr(*arr)),
            Box::new(folder.fold_expr(*idx)),
            span,
        ),
        Expr::Field(obj, name, span) => Expr::Field(Box::new(folder.fold_expr(*obj)), name, span),
        Expr::Tuple(elems, span) => Expr::Tuple(
            elems.into_iter().map(|e| folder.fold_expr(e)).collect(),
            span,
        ),
        Expr::Array(elems, span) => Expr::Array(
            elems.into_iter().map(|e| folder.fold_expr(e)).collect(),
            span,
        ),
        Expr::Struct(name, fields, base, span) => Expr::Struct(
            name,
            fields,
            base.map(|b| Box::new(folder.fold_expr(*b))),
            span,
        ),
        Expr::Closure(params, body, span) => {
            Expr::Closure(params, Box::new(folder.fold_expr(*body)), span)
        }
        Expr::Block(b, span) => Expr::Block(Box::new(fold_block(folder, *b)), span),
        Expr::If(cond, body, alt, span) => Expr::If(
            Box::new(folder.fold_expr(*cond)),
            Box::new(fold_block(folder, *body)),
            alt.map(|a| Box::new(folder.fold_expr(*a))),
            span,
        ),
        Expr::Match(expr_, arms, span) => {
            Expr::Match(Box::new(folder.fold_expr(*expr_)), arms, span)
        }
        Expr::While(cond, body, span) => Expr::While(
            Box::new(folder.fold_expr(*cond)),
            Box::new(fold_block(folder, *body)),
            span,
        ),
        Expr::Loop(body, span) => Expr::Loop(Box::new(fold_block(folder, *body)), span),
        Expr::For(pat, iter, body, span) => Expr::For(
            Box::new(folder.fold_pattern(*pat)),
            Box::new(folder.fold_expr(*iter)),
            Box::new(fold_block(folder, *body)),
            span,
        ),
        Expr::Return(e, span) => Expr::Return(e.map(|e| Box::new(folder.fold_expr(*e))), span),
        Expr::Break(e, span) => Expr::Break(e.map(|e| Box::new(folder.fold_expr(*e))), span),
        Expr::Paren(e, span) => Expr::Paren(Box::new(folder.fold_expr(*e)), span),
        Expr::Async(e, span) => Expr::Async(Box::new(folder.fold_expr(*e)), span),
        Expr::Await(e, span) => Expr::Await(Box::new(folder.fold_expr(*e)), span),
        Expr::Ref(e, mut_, span) => Expr::Ref(Box::new(folder.fold_expr(*e)), mut_, span),
        Expr::Deref(e, span) => Expr::Deref(Box::new(folder.fold_expr(*e)), span),
        Expr::Cast(e, t, span) => Expr::Cast(Box::new(folder.fold_expr(*e)), t, span),
        other => other,
    }
}

pub fn fold_stmt(folder: &mut dyn Fold, stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(e, span) => Stmt::Expr(folder.fold_expr(e), span),
        Stmt::Item(item, span) => Stmt::Item(folder.fold_item(item), span),
        Stmt::Let(l, span) => Stmt::Let(
            LetDecl {
                pattern: folder.fold_pattern(l.pattern),
                mutable: l.mutable,
                type_ann: l.type_ann,
                init: l.init.map(|i| Box::new(folder.fold_expr(*i))),
                span,
            },
            span,
        ),
        Stmt::Empty(_) => stmt,
    }
}

pub fn fold_item(folder: &mut dyn Fold, item: Item) -> Item {
    match item {
        Item::Fn(f, span) => Item::Fn(
            FnDecl {
                name: f.name,
                generics: f.generics,
                params: f.params,
                return_type: f.return_type,
                body: f.body.map(|b| fold_block(folder, b)),
                visibility: f.visibility,
                is_async: f.is_async,
                is_unsafe: f.is_unsafe,
                is_extern: f.is_extern,
                span,
            },
            span,
        ),
        Item::Mod(m, span) => Item::Mod(
            ModDecl {
                name: m.name,
                items: m.items.into_iter().map(|i| folder.fold_item(i)).collect(),
                visibility: m.visibility,
                span,
            },
            span,
        ),
        Item::Impl(imp, span) => Item::Impl(
            ImplBlock {
                trait_name: imp.trait_name,
                type_name: imp.type_name,
                methods: imp
                    .methods
                    .into_iter()
                    .map(|m| FnDecl {
                        name: m.name,
                        generics: m.generics,
                        params: m.params,
                        return_type: m.return_type,
                        body: m.body.map(|b| fold_block(folder, b)),
                        visibility: m.visibility,
                        is_async: m.is_async,
                        is_unsafe: m.is_unsafe,
                        is_extern: m.is_extern,
                        span: m.span,
                    })
                    .collect(),
                span,
            },
            span,
        ),
        other => other,
    }
}

pub fn fold_pattern(folder: &mut dyn Fold, pat: Pattern) -> Pattern {
    match pat {
        Pattern::Tuple(pats, span) => Pattern::Tuple(
            pats.into_iter().map(|p| folder.fold_pattern(p)).collect(),
            span,
        ),
        Pattern::Struct(name, fields, span) => Pattern::Struct(name, fields, span),
        Pattern::Range(low, high, span) => Pattern::Range(
            Box::new(folder.fold_pattern(*low)),
            Box::new(folder.fold_pattern(*high)),
            span,
        ),
        Pattern::Or(pats, span) => Pattern::Or(
            pats.into_iter().map(|p| folder.fold_pattern(p)).collect(),
            span,
        ),
        Pattern::Ref(pat, mut_, span) => {
            Pattern::Ref(Box::new(folder.fold_pattern(*pat)), mut_, span)
        }
        Pattern::Slice(pats, span) => Pattern::Slice(
            pats.into_iter().map(|p| folder.fold_pattern(p)).collect(),
            span,
        ),
        other => other,
    }
}
