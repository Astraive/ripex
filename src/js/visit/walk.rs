use crate::arena::Arena;
use crate::js::ast::*;

use super::visitor::Visitor;

pub fn walk_program(visitor: &mut dyn Visitor, program: &Program, ast: &Arena<Expr>) {
    match program {
        Program::Script(script) => visitor.visit_script(script, ast),
        Program::Module(module) => visitor.visit_module(module, ast),
    }
}

pub fn walk_script(visitor: &mut dyn Visitor, script: &Script, ast: &Arena<Expr>) {
    for stmt in &script.body {
        visitor.visit_stmt(stmt, ast);
    }
}

pub fn walk_module(visitor: &mut dyn Visitor, module: &Module, ast: &Arena<Expr>) {
    for item in &module.body {
        visitor.visit_module_item(item, ast);
    }
}

pub fn walk_module_item(visitor: &mut dyn Visitor, item: &ModuleItem, ast: &Arena<Expr>) {
    match item {
        ModuleItem::Stmt(stmt) => visitor.visit_stmt(stmt, ast),
        ModuleItem::Decl(decl) => visitor.visit_decl(decl, ast),
        ModuleItem::Import(imp) => visitor.visit_import_decl(imp, ast),
        ModuleItem::Export(exp) => visitor.visit_export_decl(exp, ast),
    }
}

pub fn walk_export_decl(_visitor: &mut dyn Visitor, _decl: &ExportDecl, _ast: &Arena<Expr>) {}

pub fn walk_stmt(visitor: &mut dyn Visitor, stmt: &Stmt, ast: &Arena<Expr>) {
    match stmt {
        Stmt::Block(b) => visitor.visit_block_stmt(b, ast),
        Stmt::Expr(e) => visitor.visit_expr(e.expr, ast),
        Stmt::If(s) => {
            visitor.visit_expr(s.test, ast);
            visitor.visit_stmt(&s.consequent, ast);
            if let Some(ref alt) = s.alternate {
                visitor.visit_stmt(alt, ast);
            }
        }
        Stmt::While(s) => {
            visitor.visit_expr(s.test, ast);
            visitor.visit_stmt(&s.body, ast);
        }
        Stmt::DoWhile(s) => {
            visitor.visit_stmt(&s.body, ast);
            visitor.visit_expr(s.test, ast);
        }
        Stmt::For(s) => {
            if let Some(ref init) = s.init {
                match init {
                    ForInit::Expr(e) => visitor.visit_expr(*e, ast),
                    ForInit::Decl(d) => visitor.visit_decl(d, ast),
                }
            }
            if let Some(ref test) = s.test {
                visitor.visit_expr(*test, ast);
            }
            if let Some(ref update) = s.update {
                visitor.visit_expr(*update, ast);
            }
            visitor.visit_stmt(&s.body, ast);
        }
        Stmt::ForIn(s) => {
            if let ForInit::Expr(ref e) = s.left {
                visitor.visit_expr(*e, ast);
            }
            visitor.visit_expr(s.right, ast);
            visitor.visit_stmt(&s.body, ast);
        }
        Stmt::ForOf(s) => {
            visitor.visit_expr(s.right, ast);
            visitor.visit_stmt(&s.body, ast);
        }
        Stmt::Return(s) => {
            if let Some(ref arg) = s.arg {
                visitor.visit_expr(*arg, ast);
            }
        }
        Stmt::Throw(s) => visitor.visit_expr(s.arg, ast),
        Stmt::Try(s) => {
            visitor.visit_block_stmt(&s.block, ast);
            if let Some(ref handler) = s.handler {
                if let Some(ref param) = handler.param {
                    visitor.visit_pat(param, ast);
                }
                visitor.visit_block_stmt(&handler.body, ast);
            }
            if let Some(ref finalizer) = s.finalizer {
                visitor.visit_block_stmt(finalizer, ast);
            }
        }
        Stmt::Switch(s) => {
            visitor.visit_expr(s.discriminant, ast);
            for case in &s.cases {
                if let Some(ref test) = case.test {
                    visitor.visit_expr(*test, ast);
                }
                for stmt in &case.consequent {
                    visitor.visit_stmt(stmt, ast);
                }
            }
        }
        Stmt::Labelled(s) => visitor.visit_stmt(&s.body, ast),
        Stmt::With(s) => {
            visitor.visit_expr(s.object, ast);
            visitor.visit_stmt(&s.body, ast);
        }
        Stmt::Break(_) | Stmt::Continue(_) | Stmt::Debugger(_) | Stmt::Empty(_) => {}
        Stmt::Decl(d) => visitor.visit_decl(d, ast),
    }
}

