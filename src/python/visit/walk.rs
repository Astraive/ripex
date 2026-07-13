use crate::python::ast::*;

use super::visitor::Visitor;

pub fn walk_program(visitor: &mut dyn Visitor, program: &Program) {
    for stmt in &program.stmts {
        visitor.visit_stmt(stmt);
    }
}

pub fn walk_func_def(visitor: &mut dyn Visitor, def: &FuncDef) {
    for d in &def.decorators {
        visitor.visit_expr(d);
    }
    for a in &def.args {
        if let Some(ref ann) = a.type_ann {
            visitor.visit_expr(ann);
        }
    }
    for s in &def.body {
        visitor.visit_stmt(s);
    }
}

pub fn walk_class_def(visitor: &mut dyn Visitor, def: &ClassDef) {
    for d in &def.decorators {
        visitor.visit_expr(d);
    }
    for b in &def.bases {
        visitor.visit_expr(b);
    }
    for s in &def.body {
        visitor.visit_stmt(s);
    }
}

pub fn walk_stmt(visitor: &mut dyn Visitor, stmt: &Stmt) {
    match stmt {
        Stmt::Expr(e, _) => visitor.visit_expr(e),
        Stmt::Assign(target, value, _) => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
        Stmt::AugAssign(target, _, value, _) => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
        Stmt::AnnAssign(target, ann, value, _) => {
            visitor.visit_expr(target);
            visitor.visit_expr(ann);
            if let Some(ref v) = value {
                visitor.visit_expr(v);
            }
        }
        Stmt::If(cond, body, else_, _) => {
            visitor.visit_expr(cond);
            for s in body {
                visitor.visit_stmt(s);
            }
            for s in else_ {
                visitor.visit_stmt(s);
            }
        }
        Stmt::While(cond, body, _else_, _) => {
            visitor.visit_expr(cond);
            for s in body {
                visitor.visit_stmt(s);
            }
        }
        Stmt::For(target, iter, body, else_, _) => {
            visitor.visit_expr(target);
            visitor.visit_expr(iter);
            for s in body {
                visitor.visit_stmt(s);
            }
            for s in else_.iter().flat_map(|v| v.iter()) {
                visitor.visit_stmt(s);
            }
        }
        Stmt::With(items, body, _) => {
            for i in items {
                visitor.visit_expr(&i.context);
            }
            for s in body {
                visitor.visit_stmt(s);
            }
        }
        Stmt::Match(subj, cases, _) => {
            visitor.visit_expr(subj);
            for c in cases {
                visitor.visit_pattern(&c.pattern);
                for s in &c.body {
                    visitor.visit_stmt(s);
                }
            }
        }
        Stmt::Return(Some(e), _) => visitor.visit_expr(e),
        Stmt::Yield(Some(e), _) => visitor.visit_expr(e),
        Stmt::Raise(Some(e), _, _) => visitor.visit_expr(e),
        Stmt::Assert(test, _, _) => visitor.visit_expr(test),
        Stmt::Delete(e, _) => visitor.visit_expr(e),
        Stmt::Import(_aliases, _) | Stmt::ImportFrom(_, _aliases, _, _) => {}
        Stmt::Try(body, handlers, else_, finalizer, _) => {
            for s in body {
                visitor.visit_stmt(s);
            }
            for h in handlers {
                for s in &h.body {
                    visitor.visit_stmt(s);
                }
            }
            if let Some(ref e) = else_ {
                for s in e {
                    visitor.visit_stmt(s);
                }
            }
            if let Some(ref f) = finalizer {
                for s in f {
                    visitor.visit_stmt(s);
                }
            }
        }
        Stmt::FuncDef(f, _) => visitor.visit_func_def(f),
        Stmt::ClassDef(c, _) => visitor.visit_class_def(c),
        Stmt::Async(stmt, _) => visitor.visit_stmt(stmt),
        Stmt::Block(body, _) => {
            for s in body {
                visitor.visit_stmt(s);
            }
        }
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
        Expr::Call(func, args, keywords, _) => {
            visitor.visit_expr(func);
            for a in args {
                visitor.visit_expr(a);
            }
            for k in keywords {
                visitor.visit_expr(&k.value);
            }
        }
        Expr::IfElse(cond, then, else_, _) => {
            visitor.visit_expr(cond);
            visitor.visit_expr(then);
            visitor.visit_expr(else_);
        }
        Expr::Lambda(_params, body, _) => visitor.visit_expr(body),
        Expr::List(elems, _) | Expr::Tuple(elems, _) | Expr::Set(elems, _) => {
            for e in elems {
                visitor.visit_expr(e);
            }
        }
        Expr::Dict(entries, _) => {
            for (k, v) in entries {
                visitor.visit_expr(k);
                visitor.visit_expr(v);
            }
        }
        Expr::ListComp(elt, gens, _)
        | Expr::SetComp(elt, gens, _)
        | Expr::Generator(elt, gens, _) => {
            visitor.visit_expr(elt);
            for g in gens {
                visitor.visit_expr(&g.target);
                visitor.visit_expr(&g.iter);
            }
        }
        Expr::DictComp(elt, gens, _) => {
            visitor.visit_expr(elt);
            for g in gens {
                visitor.visit_expr(&g.target);
                visitor.visit_expr(&g.iter);
            }
        }
        Expr::Await(e, _) | Expr::Yield(Some(e), _) | Expr::YieldFrom(e, _) => {
            visitor.visit_expr(e)
        }
        Expr::Starred(e, _) => visitor.visit_expr(e),
        Expr::Walrus(target, value, _) => {
            visitor.visit_expr(target);
            visitor.visit_expr(value);
        }
        Expr::FString(parts, _) => {
            for p in parts {
                if let FStringPart::Expr(e, _) = p {
                    visitor.visit_expr(e);
                }
            }
        }
        Expr::Compare(left, _, comparators, _) => {
            visitor.visit_expr(left);
            for c in comparators {
                visitor.visit_expr(c);
            }
        }
        Expr::Paren(e, _) => visitor.visit_expr(e),
        Expr::Attribute(obj, _, _) | Expr::Subscript(obj, _, _) => visitor.visit_expr(obj),
        Expr::Slice(lower, upper, step, _) => {
            if let Some(ref l) = lower {
                visitor.visit_expr(l);
            }
            if let Some(ref u) = upper {
                visitor.visit_expr(u);
            }
            if let Some(ref s) = step {
                visitor.visit_expr(s);
            }
        }
        _ => {}
    }
}

pub fn walk_pattern(visitor: &mut dyn Visitor, pat: &Pattern) {
    match pat {
        Pattern::Sequence(pats, _) => {
            for p in pats {
                visitor.visit_pattern(p);
            }
        }
        Pattern::Mapping(items, _rest, _) => {
            for (k, v) in items {
                visitor.visit_pattern(k);
                visitor.visit_pattern(v);
            }
        }
        Pattern::Class(_, args, _kwargs, _) => {
            for a in args {
                visitor.visit_pattern(a);
            }
        }
        Pattern::Or(pats, _) => {
            for p in pats {
                visitor.visit_pattern(p);
            }
        }
        Pattern::As(pat, _, _) => visitor.visit_pattern(pat),
        Pattern::Guard(pat, guard, _) => {
            visitor.visit_pattern(pat);
            visitor.visit_expr(guard);
        }
        Pattern::Group(pat, _) => visitor.visit_pattern(pat),
        _ => {}
    }
}
