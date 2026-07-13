use crate::c::ast::*;

use super::visitor::Visitor;

pub fn walk_program(visitor: &mut dyn Visitor, program: &Program) {
    for stmt in &program.decls {
        visitor.visit_stmt(stmt);
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
        Stmt::Decl(f, _) => visitor.visit_func_decl(f),
        Stmt::VarDecl(v, _) => visitor.visit_var_decl(v),
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
        Stmt::Return(Some(e), _) => visitor.visit_expr(e),
        Stmt::Block(b, _) => visitor.visit_block(b),
        Stmt::Label(_, _) | Stmt::Goto(_, _) | Stmt::Break(_) | Stmt::Continue(_) => {}
        Stmt::Preprocessor(_, _) => {}
        Stmt::Return(None, _) | Stmt::Empty(_) => {}
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
        Expr::Deref(e, _) => visitor.visit_expr(e),
        Expr::Ref(e, _) => visitor.visit_expr(e),
        Expr::Cast(e, _, _) => visitor.visit_expr(e),
        Expr::Sizeof(e, _) => visitor.visit_expr(e),
        Expr::Alignof(e, _) => visitor.visit_expr(e),
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
        Expr::StmtExpr(stmts, _) => {
            for s in stmts {
                visitor.visit_stmt(s);
            }
        }
        Expr::Paren(e, _) => visitor.visit_expr(e),
        Expr::Assign(left, right, _) => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        _ => {}
    }
}