pub fn walk_block_stmt(visitor: &mut dyn Visitor, stmt: &BlockStmt, ast: &Arena<Expr>) {
    for s in &stmt.stmts {
        visitor.visit_stmt(s, ast);
    }
}

pub fn walk_if_stmt(visitor: &mut dyn Visitor, stmt: &IfStmt, ast: &Arena<Expr>) {
    visitor.visit_expr(stmt.test, ast);
    visitor.visit_stmt(&stmt.consequent, ast);
    if let Some(ref alt) = stmt.alternate {
        visitor.visit_stmt(alt, ast);
    }
}

pub fn walk_while_stmt(visitor: &mut dyn Visitor, stmt: &WhileStmt, ast: &Arena<Expr>) {
    visitor.visit_expr(stmt.test, ast);
    visitor.visit_stmt(&stmt.body, ast);
}

pub fn walk_do_while_stmt(visitor: &mut dyn Visitor, stmt: &DoWhileStmt, ast: &Arena<Expr>) {
    visitor.visit_stmt(&stmt.body, ast);
    visitor.visit_expr(stmt.test, ast);
}

pub fn walk_for_stmt(visitor: &mut dyn Visitor, stmt: &ForStmt, ast: &Arena<Expr>) {
    if let Some(ref init) = stmt.init {
        match init {
            ForInit::Expr(e) => visitor.visit_expr(*e, ast),
            ForInit::Decl(d) => visitor.visit_decl(d, ast),
        }
    }
    if let Some(ref test) = stmt.test {
        visitor.visit_expr(*test, ast);
    }
    if let Some(ref update) = stmt.update {
        visitor.visit_expr(*update, ast);
    }
    visitor.visit_stmt(&stmt.body, ast);
}

pub fn walk_for_in_stmt(visitor: &mut dyn Visitor, stmt: &ForInStmt, ast: &Arena<Expr>) {
    if let ForInit::Expr(ref e) = stmt.left {
        visitor.visit_expr(*e, ast);
    }
    visitor.visit_expr(stmt.right, ast);
    visitor.visit_stmt(&stmt.body, ast);
}

pub fn walk_for_of_stmt(visitor: &mut dyn Visitor, stmt: &ForOfStmt, ast: &Arena<Expr>) {
    visitor.visit_expr(stmt.right, ast);
    visitor.visit_stmt(&stmt.body, ast);
}

pub fn walk_return_stmt(visitor: &mut dyn Visitor, stmt: &ReturnStmt, ast: &Arena<Expr>) {
    if let Some(ref arg) = stmt.arg {
        visitor.visit_expr(*arg, ast);
    }
}

pub fn walk_switch_stmt(visitor: &mut dyn Visitor, stmt: &SwitchStmt, ast: &Arena<Expr>) {
    visitor.visit_expr(stmt.discriminant, ast);
    for case in &stmt.cases {
        if let Some(ref test) = case.test {
            visitor.visit_expr(*test, ast);
        }
        for s in &case.consequent {
            visitor.visit_stmt(s, ast);
        }
    }
}

pub fn walk_throw_stmt(visitor: &mut dyn Visitor, stmt: &ThrowStmt, ast: &Arena<Expr>) {
    visitor.visit_expr(stmt.arg, ast);
}

pub fn walk_try_stmt(visitor: &mut dyn Visitor, stmt: &TryStmt, ast: &Arena<Expr>) {
    visitor.visit_block_stmt(&stmt.block, ast);
    if let Some(ref handler) = stmt.handler {
        if let Some(ref param) = handler.param {
            visitor.visit_pat(param, ast);
        }
        visitor.visit_block_stmt(&handler.body, ast);
    }
    if let Some(ref finalizer) = stmt.finalizer {
        visitor.visit_block_stmt(finalizer, ast);
    }
}

pub fn walk_labelled_stmt(visitor: &mut dyn Visitor, stmt: &LabelledStmt, ast: &Arena<Expr>) {
    visitor.visit_stmt(&stmt.body, ast);
}

