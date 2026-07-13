use crate::rust::ast::*;

use super::visitor::Visitor;

pub fn walk_program(visitor: &mut dyn Visitor, program: &Program) {
    for item in &program.items {
        visitor.visit_item(item);
    }
}

pub fn walk_item(visitor: &mut dyn Visitor, item: &Item) {
    match item {
        Item::Fn(f, _) => visitor.visit_fn_decl(f),
        Item::Struct(s, _) => visitor.visit_struct_decl(s),
        Item::Enum(e, _) => visitor.visit_enum_decl(e),
        Item::Trait(t, _) => visitor.visit_trait_decl(t),
        Item::Mod(m, _) => {
            for i in &m.items {
                visitor.visit_item(i);
            }
        }
        Item::Impl(imp, _) => {
            for m in &imp.methods {
                visitor.visit_fn_decl(m);
            }
        }
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
        Stmt::Item(item, _) => visitor.visit_item(item),
        Stmt::Let(l, _) => {
            visitor.visit_pattern(&l.pattern);
            if let Some(ref init) = l.init {
                visitor.visit_expr(init);
            }
        }
        Stmt::Empty(_) => {}
    }
}

pub fn walk_pattern(visitor: &mut dyn Visitor, pat: &Pattern) {
    match pat {
        Pattern::Ident(_, _) | Pattern::Wildcard(_) | Pattern::Lit(_, _) | Pattern::Rest(_) => {}
        Pattern::Tuple(pats, _) => {
            for p in pats {
                visitor.visit_pattern(p);
            }
        }
        Pattern::Struct(_, fields, _) => {
            for f in fields {
                visitor.visit_pattern(&f.pattern);
            }
        }
        Pattern::Range(low, high, _) => {
            visitor.visit_pattern(low);
            visitor.visit_pattern(high);
        }
        Pattern::Or(pats, _) => {
            for p in pats {
                visitor.visit_pattern(p);
            }
        }
        Pattern::Ref(pat, _, _) => visitor.visit_pattern(pat),
        Pattern::Slice(pats, _) => {
            for p in pats {
                visitor.visit_pattern(p);
            }
        }
    }
}

pub fn walk_expr(visitor: &mut dyn Visitor, expr: &Expr) {
    match expr {
        Expr::Binary(left, _, right, _) => {
            visitor.visit_expr(left);
            visitor.visit_expr(right);
        }
        Expr::Unary(_, operand, _) => visitor.visit_expr(operand),
        Expr::Call(callee, args, _) => {
            visitor.visit_expr(callee);
            for a in args {
                visitor.visit_expr(a);
            }
        }
        Expr::MethodCall(obj, _, args, _) => {
            visitor.visit_expr(obj);
            for a in args {
                visitor.visit_expr(a);
            }
        }
        Expr::Index(arr, idx, _) => {
            visitor.visit_expr(arr);
            visitor.visit_expr(idx);
        }
        Expr::Field(obj, _, _) => visitor.visit_expr(obj),
        Expr::Tuple(elems, _) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        Expr::Array(elems, _) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        Expr::Struct(_, fields, base, _) => {
            for f in fields {
                if let Some(ref v) = f.value {
                    visitor.visit_expr(v);
                }
            }
            if let Some(ref b) = base {
                visitor.visit_expr(b);
            }
        }
        Expr::Closure(_, body, _) => visitor.visit_expr(body),
        Expr::Block(b, _) => visitor.visit_block(b),
        Expr::If(cond, body, alt, _) => {
            visitor.visit_expr(cond);
            visitor.visit_block(body);
            if let Some(ref a) = alt {
                visitor.visit_expr(a);
            }
        }
        Expr::Match(expr_, arms, _) => {
            visitor.visit_expr(expr_);
            for a in arms {
                visitor.visit_expr(&a.body);
            }
        }
        Expr::While(cond, body, _) => {
            visitor.visit_expr(cond);
            visitor.visit_block(body);
        }
        Expr::Loop(body, _) => visitor.visit_block(body),
        Expr::For(pat, iter, body, _) => {
            visitor.visit_pattern(pat);
            visitor.visit_expr(iter);
            visitor.visit_block(body);
        }
        Expr::Return(Some(e), _) | Expr::Break(Some(e), _) => visitor.visit_expr(e),
        Expr::Paren(e, _) => visitor.visit_expr(e),
        Expr::Async(e, _) | Expr::Await(e, _) | Expr::Ref(e, _, _) | Expr::Deref(e, _) => {
            visitor.visit_expr(e)
        }
        Expr::Cast(e, _, _) => visitor.visit_expr(e),
        _ => {}
    }
}
