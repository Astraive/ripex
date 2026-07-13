use crate::arena::Arena;
use crate::js::ast::*;

pub trait Fold {
    fn fold_expr(&mut self, expr_ref: ExprRef, ast: &mut Arena<Expr>) -> ExprRef;
    fn fold_stmt(&mut self, stmt: Stmt, ast: &mut Arena<Expr>) -> Stmt;
    fn fold_decl(&mut self, decl: Decl, ast: &mut Arena<Expr>) -> Decl;
    fn fold_pat(&mut self, pat: Pat, ast: &mut Arena<Expr>) -> Pat;
}

pub fn fold_expr(folder: &mut dyn Fold, expr_ref: ExprRef, ast: &mut Arena<Expr>) -> ExprRef {
    let expr = ast[expr_ref].clone();
    match expr {
        Expr::Ident(_)
        | Expr::Lit(_)
        | Expr::This(_)
        | Expr::Super(_)
        | Expr::MetaProperty(_)
        | Expr::PrivateName(_)
        | Expr::Invalid(_) => expr_ref,
        Expr::Unary(u) => {
            let arg = folder.fold_expr(u.arg, ast);
            ast.alloc(Expr::Unary(UnaryExpr {
                span: u.span,
                op: u.op,
                arg,
            }))
        }
        Expr::UnaryOp(u) => {
            let arg = folder.fold_expr(u.arg, ast);
            ast.alloc(Expr::UnaryOp(UnaryOpExpr {
                span: u.span,
                op: u.op,
                arg,
            }))
        }
        Expr::Binary(b) => {
            let left = folder.fold_expr(b.left, ast);
            let right = folder.fold_expr(b.right, ast);
            ast.alloc(Expr::Binary(BinaryExpr {
                span: b.span,
                op: b.op,
                left,
                right,
            }))
        }
        Expr::Logical(l) => {
            let left = folder.fold_expr(l.left, ast);
            let right = folder.fold_expr(l.right, ast);
            ast.alloc(Expr::Logical(LogicalExpr {
                span: l.span,
                op: l.op,
                left,
                right,
            }))
        }
        Expr::Conditional(c) => {
            let test = folder.fold_expr(c.test, ast);
            let consequent = folder.fold_expr(c.consequent, ast);
            let alternate = folder.fold_expr(c.alternate, ast);
            ast.alloc(Expr::Conditional(ConditionalExpr {
                span: c.span,
                test,
                consequent,
                alternate,
            }))
        }
        Expr::Assignment(a) => {
            let left = folder.fold_expr(a.left, ast);
            let right = folder.fold_expr(a.right, ast);
            ast.alloc(Expr::Assignment(AssignmentExpr {
                span: a.span,
                op: a.op,
                left,
                right,
            }))
        }
        Expr::Sequence(s) => {
            let expressions = s
                .expressions
                .into_iter()
                .map(|e| folder.fold_expr(e, ast))
                .collect();
            ast.alloc(Expr::Sequence(SequenceExpr {
                span: s.span,
                expressions,
            }))
        }
        Expr::Update(u) => {
            let arg = folder.fold_expr(u.arg, ast);
            ast.alloc(Expr::Update(UpdateExpr {
                span: u.span,
                op: u.op,
                arg,
                prefix: u.prefix,
            }))
        }
        Expr::Await(a) => {
            let arg = folder.fold_expr(a.arg, ast);
            ast.alloc(Expr::Await(AwaitExpr { span: a.span, arg }))
        }
        Expr::Yield(y) => {
            let arg = y.arg.map(|a| folder.fold_expr(a, ast));
            ast.alloc(Expr::Yield(YieldExpr {
                span: y.span,
                arg,
                delegate: y.delegate,
            }))
        }
        Expr::Spread(s) => {
            let arg = folder.fold_expr(s.arg, ast);
            ast.alloc(Expr::Spread(SpreadExpr { span: s.span, arg }))
        }
        _ => expr_ref,
    }
}

fn fold_block_stmt(folder: &mut dyn Fold, stmt: BlockStmt, ast: &mut Arena<Expr>) -> BlockStmt {
    BlockStmt {
        span: stmt.span,
        stmts: stmt
            .stmts
            .into_iter()
            .map(|s| folder.fold_stmt(s, ast))
            .collect(),
    }
}