pub fn walk_with_stmt(visitor: &mut dyn Visitor, stmt: &WithStmt, ast: &Arena<Expr>) {
    visitor.visit_expr(stmt.object, ast);
    visitor.visit_stmt(&stmt.body, ast);
}

pub fn walk_decl(visitor: &mut dyn Visitor, decl: &Decl, ast: &Arena<Expr>) {
    match decl {
        Decl::Var(d) => visitor.visit_var_decl(d, ast),
        Decl::Fn(d) => visitor.visit_fn_decl(d, ast),
        Decl::Class(d) => visitor.visit_class_decl(d, ast),
        _ => {}
    }
}

pub fn walk_var_decl(visitor: &mut dyn Visitor, decl: &VarDecl, ast: &Arena<Expr>) {
    for d in &decl.decls {
        visitor.visit_pat(&d.name, ast);
        if let Some(ref init) = d.init {
            visitor.visit_expr(*init, ast);
        }
    }
}

pub fn walk_fn_decl(visitor: &mut dyn Visitor, decl: &FnDecl, ast: &Arena<Expr>) {
    for param in &decl.params {
        visitor.visit_pat(param, ast);
    }
    if let Some(ref body) = decl.body {
        visitor.visit_block_stmt(body, ast);
    }
}

pub fn walk_class_decl(_visitor: &mut dyn Visitor, _decl: &ClassDecl, _ast: &Arena<Expr>) {}

