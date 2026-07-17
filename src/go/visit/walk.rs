use crate::go::ast::*;

use super::visitor::Visitor;

pub fn walk_program(visitor: &mut dyn Visitor, program: &Program) {
    for decl in &program.decls {
        visitor.visit_decl(decl);
    }
}

pub fn walk_decl(visitor: &mut dyn Visitor, decl: &Decl) {
    match decl {
        Decl::Func(f, _) => visitor.visit_func_decl(f),
        Decl::Var(v, _) => visitor.visit_var_decl(v),
        Decl::Const(c, _) => visitor.visit_const_decl(c),
        Decl::Type(t, _) => visitor.visit_type_decl(t),
        Decl::Import(i, _) => visitor.visit_import_decl(i),
        Decl::ImportGroup(imports, _) => {
            for import in imports {
                visitor.visit_import_decl(import);
            }
        }
        Decl::Package(_, _) => {}
    }
}

pub fn walk_block(visitor: &mut dyn Visitor, block: &Block) {
    for s in &block.stmts {
        visitor.visit_stmt(s);
    }
}

pub fn walk_stmt(visitor: &mut dyn Visitor, stmt: &Stmt) {
    match stmt {
        Stmt::Expr(e, _) => visitor.visit_expr(e),
        Stmt::Decl(d, _) => visitor.visit_decl(d),
        Stmt::Assign(targets, _, _) => visitor.visit_assign_stmt(targets),
        Stmt::If(cond, body, alt, _) => {
            visitor.visit_expr(cond);
            visitor.visit_stmt(body);
            if let Some(ref a) = alt {
                visitor.visit_stmt(a);
            }
        }
        Stmt::For(init, test, update, body, _) => {
            if let Some(ref i) = init {
                visitor.visit_stmt(i);
            }
            if let Some(ref t) = test {
                visitor.visit_expr(t);
            }
            if let Some(ref u) = update {
                visitor.visit_stmt(u);
            }
            visitor.visit_stmt(body);
        }
        Stmt::ForRange(_, _key, _, body, _) => {
            visitor.visit_stmt(body);
        }
        Stmt::Switch(test, cases, _) => {
            if let Some(ref t) = test {
                visitor.visit_expr(t);
            }
            for c in cases {
                if let Some(ref e) = c.expr {
                    visitor.visit_expr(e);
                }
                for s in &c.body {
                    visitor.visit_stmt(s);
                }
            }
        }
        Stmt::Return(vals, _) => {
            for v in vals {
                visitor.visit_expr(v);
            }
        }
        Stmt::Block(b, _) => visitor.visit_block(b),
        Stmt::Defer(e, _) => visitor.visit_expr(e),
        Stmt::Go(e, _) => visitor.visit_expr(e),
        Stmt::Send(ch, val, _) => {
            visitor.visit_expr(ch);
            visitor.visit_expr(val);
        }
        Stmt::Label(_, s, _) => visitor.visit_stmt(s),
        _ => {}
    }
}

pub fn walk_expr(visitor: &mut dyn Visitor, expr: &Expr) {
    match expr {
        Expr::Binary(left, _, right, _) => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        Expr::Unary(_, operand, _) => visitor.visit_expr(operand),
        Expr::Call(func, args, _) => {
            visitor.visit_expr(func);
            for a in args {
                visitor.visit_expr(a);
            }
        }
        Expr::Index(arr, idx, _) => {
            visitor.visit_expr(arr);
            visitor.visit_expr(idx);
        }
        Expr::Selector(obj, _, _) => visitor.visit_expr(obj),
        Expr::Slice(arr, low, high, _) => {
            visitor.visit_expr(arr);
            if let Some(ref l) = low {
                visitor.visit_expr(l);
            }
            if let Some(ref h) = high {
                visitor.visit_expr(h);
            }
        }
        Expr::Array(elems, _) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        Expr::StructLit(_, fields, _) => {
            for f in fields {
                if let Some(ref v) = f.value {
                    visitor.visit_expr(v);
                }
            }
        }
        Expr::MapLit(entries, _) => {
            for (k, v) in entries {
                visitor.visit_expr(k);
                visitor.visit_expr(v);
            }
        }
        Expr::FuncLit(_, body, _) => visitor.visit_block(body),
        Expr::Paren(e, _) => visitor.visit_expr(e),
        Expr::TypeAssert(e, _, _) => visitor.visit_expr(e),
        Expr::CompositeLit(t, elems, _) => {
            visitor.visit_expr(t);
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        _ => {}
    }
}
