use crate::go::ast::*;

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
        Expr::Binary(left, op, right, span) => {
            let left = Box::new(folder.fold_expr(*left));
            let right = Box::new(folder.fold_expr(*right));
            Expr::Binary(left, op, right, span)
        }
        Expr::Unary(op, operand, span) => {
            let operand = Box::new(folder.fold_expr(*operand));
            Expr::Unary(op, operand, span)
        }
        Expr::Call(callee, args, span) => {
            let callee = Box::new(folder.fold_expr(*callee));
            let args = args.into_iter().map(|a| folder.fold_expr(a)).collect();
            Expr::Call(callee, args, span)
        }
        Expr::Index(arr, idx, span) => {
            let arr = Box::new(folder.fold_expr(*arr));
            let idx = Box::new(folder.fold_expr(*idx));
            Expr::Index(arr, idx, span)
        }
        Expr::Selector(obj, name, span) => {
            let obj = Box::new(folder.fold_expr(*obj));
            Expr::Selector(obj, name, span)
        }
        Expr::Slice(arr, low, high, span) => {
            let arr = Box::new(folder.fold_expr(*arr));
            let low = low.map(|l| Box::new(folder.fold_expr(*l)));
            let high = high.map(|h| Box::new(folder.fold_expr(*h)));
            Expr::Slice(arr, low, high, span)
        }
        Expr::Array(elems, span) => {
            let elems = elems.into_iter().map(|e| folder.fold_expr(e)).collect();
            Expr::Array(elems, span)
        }
        Expr::StructLit(name, fields, span) => {
            let fields = fields
                .into_iter()
                .map(|f| FieldInit {
                    name: f.name,
                    value: f.value.map(|v| Box::new(folder.fold_expr(*v))),
                    span: f.span,
                })
                .collect();
            Expr::StructLit(name, fields, span)
        }
        Expr::MapLit(entries, span) => {
            let entries = entries
                .into_iter()
                .map(|(k, v)| (folder.fold_expr(k), folder.fold_expr(v)))
                .collect();
            Expr::MapLit(entries, span)
        }
        Expr::FuncLit(ft, body, span) => {
            let body = Box::new(fold_block(folder, *body));
            Expr::FuncLit(ft, body, span)
        }
        Expr::Paren(e, span) => {
            let e = Box::new(folder.fold_expr(*e));
            Expr::Paren(e, span)
        }
        Expr::TypeAssert(e, t, span) => {
            let e = Box::new(folder.fold_expr(*e));
            Expr::TypeAssert(e, t, span)
        }
        Expr::CompositeLit(t, elems, span) => {
            let t = Box::new(folder.fold_expr(*t));
            let elems = elems.into_iter().map(|e| folder.fold_expr(e)).collect();
            Expr::CompositeLit(t, elems, span)
        }
        other => other,
    }
}

pub fn fold_stmt(folder: &mut dyn Fold, stmt: Stmt) -> Stmt {
    match stmt {
        Stmt::Expr(e, span) => Stmt::Expr(folder.fold_expr(e), span),
        Stmt::Decl(d, span) => Stmt::Decl(folder.fold_decl(d), span),
        Stmt::Assign(targets, values, span) => {
            let targets = targets.into_iter().map(|t| folder.fold_expr(t)).collect();
            let values = values.into_iter().map(|v| folder.fold_expr(v)).collect();
            Stmt::Assign(targets, values, span)
        }
        Stmt::If(cond, body, alt, span) => {
            let cond = folder.fold_expr(cond);
            let body = Box::new(folder.fold_stmt(*body));
            let alt = alt.map(|a| Box::new(folder.fold_stmt(*a)));
            Stmt::If(cond, body, alt, span)
        }
        Stmt::For(init, test, update, body, span) => {
            let init = init.map(|i| Box::new(folder.fold_stmt(*i)));
            let test = test.map(|t| folder.fold_expr(t));
            let update = update.map(|u| Box::new(folder.fold_stmt(*u)));
            let body = Box::new(folder.fold_stmt(*body));
            Stmt::For(init, test, update, body, span)
        }
        Stmt::Block(block, span) => Stmt::Block(fold_block(folder, block), span),
        Stmt::Return(vals, span) => {
            let vals = vals.into_iter().map(|v| folder.fold_expr(v)).collect();
            Stmt::Return(vals, span)
        }
        Stmt::Defer(e, span) => Stmt::Defer(folder.fold_expr(e), span),
        Stmt::Go(e, span) => Stmt::Go(folder.fold_expr(e), span),
        Stmt::Send(ch, val, span) => Stmt::Send(folder.fold_expr(ch), folder.fold_expr(val), span),
        Stmt::Label(name, s, span) => Stmt::Label(name, Box::new(folder.fold_stmt(*s)), span),
        _ => stmt,
    }
}

pub fn fold_decl(folder: &mut dyn Fold, decl: Decl) -> Decl {
    match decl {
        Decl::Func(f, span) => Decl::Func(
            FuncDecl {
                name: f.name,
                receiver: f.receiver,
                params: f.params,
                returns: f.returns,
                body: f.body.map(|b| fold_block(folder, b)),
                span,
            },
            span,
        ),
        Decl::Var(v, span) => Decl::Var(v, span),
        Decl::Const(c, span) => Decl::Const(c, span),
        Decl::Type(t, span) => Decl::Type(t, span),
        Decl::Import(i, span) => Decl::Import(i, span),
        Decl::ImportGroup(imports, span) => Decl::ImportGroup(imports, span),
        Decl::Package(n, span) => Decl::Package(n, span),
    }
}