pub fn walk_expr(visitor: &mut dyn Visitor, expr_ref: ExprRef, ast: &Arena<Expr>) {
    let expr = &ast[expr_ref];
    match expr {
        Expr::Ident(i) => visitor.visit_ident(i, ast),
        Expr::Lit(l) => visitor.visit_lit(l, ast),
        Expr::This(_)
        | Expr::Super(_)
        | Expr::MetaProperty(_)
        | Expr::PrivateName(_)
        | Expr::Invalid(_) => {}
        Expr::Unary(u) => {
            visitor.visit_unary_expr(u, ast);
            visitor.visit_expr(u.arg, ast);
        }
        Expr::UnaryOp(u) => visitor.visit_expr(u.arg, ast),
        Expr::Binary(b) => {
            visitor.visit_binary_expr(b, ast);
            visitor.visit_expr(b.left, ast);
            visitor.visit_expr(b.right, ast);
        }
        Expr::Logical(l) => {
            visitor.visit_expr(l.left, ast);
            visitor.visit_expr(l.right, ast);
        }
        Expr::Conditional(c) => {
            visitor.visit_cond_expr(c, ast);
            visitor.visit_expr(c.test, ast);
            visitor.visit_expr(c.consequent, ast);
            visitor.visit_expr(c.alternate, ast);
        }
        Expr::Assignment(a) => {
            visitor.visit_assign_expr(a, ast);
            visitor.visit_expr(a.left, ast);
            visitor.visit_expr(a.right, ast);
        }
        Expr::Sequence(s) => {
            for e in &s.expressions {
                visitor.visit_expr(*e, ast);
            }
        }
        Expr::Update(u) => {
            visitor.visit_update_expr(u, ast);
            visitor.visit_expr(u.arg, ast);
        }
        Expr::Await(a) => {
            visitor.visit_await_expr(a, ast);
            visitor.visit_expr(a.arg, ast);
        }
        Expr::Yield(y) => {
            visitor.visit_yield_expr(y, ast);
            if let Some(ref arg) = y.arg {
                visitor.visit_expr(*arg, ast);
            }
        }
        Expr::Spread(s) => visitor.visit_expr(s.arg, ast),
        Expr::Call(c) => {
            visitor.visit_call_expr(c, ast);
            visitor.visit_expr(c.callee, ast);
            for arg in &c.args {
                visitor.visit_expr(*arg, ast);
            }
        }
        Expr::New(n) => {
            visitor.visit_new_expr(n, ast);
            visitor.visit_expr(n.callee, ast);
            for arg in &n.args {
                visitor.visit_expr(*arg, ast);
            }
        }
        Expr::Member(m) => {
            visitor.visit_member_expr(m, ast);
            visitor.visit_expr(m.object, ast);
        }
        Expr::Array(a) => {
            visitor.visit_array_expr(a, ast);
            for e in a.elements.iter().flatten() {
                visitor.visit_expr(*e, ast);
            }
        }
        Expr::Object(o) => {
            visitor.visit_object_expr(o, ast);
            for prop in &o.props {
                match prop {
                    ObjProp::KeyValue(kv) => visitor.visit_expr(kv.value, ast),
                    ObjProp::Shorthand(_) => {}
                    ObjProp::Method(m) => {
                        if let Some(ref body) = m.function.body {
                            visitor.visit_block_stmt(body, ast);
                        }
                    }
                    ObjProp::Spread(s) => visitor.visit_expr(s.arg, ast),
                    ObjProp::Getter(g) => {
                        if let Some(ref body) = g.body {
                            visitor.visit_block_stmt(body, ast);
                        }
                    }
                    ObjProp::Setter(s) => {
                        if let Some(ref body) = s.body {
                            visitor.visit_block_stmt(body, ast);
                        }
                    }
                }
            }
        }
        Expr::Fn(f) => {
            visitor.visit_fn_expr(f, ast);
            for param in &f.params {
                visitor.visit_pat(param, ast);
            }
            if let Some(ref body) = f.body {
                visitor.visit_block_stmt(body, ast);
            }
        }
        Expr::Arrow(a) => {
            visitor.visit_arrow_expr(a, ast);
            match &a.body {
                ArrowBody::Expr(e) => visitor.visit_expr(*e, ast),
                ArrowBody::Block(b) => visitor.visit_block_stmt(b, ast),
            }
        }
        Expr::Class(c) => {
            visitor.visit_class_expr(c, ast);
        }
        Expr::Template(t) => {
            for e in &t.expressions {
                visitor.visit_expr(*e, ast);
            }
        }
        Expr::TaggedTemplate(t) => {
            visitor.visit_expr(t.tag, ast);
        }
        Expr::Import(i) => visitor.visit_expr(i.source, ast),
        Expr::Parenthesized(p) => visitor.visit_expr(p.expr, ast),
        Expr::Chain(c) => visitor.visit_expr(c.expr, ast),
        Expr::TSAs(e) => visitor.visit_expr(e.expr, ast),
        Expr::TSSatisfies(e) => visitor.visit_expr(e.expr, ast),
        Expr::TSTypeAssertion(e) => visitor.visit_expr(e.expr, ast),
        Expr::TSNonNull(e) => visitor.visit_expr(e.expr, ast),
        Expr::TSInst(e) => visitor.visit_expr(e.expr, ast),
        Expr::OptionalCall(c) => {
            visitor.visit_expr(c.callee, ast);
            for arg in &c.args {
                visitor.visit_expr(*arg, ast);
            }
        }
        Expr::OptionalMember(m) => {
            visitor.visit_expr(m.object, ast);
        }
        Expr::Record(r) => {
            for prop in &r.props {
                match prop {
                    ObjProp::KeyValue(kv) => visitor.visit_expr(kv.value, ast),
                    ObjProp::Method(m) => {
                        if let Some(ref body) = m.function.body {
                            visitor.visit_block_stmt(body, ast);
                        }
                    }
                    ObjProp::Getter(g) => {
                        if let Some(ref body) = g.body {
                            visitor.visit_block_stmt(body, ast);
                        }
                    }
                    ObjProp::Setter(s) => {
                        if let Some(ref body) = s.body {
                            visitor.visit_block_stmt(body, ast);
                        }
                    }
                    ObjProp::Shorthand(_) | ObjProp::Spread(_) => {}
                }
            }
        }
        Expr::Tuple(t) => {
            for e in t.elements.iter().flatten() {
                visitor.visit_expr(*e, ast);
            }
        }
        Expr::Pipeline(p) => {
            visitor.visit_expr(p.input, ast);
            visitor.visit_expr(p.body, ast);
        }
        Expr::JSXElement(_) | Expr::JSXFragment(_) => {}
    }
}

pub fn walk_unary_expr(visitor: &mut dyn Visitor, expr: &UnaryExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.arg, ast);
}

pub fn walk_binary_expr(visitor: &mut dyn Visitor, expr: &BinaryExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.left, ast);
    visitor.visit_expr(expr.right, ast);
}

