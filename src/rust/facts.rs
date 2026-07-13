use super::ast;
use crate::{facts, ExtractionResult};
use std::string::String as StdString;

// ── Public API ──────────────────────────────────────────────────────────────

pub fn extract_facts(program: &ast::Program) -> ExtractionResult {
    ExtractionResult {
        symbols: extract_symbols(program),
        imports: extract_imports(program),
        calls: extract_calls(program),
        variables: extract_variables(program),
    }
}

pub fn extract_symbols(program: &ast::Program) -> Vec<facts::ParsedSymbol> {
    let mut symbols = Vec::new();
    collect_symbols(&program.items, None, &mut symbols);
    symbols
}

pub fn extract_imports(program: &ast::Program) -> Vec<facts::ParsedImport> {
    let mut imports = Vec::new();
    collect_imports(&program.items, &mut imports);
    imports
}

pub fn extract_calls(program: &ast::Program) -> Vec<facts::ParsedCall> {
    let mut calls = Vec::new();
    for item in &program.items {
        collect_calls_in_item(item, &mut calls);
    }
    calls
}

pub fn extract_variables(program: &ast::Program) -> Vec<facts::ParsedVariable> {
    let mut variables = Vec::new();
    collect_variables_at_items(&program.items, &mut variables);
    for item in &program.items {
        collect_vars_in_fn_bodies(item, &mut variables);
    }
    variables
}

// ── Visibility ──────────────────────────────────────────────────────────────

fn conv_visibility(v: &ast::stmt::Visibility) -> facts::Visibility {
    use ast::stmt::Visibility as RV;
    match v {
        RV::Pub => facts::Visibility::Public,
        RV::PubCrate | RV::PubSuper | RV::PubIn(_) => facts::Visibility::Internal,
        RV::Private => facts::Visibility::Private,
    }
}

// ── Expr → string / TypeKind ───────────────────────────────────────────────

fn expr_to_string(e: &ast::Expr) -> String {
    use ast::Expr::*;
    match e {
        Path(segments, _) => segments.join("::"),
        Ident(name, _) => name.clone(),
        String(s, _) => format!("\"{s}\""),
        Bool(b, _) => b.to_string(),
        Int(i, _) => i.to_string(),
        Float(f, _) => f.to_string(),
        Char(c, _) => format!("'{c}'"),
        Call(callee, args, _) => {
            let a: Vec<StdString> = args.iter().map(expr_to_string).collect();
            format!("{}({})", expr_to_string(callee), a.join(", "))
        }
        MethodCall(object, method, args, _) => {
            let a: Vec<StdString> = args.iter().map(expr_to_string).collect();
            format!("{}.{}({})", expr_to_string(object), method, a.join(", "))
        }
        Field(object, name, _) => format!("{}.{}", expr_to_string(object), name),
        Index(base, index, _) => format!("{}[{}]", expr_to_string(base), expr_to_string(index)),
        Tuple(elts, _) => {
            let i: Vec<StdString> = elts.iter().map(expr_to_string).collect();
            format!("({})", i.join(", "))
        }
        Array(elts, _) => {
            let i: Vec<StdString> = elts.iter().map(expr_to_string).collect();
            format!("[{}]", i.join(", "))
        }
        Struct(name, fields, base, _) => {
            let f: Vec<StdString> = fields.iter().map(|f| f.name.clone()).collect();
            let b = base.as_ref().map(|_| " ..").unwrap_or("");
            format!("{name} {{ {}{b} }}", f.join(", "))
        }
        Binary(left, op, right, _) => {
            format!(
                "{} {} {}",
                expr_to_string(left),
                binop_str(op),
                expr_to_string(right),
            )
        }
        Unary(_, operand, _) => expr_to_string(operand),
        Paren(inner, _) => format!("({})", expr_to_string(inner)),
        Ref(inner, mut_, _) => {
            let m = if *mut_ { "mut " } else { "" };
            format!("&{m}{}", expr_to_string(inner))
        }
        Deref(inner, _) => format!("*{}", expr_to_string(inner)),
        Closure(..) => "|...| { ... }".into(),
        Block(..) => "{ ... }".into(),
        If(..) => "if ... { ... }".into(),
        Match(..) => "match ... { ... }".into(),
        While(..) => "while ... { ... }".into(),
        Loop(..) => "loop { ... }".into(),
        For(..) => "for ... in ... { ... }".into(),
        Return(v, _) => v
            .as_ref()
            .map_or("return".into(), |x| format!("return {}", expr_to_string(x))),
        Break(v, _) => v
            .as_ref()
            .map_or("break".into(), |x| format!("break {}", expr_to_string(x))),
        Continue(_) => "continue".into(),
        Async(inner, _) => format!("async {}", expr_to_string(inner)),
        Await(inner, _) => format!("{}.await", expr_to_string(inner)),
        Cast(expr, ty, _) => format!("{} as {}", expr_to_string(expr), expr_to_string(ty)),
        Error(_) => "<error>".into(),
    }
}