pub fn fold_stmt(folder: &mut dyn Fold, stmt: Stmt, ast: &mut Arena<Expr>) -> Stmt {
    match stmt {
        Stmt::Block(s) => Stmt::Block(fold_block_stmt(folder, s, ast)),
        Stmt::Expr(s) => Stmt::Expr(ExprStmt {
            span: s.span,
            expr: folder.fold_expr(s.expr, ast),
        }),
        Stmt::If(s) => Stmt::If(IfStmt {
            span: s.span,
            test: folder.fold_expr(s.test, ast),
            consequent: Box::new(folder.fold_stmt(*s.consequent, ast)),
            alternate: s.alternate.map(|a| Box::new(folder.fold_stmt(*a, ast))),
        }),
        Stmt::While(s) => Stmt::While(WhileStmt {
            span: s.span,
            test: folder.fold_expr(s.test, ast),
            body: Box::new(folder.fold_stmt(*s.body, ast)),
        }),
        Stmt::DoWhile(s) => Stmt::DoWhile(DoWhileStmt {
            span: s.span,
            body: Box::new(folder.fold_stmt(*s.body, ast)),
            test: folder.fold_expr(s.test, ast),
        }),
        Stmt::For(s) => Stmt::For(ForStmt {
            span: s.span,
            init: s.init.map(|i| match i {
                ForInit::Expr(e) => ForInit::Expr(folder.fold_expr(e, ast)),
                ForInit::Decl(d) => ForInit::Decl(Box::new(folder.fold_decl(*d, ast))),
            }),
            test: s.test.map(|t| folder.fold_expr(t, ast)),
            update: s.update.map(|u| folder.fold_expr(u, ast)),
            body: Box::new(folder.fold_stmt(*s.body, ast)),
        }),
        Stmt::ForIn(s) => Stmt::ForIn(ForInStmt {
            span: s.span,
            left: fold_for_init(folder, s.left, ast),
            right: folder.fold_expr(s.right, ast),
            body: Box::new(folder.fold_stmt(*s.body, ast)),
        }),
        Stmt::ForOf(s) => Stmt::ForOf(ForOfStmt {
            span: s.span,
            left: fold_for_init(folder, s.left, ast),
            right: folder.fold_expr(s.right, ast),
            body: Box::new(folder.fold_stmt(*s.body, ast)),
            await_: s.await_,
        }),
        Stmt::Return(s) => Stmt::Return(ReturnStmt {
            span: s.span,
            arg: s.arg.map(|a| folder.fold_expr(a, ast)),
        }),
        Stmt::Throw(s) => Stmt::Throw(ThrowStmt {
            span: s.span,
            arg: folder.fold_expr(s.arg, ast),
        }),
        Stmt::Try(s) => Stmt::Try(TryStmt {
            span: s.span,
            block: fold_block_stmt(folder, s.block, ast),
            handler: s.handler.map(|h| CatchClause {
                span: h.span,
                param: h.param.map(|p| folder.fold_pat(p, ast)),
                body: fold_block_stmt(folder, h.body, ast),
            }),
            finalizer: s.finalizer.map(|f| fold_block_stmt(folder, f, ast)),
        }),
        Stmt::Switch(s) => Stmt::Switch(SwitchStmt {
            span: s.span,
            discriminant: folder.fold_expr(s.discriminant, ast),
            cases: s
                .cases
                .into_iter()
                .map(|c| SwitchCase {
                    span: c.span,
                    test: c.test.map(|t| folder.fold_expr(t, ast)),
                    consequent: c
                        .consequent
                        .into_iter()
                        .map(|s| folder.fold_stmt(s, ast))
                        .collect(),
                })
                .collect(),
        }),
        Stmt::Labelled(s) => Stmt::Labelled(LabelledStmt {
            span: s.span,
            label: s.label,
            body: Box::new(folder.fold_stmt(*s.body, ast)),
        }),
        Stmt::With(s) => Stmt::With(WithStmt {
            span: s.span,
            object: folder.fold_expr(s.object, ast),
            body: Box::new(folder.fold_stmt(*s.body, ast)),
        }),
        Stmt::Decl(d) => Stmt::Decl(folder.fold_decl(d, ast)),
        other => other,
    }
}

