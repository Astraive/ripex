use crate::cpp::ast::*;

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
        Decl::Namespace(name, members, _) => visitor.visit_namespace_decl(name, members),
        Decl::Class(c, _) => visitor.visit_class_decl(c),
        Decl::Struct(s, _) => visitor.visit_struct_decl(s),
        Decl::Enum(e, _) => visitor.visit_enum_decl(e),
        _ => {}
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
        Stmt::If(cond, body, alt, _) => {
            visitor.visit_expr(cond);
            visitor.visit_stmt(body);
            if let Some(ref a) = alt {
                visitor.visit_stmt(a);
            }
        }
        Stmt::Switch(test, cases, _) => {
            visitor.visit_expr(test);
            for c in cases {
                if let Some(ref e) = c.expr {
                    visitor.visit_expr(e);
                }
                for s in &c.stmts {
                    visitor.visit_stmt(s);
                }
            }
        }
        Stmt::While(cond, body, _) => {
            visitor.visit_expr(cond);
            visitor.visit_stmt(body);
        }
        Stmt::Do(body, test, _) => {
            visitor.visit_stmt(body);
            visitor.visit_expr(test);
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
        Stmt::RangeFor(_, _, body, _) => {
            visitor.visit_stmt(body);
        }
        Stmt::Return(Some(e), _) => visitor.visit_expr(e),
        Stmt::Throw(Some(e), _) => visitor.visit_expr(e),
        Stmt::Try(body, catches, finalizer, _) => {
            visitor.visit_stmt(body);
            for c in catches {
                visitor.visit_stmt(&c.body);
            }
            if let Some(ref f) = finalizer {
                visitor.visit_stmt(f);
            }
        }
        Stmt::Block(b, _) => visitor.visit_block(b),
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
        Expr::Member(obj, _, _) => visitor.visit_expr(obj),
        Expr::Arrow(obj, _, _) => visitor.visit_expr(obj),
        Expr::Deref(e, _) | Expr::Ref(e, _) => visitor.visit_expr(e),
        Expr::Cast(e, _, _)
        | Expr::DynamicCast(e, _, _)
        | Expr::StaticCast(e, _, _)
        | Expr::ConstCast(e, _, _)
        | Expr::ReinterpretCast(e, _, _) => visitor.visit_expr(e),
        Expr::Sizeof(e, _) | Expr::Alignof(e, _) | Expr::Typeid(e, _) => visitor.visit_expr(e),
        Expr::Ternary(cond, then, els, _) => {
            visitor.visit_expr(cond);
            visitor.visit_expr(then);
            visitor.visit_expr(els);
        }
        Expr::Comma(exprs, _) => {
            for e in exprs {
                visitor.visit_expr(e);
            }
        }
        Expr::Lambda(l, _) => {
            visitor.visit_block(&l.body);
        }
        Expr::New(t, args, _) => {
            visitor.visit_expr(t);
            for a in args {
                visitor.visit_expr(a);
            }
        }
        Expr::Delete(e, _) => visitor.visit_expr(e),
        Expr::Paren(e, _) => visitor.visit_expr(e),
        Expr::Assign(left, right, _) => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        Expr::Template(t, args, _) => {
            visitor.visit_expr(t);
            for a in args {
                visitor.visit_expr(a);
            }
        }
        Expr::BraceInit(exprs, _) => {
            for e in exprs {
                visitor.visit_expr(e);
            }
        }
        _ => {}
    }
}