fn binop_str(op: &ast::BinaryOp) -> &'static str {
    use ast::BinaryOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
        Eq => "==",
        Ne => "!=",
        Lt => "<",
        Gt => ">",
        Le => "<=",
        Ge => ">=",
        And => "&&",
        Or => "||",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
        Assign => "=",
        AddAssign => "+=",
        SubAssign => "-=",
        MulAssign => "*=",
        DivAssign => "/=",
        RemAssign => "%=",
        AndAssign => "&=",
        OrAssign => "|=",
        XorAssign => "^=",
        ShlAssign => "<<=",
        ShrAssign => ">>=",
        Range => "..",
        RangeInclusive => "..=",
        Pipe => "|>",
    }
}

fn expr_to_type_kind(e: &ast::Expr) -> facts::TypeKind {
    match e {
        ast::Expr::Path(segments, _) => facts::TypeKind::simple(segments.join("::")),
        ast::Expr::Ident(name, _) => facts::TypeKind::simple(name.clone()),
        _ => facts::TypeKind::Inferred,
    }
}

fn pattern_to_string(p: &ast::Pattern) -> String {
    use ast::Pattern::*;
    match p {
        Wildcard(_) => "_".into(),
        Ident(name, _) => name.clone(),
        Lit(expr, _) => expr_to_string(expr),
        Tuple(patterns, _) => {
            let i: Vec<StdString> = patterns.iter().map(pattern_to_string).collect();
            format!("({})", i.join(", "))
        }
        Struct(name, fields, _) => {
            let f: Vec<StdString> = fields.iter().map(|fp| fp.name.clone()).collect();
            format!("{name} {{ {} }}", f.join(", "))
        }
        Range(lo, hi, _) => format!("{}..{}", pattern_to_string(lo), pattern_to_string(hi)),
        Or(patterns, _) => {
            let i: Vec<StdString> = patterns.iter().map(pattern_to_string).collect();
            i.join(" | ")
        }
        Ref(inner, _, _) => format!("&{}", pattern_to_string(inner)),
        Slice(patterns, _) => {
            let i: Vec<StdString> = patterns.iter().map(pattern_to_string).collect();
            format!("[{}]", i.join(", "))
        }
        Rest(_) => "..".into(),
    }
}

fn build_signature(fd: &ast::FnDecl) -> String {
    let mut s = format!("fn {}", fd.name);
    if !fd.generics.is_empty() {
        let p: Vec<&str> = fd.generics.iter().map(|g| g.name.as_str()).collect();
        s.push('<');
        s.push_str(&p.join(", "));
        s.push('>');
    }
    s.push('(');
    let p: Vec<StdString> = fd
        .params
        .iter()
        .map(|p| {
            let mut s = pattern_to_string(&p.pattern);
            if let Some(ref ty) = p.type_ann {
                s.push_str(": ");
                s.push_str(&expr_to_string(ty));
            }
            s
        })
        .collect();
    s.push_str(&p.join(", "));
    s.push(')');
    if let Some(ref ret) = fd.return_type {
        s.push_str(" -> ");
        s.push_str(&expr_to_string(ret));
    }
    s
}