pub fn walk_cond_expr(visitor: &mut dyn Visitor, expr: &ConditionalExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.test, ast);
    visitor.visit_expr(expr.consequent, ast);
    visitor.visit_expr(expr.alternate, ast);
}

pub fn walk_call_expr(visitor: &mut dyn Visitor, expr: &CallExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.callee, ast);
    for arg in &expr.args {
        visitor.visit_expr(*arg, ast);
    }
}

pub fn walk_new_expr(visitor: &mut dyn Visitor, expr: &NewExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.callee, ast);
    for arg in &expr.args {
        visitor.visit_expr(*arg, ast);
    }
}

pub fn walk_member_expr(visitor: &mut dyn Visitor, expr: &MemberExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.object, ast);
}

pub fn walk_array_expr(visitor: &mut dyn Visitor, expr: &ArrayExpr, ast: &Arena<Expr>) {
    for e in expr.elements.iter().flatten() {
        visitor.visit_expr(*e, ast);
    }
}

pub fn walk_object_expr(visitor: &mut dyn Visitor, expr: &ObjectExpr, ast: &Arena<Expr>) {
    for prop in &expr.props {
        match prop {
            ObjProp::KeyValue(kv) => visitor.visit_expr(kv.value, ast),
            ObjProp::Spread(s) => visitor.visit_expr(s.arg, ast),
            ObjProp::Method(m) => {
                if let Some(ref body) = m.function.body {
                    visitor.visit_block_stmt(body, ast);
                }
            }
            _ => {}
        }
    }
}

pub fn walk_fn_expr(visitor: &mut dyn Visitor, expr: &FnExpr, ast: &Arena<Expr>) {
    for param in &expr.params {
        visitor.visit_pat(param, ast);
    }
    if let Some(ref body) = expr.body {
        visitor.visit_block_stmt(body, ast);
    }
}

pub fn walk_arrow_expr(visitor: &mut dyn Visitor, expr: &ArrowExpr, ast: &Arena<Expr>) {
    match &expr.body {
        ArrowBody::Expr(e) => visitor.visit_expr(*e, ast),
        ArrowBody::Block(b) => visitor.visit_block_stmt(b, ast),
    }
}

pub fn walk_class_expr(_visitor: &mut dyn Visitor, _expr: &ClassExpr, _ast: &Arena<Expr>) {}

pub fn walk_assign_expr(visitor: &mut dyn Visitor, expr: &AssignmentExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.left, ast);
    visitor.visit_expr(expr.right, ast);
}

pub fn walk_update_expr(visitor: &mut dyn Visitor, expr: &UpdateExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.arg, ast);
}

pub fn walk_yield_expr(visitor: &mut dyn Visitor, expr: &YieldExpr, ast: &Arena<Expr>) {
    if let Some(ref arg) = expr.arg {
        visitor.visit_expr(*arg, ast);
    }
}

pub fn walk_await_expr(visitor: &mut dyn Visitor, expr: &AwaitExpr, ast: &Arena<Expr>) {
    visitor.visit_expr(expr.arg, ast);
}

pub fn walk_pat(visitor: &mut dyn Visitor, pat: &Pat, ast: &Arena<Expr>) {
    match pat {
        Pat::Ident(i) => visitor.visit_binding_ident(i, ast),
        Pat::Object(o) => {
            for prop in &o.props {
                match prop {
                    ObjectPatProp::KeyValue(kv) => visitor.visit_pat(&kv.value, ast),
                    ObjectPatProp::Shorthand(i) => visitor.visit_binding_ident(i, ast),
                    ObjectPatProp::Rest(r) => visitor.visit_pat(&r.arg, ast),
                }
            }
            if let Some(ref rest) = o.rest {
                visitor.visit_pat(&rest.arg, ast);
            }
        }
        Pat::Array(a) => {
            for e in a.elements.iter().flatten() {
                visitor.visit_pat(e, ast);
            }
            if let Some(ref rest) = a.rest {
                visitor.visit_pat(&rest.arg, ast);
            }
        }
        Pat::Rest(r) => visitor.visit_pat(&r.arg, ast),
        Pat::Assign(a) => {
            visitor.visit_pat(&a.left, ast);
            visitor.visit_expr(a.right, ast);
        }
        Pat::Expr(e) => visitor.visit_expr(*e, ast),
        Pat::Invalid(_) => {}
    }
}