fn fold_for_init(folder: &mut dyn Fold, init: ForInit, ast: &mut Arena<Expr>) -> ForInit {
    match init {
        ForInit::Expr(e) => ForInit::Expr(folder.fold_expr(e, ast)),
        ForInit::Decl(d) => ForInit::Decl(Box::new(folder.fold_decl(*d, ast))),
    }
}

pub fn fold_decl(folder: &mut dyn Fold, decl: Decl, ast: &mut Arena<Expr>) -> Decl {
    match decl {
        Decl::Var(d) => Decl::Var(VarDecl {
            span: d.span,
            kind: d.kind,
            decls: d
                .decls
                .into_iter()
                .map(|v| VarDeclarator {
                    span: v.span,
                    name: folder.fold_pat(v.name, ast),
                    init: v.init.map(|i| folder.fold_expr(i, ast)),
                })
                .collect(),
            await_: d.await_,
        }),
        Decl::Fn(d) => Decl::Fn(FnDecl {
            span: d.span,
            id: d.id,
            params: d
                .params
                .into_iter()
                .map(|p| folder.fold_pat(p, ast))
                .collect(),
            body: d.body.map(|b| fold_block_stmt(folder, b, ast)),
            generator: d.generator,
            async_: d.async_,
            declare: d.declare,
            decorators: d
                .decorators
                .into_iter()
                .map(|dec| Decorator {
                    span: dec.span,
                    expr: folder.fold_expr(dec.expr, ast),
                })
                .collect(),
        }),
        Decl::Class(d) => Decl::Class(ClassDecl {
            span: d.span,
            id: d.id,
            super_class: d.super_class.map(|s| folder.fold_expr(s, ast)),
            body: d.body,
            declare: d.declare,
            abstract_: d.abstract_,
            decorators: d
                .decorators
                .into_iter()
                .map(|dec| Decorator {
                    span: dec.span,
                    expr: folder.fold_expr(dec.expr, ast),
                })
                .collect(),
        }),
        other => other,
    }
}

pub fn fold_pat(folder: &mut dyn Fold, pat: Pat, ast: &mut Arena<Expr>) -> Pat {
    match pat {
        Pat::Array(ap) => {
            let elements = ap
                .elements
                .into_iter()
                .map(|e| e.map(|p| folder.fold_pat(p, ast)))
                .collect();
            let rest = ap.rest.map(|r| {
                let arg = Box::new(folder.fold_pat(*r.arg, ast));
                Box::new(RestPat { span: r.span, arg })
            });
            Pat::Array(ArrayPat {
                span: ap.span,
                elements,
                rest,
            })
        }
        Pat::Object(op) => {
            let props = op
                .props
                .into_iter()
                .map(|p| match p {
                    ObjectPatProp::KeyValue(kv) => {
                        let value = Box::new(folder.fold_pat(*kv.value, ast));
                        ObjectPatProp::KeyValue(KeyValuePatProp {
                            span: kv.span,
                            key: kv.key,
                            value,
                        })
                    }
                    other => other,
                })
                .collect();
            let rest = op.rest.map(|r| {
                let arg = Box::new(folder.fold_pat(*r.arg, ast));
                Box::new(RestPat { span: r.span, arg })
            });
            Pat::Object(ObjectPat {
                span: op.span,
                props,
                rest,
            })
        }
        Pat::Rest(r) => {
            let arg = Box::new(folder.fold_pat(*r.arg, ast));
            Pat::Rest(RestPat { span: r.span, arg })
        }
        Pat::Assign(a) => {
            let left = Box::new(folder.fold_pat(*a.left, ast));
            let right = folder.fold_expr(a.right, ast);
            Pat::Assign(AssignPat {
                span: a.span,
                left,
                right,
            })
        }
        Pat::Expr(e) => Pat::Expr(folder.fold_expr(e, ast)),
        other => other,
    }
}