fn is_self_param(p: &ast::FnParam) -> bool {
    matches!(&p.pattern, ast::Pattern::Ident(name, _) if name == "self")
}

// ── Symbols ────────────────────────────────────────────────────────────────

fn collect_symbols(
    items: &[ast::Item],
    parent_impl: Option<&ast::ImplBlock>,
    symbols: &mut Vec<facts::ParsedSymbol>,
) {
    for item in items {
        match item {
            ast::Item::Fn(fd, span) => {
                handle_fn_symbol(fd, span, parent_impl, symbols);
            }
            ast::Item::Struct(sd, span) => {
                let vis = conv_visibility(&sd.visibility);
                symbols.push(
                    facts::ParsedSymbol::builder(facts::SymbolKind::Struct, &sd.name)
                        .exported(matches!(vis, facts::Visibility::Public))
                        .visibility(vis)
                        .lines(span.start.line, span.end.line)
                        .type_kind(facts::TypeKind::simple("struct"))
                        .build(),
                );
                for field in &sd.fields {
                    symbols.push(
                        facts::ParsedSymbol::builder(facts::SymbolKind::Property, &field.name)
                            .lines(field.span.start.line, field.span.end.line)
                            .type_kind(expr_to_type_kind(&field.type_ann))
                            .build(),
                    );
                }
            }
            ast::Item::Enum(ed, span) => {
                let vis = conv_visibility(&ed.visibility);
                symbols.push(
                    facts::ParsedSymbol::builder(facts::SymbolKind::Enum, &ed.name)
                        .exported(matches!(vis, facts::Visibility::Public))
                        .visibility(vis)
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                for variant in &ed.variants {
                    symbols.push(
                        facts::ParsedSymbol::builder(facts::SymbolKind::Constant, &variant.name)
                            .lines(variant.span.start.line, variant.span.end.line)
                            .build(),
                    );
                }
            }
            ast::Item::Trait(td, span) => {
                let vis = conv_visibility(&td.visibility);
                symbols.push(
                    facts::ParsedSymbol::builder(facts::SymbolKind::Trait, &td.name)
                        .exported(matches!(vis, facts::Visibility::Public))
                        .visibility(vis)
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                for method in &td.methods {
                    let mut kind = facts::SymbolKind::Method;
                    let is_constructor = method.name == "new";
                    if is_constructor {
                        kind = facts::SymbolKind::Constructor;
                    }
                    let ret = method.return_type.as_ref().map(|r| expr_to_string(r));
                    symbols.push(
                        facts::ParsedSymbol::builder(kind, &method.name)
                            .visibility(conv_visibility(&method.visibility))
                            .lines(method.span.start.line, method.span.end.line)
                            .signature(build_signature(method))
                            .return_type(ret)
                            .constructor(is_constructor)
                            .is_async(method.is_async)
                            .static_(method.params.first().is_none_or(|p| !is_self_param(p)))
                            .is_test(is_rust_test_fn(&method.name))
                            .type_kind(facts::TypeKind::simple("fn"))
                            .build(),
                    );
                }
            }
            ast::Item::Impl(ib, _span) => {
                collect_symbols_from_impl(ib, symbols);
            }
            ast::Item::Mod(md, span) => {
                let vis = conv_visibility(&md.visibility);
                symbols.push(
                    facts::ParsedSymbol::builder(facts::SymbolKind::Module, &md.name)
                        .exported(matches!(vis, facts::Visibility::Public))
                        .visibility(vis)
                        .lines(span.start.line, span.end.line)
                        .build(),
                );
                collect_symbols(&md.items, None, symbols);
            }
            ast::Item::Type(ta, span) => {
                let vis = conv_visibility(&ta.visibility);
                symbols.push(
                    facts::ParsedSymbol::builder(facts::SymbolKind::Type, &ta.name)
                        .exported(matches!(vis, facts::Visibility::Public))
                        .visibility(vis)
                        .lines(span.start.line, span.end.line)
                        .type_kind(expr_to_type_kind(&ta.type_))
                        .build(),
                );
            }
            ast::Item::Static(sd, span) => {
                let vis = conv_visibility(&sd.visibility);
                symbols.push(
                    facts::ParsedSymbol::builder(facts::SymbolKind::Constant, &sd.name)
                        .exported(matches!(vis, facts::Visibility::Public))
                        .visibility(vis)
                        .lines(span.start.line, span.end.line)
                        .storage(facts::StorageClass::Static)
                        .type_kind(expr_to_type_kind(&sd.type_))
                        .build(),
                );
            }
            ast::Item::Const(cd, span) => {
                let vis = conv_visibility(&cd.visibility);
                symbols.push(
                    facts::ParsedSymbol::builder(facts::SymbolKind::Constant, &cd.name)
                        .exported(matches!(vis, facts::Visibility::Public))
                        .visibility(vis)
                        .lines(span.start.line, span.end.line)
                        .storage(facts::StorageClass::Global)
                        .type_kind(
                            cd.type_
                                .as_ref()
                                .map_or(facts::TypeKind::Inferred, |t| expr_to_type_kind(t)),
                        )
                        .build(),
                );
            }
            ast::Item::Use(..) | ast::Item::Macro(..) | ast::Item::ExternCrate(..) => {}
        }
    }
}

fn handle_fn_symbol(
    fd: &ast::FnDecl,
    span: &crate::span::Span,
    parent_impl: Option<&ast::ImplBlock>,
    symbols: &mut Vec<facts::ParsedSymbol>,
) {
    let is_in_impl = parent_impl.is_some();
    let mut kind = facts::SymbolKind::Function;
    let mut is_constructor = false;
    let mut is_destructor = false;

    if is_in_impl {
        if fd.name == "new" {
            kind = facts::SymbolKind::Constructor;
            is_constructor = true;
        }
        if let Some(ib) = parent_impl {
            if ib.trait_name.as_deref() == Some("Drop") && fd.name == "drop" {
                kind = facts::SymbolKind::Destructor;
                is_destructor = true;
            }
        }
    }

    let vis = conv_visibility(&fd.visibility);
    let ret = fd.return_type.as_ref().map(|r| expr_to_string(r));
    let is_static = is_in_impl && fd.params.first().is_none_or(|p| !is_self_param(p));
    let is_test = is_rust_test_fn(&fd.name);

    symbols.push(
        facts::ParsedSymbol::builder(kind, &fd.name)
            .exported(matches!(vis, facts::Visibility::Public))
            .visibility(vis)
            .lines(span.start.line, span.end.line)
            .signature(build_signature(fd))
            .return_type(ret)
            .is_async(fd.is_async)
            .constructor(is_constructor)
            .destructor(is_destructor)
            .static_(is_static)
            .is_test(is_test)
            .type_kind(facts::TypeKind::simple("fn"))
            .build(),
    );
}

/// Rust test-function heuristic. The AST does not retain `#[test]` attributes,
/// so we use the naming convention: functions named `test_*` (the standard
/// `#[test] fn test_foo()` idiom). This matches the tree-sitter indexer and
/// graxus's `dead-code --exclude-tests` expectation.
fn is_rust_test_fn(name: &str) -> bool {
    name.starts_with("test_")
}

fn collect_symbols_from_impl(ib: &ast::ImplBlock, symbols: &mut Vec<facts::ParsedSymbol>) {
    for method in &ib.methods {
        let mut kind = facts::SymbolKind::Function;
        let mut is_constructor = false;
        let mut is_destructor = false;

        if method.name == "new" {
            kind = facts::SymbolKind::Constructor;
            is_constructor = true;
        }
        if ib.trait_name.as_deref() == Some("Drop") && method.name == "drop" {
            kind = facts::SymbolKind::Destructor;
            is_destructor = true;
        }

        let vis = conv_visibility(&method.visibility);
        let ret = method.return_type.as_ref().map(|r| expr_to_string(r));
        let is_static = method.params.first().is_none_or(|p| !is_self_param(p));

        symbols.push(
            facts::ParsedSymbol::builder(kind, &method.name)
                .exported(matches!(vis, facts::Visibility::Public))
                .visibility(vis)
                .lines(method.span.start.line, method.span.end.line)
                .signature(build_signature(method))
                .return_type(ret)
                .is_async(method.is_async)
                .constructor(is_constructor)
                .destructor(is_destructor)
                .static_(is_static)
                .is_test(is_rust_test_fn(&method.name))
                .type_kind(facts::TypeKind::simple("fn"))
                .build(),
        );
    }
}

// ── Imports ─────────────────────────────────────────────────────────────────

fn collect_imports(items: &[ast::Item], imports: &mut Vec<facts::ParsedImport>) {
    for item in items {
        match item {
            ast::Item::Use(ud, span) => convert_use_path(&ud.path, imports, span.start.line),
            ast::Item::Mod(md, _) => collect_imports(&md.items, imports),
            _ => {}
        }
    }
}

fn convert_use_path(path: &ast::UsePath, imports: &mut Vec<facts::ParsedImport>, line: usize) {
    match path {
        ast::UsePath::Simple(p, _) => {
            imports.push(
                facts::ParsedImport::builder(facts::ImportKind::RustUse, p)
                    .line(line)
                    .build(),
            );
        }
        ast::UsePath::Glob(p, _) => {
            let source = format!("{p}::*");
            imports.push(
                facts::ParsedImport::builder(facts::ImportKind::NamespaceImport, &source)
                    .line(line)
                    .star(true)
                    .build(),
            );
        }
        ast::UsePath::Nested(base, children, _) => {
            for child in children {
                match child {
                    ast::UsePath::Simple(name, _) => {
                        let full = format!("{base}::{name}");
                        imports.push(
                            facts::ParsedImport::builder(facts::ImportKind::RustUse, &full)
                                .line(line)
                                .build(),
                        );
                    }
                    ast::UsePath::Glob(_, _) => {
                        let source = format!("{base}::*");
                        imports.push(
                            facts::ParsedImport::builder(
                                facts::ImportKind::NamespaceImport,
                                &source,
                            )
                            .line(line)
                            .star(true)
                            .build(),
                        );
                    }
                    ast::UsePath::Self_(name, _) => {
                        let full = format!("{base}::{name}");
                        imports.push(
                            facts::ParsedImport::builder(facts::ImportKind::RustUse, &full)
                                .line(line)
                                .build(),
                        );
                    }
                    ast::UsePath::Nested(..) => {}
                }
            }
        }
        ast::UsePath::Self_(p, _) => {
            imports.push(
                facts::ParsedImport::builder(facts::ImportKind::RustUse, p)
                    .line(line)
                    .build(),
            );
        }
    }
}

// ── Variables ───────────────────────────────────────────────────────────────

fn collect_variables_at_items(items: &[ast::Item], variables: &mut Vec<facts::ParsedVariable>) {
    for item in items {
        match item {
            ast::Item::Static(sd, span) => {
                variables.push(
                    facts::ParsedVariable::builder(&sd.name, facts::VarKind::Static)
                        .mutable(sd.mutable)
                        .line(span.start.line)
                        .storage(facts::StorageClass::Static)
                        .type_kind(expr_to_type_kind(&sd.type_))
                        .build(),
                );
            }
            ast::Item::Const(cd, span) => {
                variables.push(
                    facts::ParsedVariable::builder(&cd.name, facts::VarKind::Const)
                        .line(span.start.line)
                        .storage(facts::StorageClass::Global)
                        .type_kind(
                            cd.type_
                                .as_ref()
                                .map_or(facts::TypeKind::Inferred, |t| expr_to_type_kind(t)),
                        )
                        .build(),
                );
            }
            ast::Item::Mod(md, _) => collect_variables_at_items(&md.items, variables),
            _ => {}
        }
    }
}

fn collect_vars_in_fn_bodies(item: &ast::Item, variables: &mut Vec<facts::ParsedVariable>) {
    match item {
        ast::Item::Fn(fd, _) => {
            for param in &fd.params {
                if let ast::Pattern::Ident(name, _) = &param.pattern {
                    let type_ann = param.type_ann.as_ref().map(|t| expr_to_string(t));
                    let type_kind = param
                        .type_ann
                        .as_ref()
                        .map_or(facts::TypeKind::Inferred, |t| expr_to_type_kind(t));
                    variables.push(
                        facts::ParsedVariable::builder(name, facts::VarKind::Parameter)
                            .line(fd.span.start.line)
                            .type_ann(type_ann)
                            .type_kind(type_kind)
                            .build(),
                    );
                }
            }
            if let Some(ref body) = fd.body {
                for stmt in &body.stmts {
                    collect_vars_in_stmt(stmt, variables);
                }
            }
        }
        ast::Item::Impl(ib, _) => {
            for method in &ib.methods {
                for param in &method.params {
                    if let ast::Pattern::Ident(name, _) = &param.pattern {
                        let type_ann = param.type_ann.as_ref().map(|t| expr_to_string(t));
                        let type_kind = param
                            .type_ann
                            .as_ref()
                            .map_or(facts::TypeKind::Inferred, |t| expr_to_type_kind(t));
                        variables.push(
                            facts::ParsedVariable::builder(name, facts::VarKind::Parameter)
                                .line(method.span.start.line)
                                .type_ann(type_ann)
                                .type_kind(type_kind)
                                .build(),
                        );
                    }
                }
                if let Some(ref body) = method.body {
                    for stmt in &body.stmts {
                        collect_vars_in_stmt(stmt, variables);
                    }
                }
            }
        }
        ast::Item::Trait(td, _) => {
            for method in &td.methods {
                for param in &method.params {
                    if let ast::Pattern::Ident(name, _) = &param.pattern {
                        let type_ann = param.type_ann.as_ref().map(|t| expr_to_string(t));
                        let type_kind = param
                            .type_ann
                            .as_ref()
                            .map_or(facts::TypeKind::Inferred, |t| expr_to_type_kind(t));
                        variables.push(
                            facts::ParsedVariable::builder(name, facts::VarKind::Parameter)
                                .line(method.span.start.line)
                                .type_ann(type_ann)
                                .type_kind(type_kind)
                                .build(),
                        );
                    }
                }
                if let Some(ref body) = method.body {
                    for stmt in &body.stmts {
                        collect_vars_in_stmt(stmt, variables);
                    }
                }
            }
        }
        ast::Item::Mod(md, _) => {
            for child in &md.items {
                collect_vars_in_fn_bodies(child, variables);
            }
        }
        _ => {}
    }
}

fn collect_vars_in_stmt(stmt: &ast::Stmt, variables: &mut Vec<facts::ParsedVariable>) {
    match stmt {
        ast::Stmt::Let(ld, span) => {
            if let ast::Pattern::Ident(name, _) = &ld.pattern {
                let type_ann = ld.type_ann.as_ref().map(|t| expr_to_string(t));
                let type_kind = ld
                    .type_ann
                    .as_ref()
                    .map_or(facts::TypeKind::Inferred, |t| expr_to_type_kind(t));
                variables.push(
                    facts::ParsedVariable::builder(name, facts::VarKind::Let)
                        .mutable(ld.mutable)
                        .line(span.start.line)
                        .type_ann(type_ann)
                        .storage(facts::StorageClass::Local)
                        .type_kind(type_kind)
                        .build(),
                );
            }
        }
        ast::Stmt::Item(item, _) => {
            if let ast::Item::Static(sd, span) = item {
                variables.push(
                    facts::ParsedVariable::builder(&sd.name, facts::VarKind::Static)
                        .mutable(sd.mutable)
                        .line(span.start.line)
                        .storage(facts::StorageClass::Static)
                        .build(),
                );
            } else if let ast::Item::Const(cd, span) = item {
                variables.push(
                    facts::ParsedVariable::builder(&cd.name, facts::VarKind::Const)
                        .line(span.start.line)
                        .storage(facts::StorageClass::Global)
                        .build(),
                );
            }
        }
        ast::Stmt::Expr(e, _) => {
            if let ast::Expr::Block(block, _) = e {
                for s in &block.stmts {
                    collect_vars_in_stmt(s, variables);
                }
            }
        }
        ast::Stmt::Empty(_) => {}
    }
}

// ── Calls ───────────────────────────────────────────────────────────────────

fn collect_calls_in_item(item: &ast::Item, calls: &mut Vec<facts::ParsedCall>) {
    match item {
        ast::Item::Fn(fd, _) => {
            if let Some(ref body) = fd.body {
                for stmt in &body.stmts {
                    collect_calls_in_stmt(stmt, calls);
                }
            }
        }
        ast::Item::Impl(ib, _) => {
            for method in &ib.methods {
                if let Some(ref body) = method.body {
                    for stmt in &body.stmts {
                        collect_calls_in_stmt(stmt, calls);
                    }
                }
            }
        }
        ast::Item::Trait(td, _) => {
            for method in &td.methods {
                if let Some(ref body) = method.body {
                    for stmt in &body.stmts {
                        collect_calls_in_stmt(stmt, calls);
                    }
                }
            }
        }
        ast::Item::Mod(md, _) => {
            for child in &md.items {
                collect_calls_in_item(child, calls);
            }
        }
        // Macro invocations (e.g. `vec![...]`, `println!(...)`) are call-like
        // constructs; previously dropped from the call graph.
        ast::Item::Macro(inv, span) => {
            calls.push(
                facts::ParsedCall::builder(facts::CallKind::FunctionCall, &inv.name)
                    .pos(span.start.line, span.start.column)
                    .build(),
            );
        }
        _ => {}
    }
}

fn collect_calls_in_stmt(stmt: &ast::Stmt, calls: &mut Vec<facts::ParsedCall>) {
    match stmt {
        ast::Stmt::Expr(expr, _) => collect_calls_in_expr(expr, calls),
        ast::Stmt::Let(ld, _) => {
            if let Some(ref init) = ld.init {
                collect_calls_in_expr(init, calls);
            }
            if let Some(ref type_ann) = ld.type_ann {
                collect_calls_in_expr(type_ann, calls);
            }
        }
        ast::Stmt::Item(item, _) => {
            // static/const initializers may contain calls
            match item {
                ast::Item::Static(sd, _) => collect_calls_in_expr(&sd.init, calls),
                ast::Item::Const(cd, _) => collect_calls_in_expr(&cd.init, calls),
                _ => collect_calls_in_item(item, calls),
            }
        }
        ast::Stmt::Empty(_) => {} // Macro invocations inside fn bodies are handled via Stmt::Item →
                                  // Item::Macro in collect_calls_in_item above; Stmt has no Macro variant.
    }
}

fn collect_calls_in_expr(expr: &ast::Expr, calls: &mut Vec<facts::ParsedCall>) {
    match expr {
        ast::Expr::Call(callee, args, span) => {
            let text = expr_to_string(callee);
            let kind = match callee.as_ref() {
                ast::Expr::Field(..) => facts::CallKind::MethodCall,
                _ => facts::CallKind::FunctionCall,
            };
            calls.push(
                facts::ParsedCall::builder(kind, &text)
                    .pos(span.start.line, span.start.column)
                    .build(),
            );
            collect_calls_in_expr(callee, calls);
            for a in args {
                collect_calls_in_expr(a, calls);
            }
        }
        ast::Expr::MethodCall(object, method, args, span) => {
            calls.push(
                facts::ParsedCall::builder(facts::CallKind::MethodCall, method)
                    .object(expr_to_string(object))
                    .pos(span.start.line, span.start.column)
                    .build(),
            );
            collect_calls_in_expr(object, calls);
            for a in args {
                collect_calls_in_expr(a, calls);
            }
        }
        ast::Expr::Binary(left, _, right, _) => {
            collect_calls_in_expr(left, calls);
            collect_calls_in_expr(right, calls);
        }
        ast::Expr::Unary(_, operand, _) => collect_calls_in_expr(operand, calls),
        ast::Expr::Index(base, index, _) => {
            collect_calls_in_expr(base, calls);
            collect_calls_in_expr(index, calls);
        }
        ast::Expr::Field(object, _, _) => collect_calls_in_expr(object, calls),
        ast::Expr::Tuple(elts, _) => {
            for e in elts {
                collect_calls_in_expr(e, calls);
            }
        }
        ast::Expr::Array(elts, _) => {
            for e in elts {
                collect_calls_in_expr(e, calls);
            }
        }
        ast::Expr::Struct(_, fields, base, _) => {
            for f in fields {
                if let Some(ref val) = f.value {
                    collect_calls_in_expr(val, calls);
                }
            }
            if let Some(base_expr) = base {
                collect_calls_in_expr(base_expr, calls);
            }
        }
        ast::Expr::Closure(_, body, _) => collect_calls_in_expr(body, calls),
        ast::Expr::Block(block, _) => {
            for s in &block.stmts {
                collect_calls_in_stmt(s, calls);
            }
        }
        ast::Expr::If(cond, then_, else_, _) => {
            collect_calls_in_expr(cond, calls);
            for s in &then_.stmts {
                collect_calls_in_stmt(s, calls);
            }
            if let Some(else_expr) = else_ {
                collect_calls_in_expr(else_expr, calls);
            }
        }
        ast::Expr::Match(expr_val, arms, _) => {
            collect_calls_in_expr(expr_val, calls);
            for arm in arms {
                collect_calls_in_expr(&arm.body, calls);
            }
        }
        ast::Expr::While(cond, body, _) => {
            collect_calls_in_expr(cond, calls);
            for s in &body.stmts {
                collect_calls_in_stmt(s, calls);
            }
        }
        ast::Expr::Loop(body, _) => {
            for s in &body.stmts {
                collect_calls_in_stmt(s, calls);
            }
        }
        ast::Expr::For(_, iter, body, _) => {
            collect_calls_in_expr(iter, calls);
            for s in &body.stmts {
                collect_calls_in_stmt(s, calls);
            }
        }
        ast::Expr::Return(v, _) => {
            if let Some(inner) = v {
                collect_calls_in_expr(inner, calls);
            }
        }
        ast::Expr::Break(v, _) => {
            if let Some(inner) = v {
                collect_calls_in_expr(inner, calls);
            }
        }
        ast::Expr::Paren(inner, _) => collect_calls_in_expr(inner, calls),
        ast::Expr::Async(inner, _) => collect_calls_in_expr(inner, calls),
        ast::Expr::Await(inner, _) => collect_calls_in_expr(inner, calls),
        ast::Expr::Ref(inner, _, _) => collect_calls_in_expr(inner, calls),
        ast::Expr::Deref(inner, _) => collect_calls_in_expr(inner, calls),
        ast::Expr::Cast(expr_val, type_, _) => {
            collect_calls_in_expr(expr_val, calls);
            collect_calls_in_expr(type_, calls);
        }
        ast::Expr::Continue(_)
        | ast::Expr::Bool(..)
        | ast::Expr::Int(..)
        | ast::Expr::Float(..)
        | ast::Expr::String(..)
        | ast::Expr::Char(..)
        | ast::Expr::Ident(..)
        | ast::Expr::Path(..)
        | ast::Expr::Error(_) => {}
    }
}
