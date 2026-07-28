use super::ast::{
    Block, CaseClause, Decl, Expr, FieldInit, FuncDecl, Program, Stmt, TypeDecl, UnaryOp,
};
use crate::facts::{
    CallKind, ImportKind, ParsedCall, ParsedImport, ParsedSymbol, ParsedVariable, SymbolKind,
    TypeKind, VarKind, Visibility,
};
use crate::ExtractionResult;

pub fn extract_facts(program: &Program) -> ExtractionResult {
    ExtractionResult {
        symbols: extract_symbols(program),
        imports: extract_imports(program),
        calls: extract_calls(program),
        variables: extract_variables(program),
    }
}

pub fn extract_symbols(program: &Program) -> Vec<ParsedSymbol> {
    program.decls.iter().flat_map(symbols_from_decl).collect()
}

pub fn extract_imports(program: &Program) -> Vec<ParsedImport> {
    program.decls.iter().flat_map(imports_from_decl).collect()
}

pub fn extract_calls(program: &Program) -> Vec<ParsedCall> {
    program.decls.iter().flat_map(calls_from_decl).collect()
}

pub fn extract_variables(program: &Program) -> Vec<ParsedVariable> {
    program.decls.iter().flat_map(variables_from_decl).collect()
}

fn is_exported(name: &str) -> bool {
    name.chars().next().is_some_and(|c| c.is_uppercase())
}

fn expr_to_type_kind(e: &Expr) -> TypeKind {
    match e {
        Expr::Ident(s, _) => TypeKind::Simple(s.clone()),
        Expr::Selector(obj, field, _) => {
            let base = expr_to_string(obj);
            TypeKind::Simple(format!("{base}.{field}"))
        }
        Expr::Unary(UnaryOp::Deref, inner, _) => {
            TypeKind::Pointer(Box::new(expr_to_type_kind(inner)))
        }
        Expr::Paren(inner, _) => expr_to_type_kind(inner),
        _ => TypeKind::Unknown,
    }
}

fn expr_to_string(e: &Expr) -> String {
    match e {
        Expr::Ident(s, _) => s.clone(),
        Expr::Selector(obj, field, _) => format!("{}.{field}", expr_to_string(obj)),
        Expr::Binary(l, op, r, _) => {
            format!("{} {:?} {}", expr_to_string(l), op, expr_to_string(r))
        }
        Expr::Unary(op, x, _) => format!("{:?}{}", op, expr_to_string(x)),
        Expr::Call(fun, args, _) => {
            let args: Vec<String> = args.iter().map(expr_to_string).collect();
            format!("{}({})", expr_to_string(fun), args.join(", "))
        }
        Expr::Index(obj, idx, _) => format!("{}[{}]", expr_to_string(obj), expr_to_string(idx)),
        Expr::Paren(inner, _) => format!("({})", expr_to_string(inner)),
        Expr::TypeAssert(expr, typ, _) => {
            format!("{}.({})", expr_to_string(expr), expr_to_string(typ))
        }
        Expr::Slice(expr, from, to, _) => {
            let e = expr_to_string(expr);
            let f = from.as_ref().map(|x| expr_to_string(x)).unwrap_or_default();
            let t = to.as_ref().map(|x| expr_to_string(x)).unwrap_or_default();
            format!("{}[{}:{}]", e, f, t)
        }
        Expr::Array(items, _) => {
            let inner: Vec<String> = items.iter().map(expr_to_string).collect();
            format!("[{}]", inner.join(", "))
        }
        Expr::StructLit(name, ..) => name.clone(),
        Expr::MapLit(..) => "map[...]".to_string(),
        Expr::FuncLit(..) => "func(...)".to_string(),
        Expr::CompositeLit(typ, ..) => expr_to_string(typ),
        Expr::Bool(v, _) => v.to_string(),
        Expr::Int(v, _) => v.to_string(),
        Expr::Float(v, _) => v.to_string(),
        Expr::String(s, _) => format!("\"{s}\""),
        Expr::Nil(_) => "nil".to_string(),
    }
}

fn returns_to_string(returns: &[Box<Expr>]) -> Option<String> {
    if returns.is_empty() {
        None
    } else {
        Some(
            returns
                .iter()
                .map(|e| expr_to_string(e))
                .collect::<Vec<_>>()
                .join(", "),
        )
    }
}

fn symbols_from_decl(decl: &Decl) -> Vec<ParsedSymbol> {
    match decl {
        Decl::Func(fd, _) => vec![symbol_from_func(fd)],
        Decl::Type(td, _) => vec![symbol_from_type(td)],
        Decl::Const(cd, _) => cd.names.iter().map(|n| symbol_from_const(n)).collect(),
        Decl::Var(vd, _) => vd.names.iter().map(|n| symbol_from_var(n)).collect(),
        _ => vec![],
    }
}

fn variables_from_func(fd: &FuncDecl) -> Vec<ParsedVariable> {
    let mut variables: Vec<_> = fd
        .params
        .iter()
        .map(|(name, typ)| {
            ParsedVariable::builder(name, VarKind::Parameter)
                .type_ann(Some(expr_to_string(typ)))
                .type_kind(expr_to_type_kind(typ))
                .line(fd.span.start.line)
                .build()
        })
        .collect();
    if let Some(body) = &fd.body {
        variables.extend(variables_from_block(body));
    }
    variables
}

fn variables_from_block(block: &Block) -> Vec<ParsedVariable> {
    block.stmts.iter().flat_map(variables_from_stmt).collect()
}

fn variables_from_stmt(stmt: &Stmt) -> Vec<ParsedVariable> {
    match stmt {
        Stmt::Decl(decl, _) => variables_from_decl(decl),
        Stmt::Define(names, _, span) => names
            .iter()
            .filter_map(|expr| match expr {
                Expr::Ident(name, _) if name != "_" => Some(
                    ParsedVariable::builder(name, VarKind::Let)
                        .mutable(true)
                        .line(span.start.line)
                        .build(),
                ),
                _ => None,
            })
            .collect(),
        Stmt::If(_, body, alternate, _) => {
            let mut values = variables_from_stmt(body);
            if let Some(alternate) = alternate {
                values.extend(variables_from_stmt(alternate));
            }
            values
        }
        Stmt::For(init, _, post, body, _) => {
            let mut values = init.as_deref().map(variables_from_stmt).unwrap_or_default();
            if let Some(post) = post {
                values.extend(variables_from_stmt(post));
            }
            values.extend(variables_from_stmt(body));
            values
        }
        Stmt::ForRange(_, first, second, body, span) => {
            let mut values = Vec::new();
            for name in std::iter::once(first)
                .chain(second.iter())
                .filter(|name| !name.is_empty() && name.as_str() != "_")
            {
                values.push(
                    ParsedVariable::builder(name, VarKind::Let)
                        .mutable(true)
                        .line(span.start.line)
                        .build(),
                );
            }
            values.extend(variables_from_stmt(body));
            values
        }
        Stmt::Switch(_, cases, _) | Stmt::Select(cases, _) => cases
            .iter()
            .flat_map(|case| case.body.iter().flat_map(variables_from_stmt))
            .collect(),
        Stmt::Block(block, _) => variables_from_block(block),
        Stmt::Label(_, body, _) => variables_from_stmt(body),
        _ => Vec::new(),
    }
}

fn symbol_from_func(fd: &FuncDecl) -> ParsedSymbol {
    let exported = is_exported(&fd.name);
    let is_constructor = fd.name.starts_with("New");
    let is_test = is_go_test_fn(&fd.name);
    let (kind, name) = if let Some((receiver_type, _)) = &fd.receiver {
        (SymbolKind::Method, format!("{}.{}", receiver_type, fd.name))
    } else if is_constructor {
        (SymbolKind::Constructor, fd.name.clone())
    } else {
        (SymbolKind::Function, fd.name.clone())
    };
    let visibility = if exported {
        Visibility::Public
    } else {
        Visibility::Private
    };
    let params: Vec<String> = fd
        .params
        .iter()
        .map(|(n, t)| format!("{} {}", n, expr_to_string(t)))
        .collect();
    let ret = returns_to_string(&fd.returns).unwrap_or_default();
    let signature = if ret.is_empty() {
        format!("({})", params.join(", "))
    } else {
        format!("({}) {}", params.join(", "), ret)
    };
    ParsedSymbol::builder(kind, name)
        .exported(exported)
        .visibility(visibility)
        .lines(fd.span.start.line, fd.span.end.line)
        .signature(signature)
        .return_type(returns_to_string(&fd.returns))
        .constructor(is_constructor)
        .is_test(is_test)
        .build()
}

/// Go test-function conventions: `TestXxx`, `BenchmarkXxx`, `ExampleXxx`,
/// `FuzzXxx` (all must take a `*testing.T` / `*testing.B` to be real tests,
/// but the name prefix is the standard signal graxus relies on).
fn is_go_test_fn(name: &str) -> bool {
    name.starts_with("Test")
        || name.starts_with("Benchmark")
        || name.starts_with("Example")
        || name.starts_with("Fuzz")
}

fn symbol_from_type(td: &TypeDecl) -> ParsedSymbol {
    let exported = is_exported(&td.name);
    let kind = match &*td.kind {
        Expr::StructLit(..) => SymbolKind::Struct,
        Expr::MapLit(..) => SymbolKind::Interface,
        _ => SymbolKind::Type,
    };
    ParsedSymbol::builder(kind, &td.name)
        .exported(exported)
        .visibility(if exported {
            Visibility::Public
        } else {
            Visibility::Private
        })
        .lines(td.span.start.line, td.span.end.line)
        .type_kind(expr_to_type_kind(&td.kind))
        .build()
}

fn symbol_from_const(name: &str) -> ParsedSymbol {
    let exported = is_exported(name);
    ParsedSymbol::builder(SymbolKind::Constant, name)
        .exported(exported)
        .visibility(if exported {
            Visibility::Public
        } else {
            Visibility::Private
        })
        .build()
}

fn symbol_from_var(name: &str) -> ParsedSymbol {
    let exported = is_exported(name);
    ParsedSymbol::builder(SymbolKind::Variable, name)
        .exported(exported)
        .visibility(if exported {
            Visibility::Public
        } else {
            Visibility::Private
        })
        .build()
}

fn imports_from_decl(decl: &Decl) -> Vec<ParsedImport> {
    match decl {
        Decl::Import(id, span) => {
            let mut builder =
                ParsedImport::builder(ImportKind::GoImport, &id.path).line(span.start.line);
            if let Some(alias) = &id.alias {
                builder = builder.local(alias);
            }
            vec![builder.build()]
        }
        Decl::ImportGroup(imports, span) => imports
            .iter()
            .map(|id| {
                let mut builder =
                    ParsedImport::builder(ImportKind::GoImport, &id.path).line(span.start.line);
                if let Some(alias) = &id.alias {
                    builder = builder.local(alias);
                }
                builder.build()
            })
            .collect(),
        _ => vec![],
    }
}

fn variables_from_decl(decl: &Decl) -> Vec<ParsedVariable> {
    match decl {
        Decl::Func(fd, _) => variables_from_func(fd),
        Decl::Var(vd, _) => vd
            .names
            .iter()
            .map(|name| {
                let type_kind = vd
                    .kind
                    .as_ref()
                    .map(|e| expr_to_type_kind(e))
                    .unwrap_or(TypeKind::Unknown);
                ParsedVariable::builder(name, VarKind::Let)
                    .mutable(true)
                    .line(vd.span.start.line)
                    .type_ann(vd.kind.as_ref().map(|e| expr_to_string(e)))
                    .type_kind(type_kind)
                    .build()
            })
            .collect(),
        Decl::Const(cd, _) => cd
            .names
            .iter()
            .map(|name| {
                let type_kind = cd
                    .kind
                    .as_ref()
                    .map(|e| expr_to_type_kind(e))
                    .unwrap_or(TypeKind::Unknown);
                ParsedVariable::builder(name, VarKind::Const)
                    .line(cd.span.start.line)
                    .type_ann(cd.kind.as_ref().map(|e| expr_to_string(e)))
                    .type_kind(type_kind)
                    .build()
            })
            .collect(),
        _ => vec![],
    }
}

fn calls_from_decl(decl: &Decl) -> Vec<ParsedCall> {
    match decl {
        Decl::Func(fd, _) => fd.body.as_ref().map(calls_from_block).unwrap_or_default(),
        Decl::Var(vd, _) => vd.values.iter().flat_map(calls_from_expr).collect(),
        Decl::Const(cd, _) => cd.values.iter().flat_map(calls_from_expr).collect(),
        _ => vec![],
    }
}

fn calls_from_block(block: &Block) -> Vec<ParsedCall> {
    block.stmts.iter().flat_map(calls_from_stmt).collect()
}

fn calls_from_stmt(stmt: &Stmt) -> Vec<ParsedCall> {
    match stmt {
        Stmt::Expr(expr, _) => calls_from_expr(expr),
        Stmt::Decl(decl, _) => calls_from_decl(decl),
        Stmt::Assign(left, right, _) | Stmt::Define(left, right, _) => {
            let mut calls = Vec::new();
            for e in left {
                calls.extend(calls_from_expr(e));
            }
            for e in right {
                calls.extend(calls_from_expr(e));
            }
            calls
        }
        Stmt::If(cond, then, else_, _) => {
            let mut calls = calls_from_expr(cond);
            calls.extend(calls_from_stmt(then));
            if let Some(s) = else_ {
                calls.extend(calls_from_stmt(s));
            }
            calls
        }
        Stmt::For(init, cond, post, body, _) => {
            let mut calls = Vec::new();
            if let Some(s) = init {
                calls.extend(calls_from_stmt(s));
            }
            if let Some(e) = cond {
                calls.extend(calls_from_expr(e));
            }
            if let Some(s) = post {
                calls.extend(calls_from_stmt(s));
            }
            calls.extend(calls_from_stmt(body));
            calls
        }
        Stmt::ForRange(expr, .., body, _) => {
            let mut calls = calls_from_expr(expr);
            calls.extend(calls_from_stmt(body));
            calls
        }
        Stmt::Switch(expr, cases, _) => {
            let mut calls = Vec::new();
            if let Some(e) = expr {
                calls.extend(calls_from_expr(e));
            }
            for case in cases {
                calls.extend(calls_from_case(case));
            }
            calls
        }
        Stmt::Select(cases, _) => cases.iter().flat_map(calls_from_case).collect(),
        Stmt::Return(results, _) => results.iter().flat_map(calls_from_expr).collect(),
        Stmt::Defer(expr, _) | Stmt::Go(expr, _) => calls_from_expr(expr),
        Stmt::Block(block, _) => calls_from_block(block),
        Stmt::Label(_, stmt, _) => calls_from_stmt(stmt),
        Stmt::Send(ch, val, _) => {
            let mut calls = calls_from_expr(ch);
            calls.extend(calls_from_expr(val));
            calls
        }
        _ => vec![],
    }
}

fn calls_from_case(case: &CaseClause) -> Vec<ParsedCall> {
    let mut calls = Vec::new();
    if let Some(expr) = &case.expr {
        calls.extend(calls_from_expr(expr));
    }
    for stmt in &case.body {
        calls.extend(calls_from_stmt(stmt));
    }
    calls
}

fn calls_from_expr(expr: &Expr) -> Vec<ParsedCall> {
    let mut calls = Vec::new();
    match expr {
        Expr::Call(fun, args, span) => {
            let (callee_text, object) = match fun.as_ref() {
                Expr::Ident(name, _) => (name.clone(), None),
                Expr::Selector(obj, field, _) => (field.clone(), Some(expr_to_string(obj))),
                _ => (expr_to_string(fun), None),
            };
            let kind = if object.is_some() {
                CallKind::MethodCall
            } else {
                CallKind::FunctionCall
            };
            calls.extend(calls_from_expr(fun));
            for arg in args {
                calls.extend(calls_from_expr(arg));
            }
            let mut builder =
                ParsedCall::builder(kind, callee_text).pos(span.start.line, span.start.column);
            if let Some(obj) = object {
                builder = builder.object(obj);
            }
            if let Ok(call) = builder.try_build() {
                calls.push(call);
            }
        }
        Expr::Binary(l, _, r, _) => {
            calls.extend(calls_from_expr(l));
            calls.extend(calls_from_expr(r));
        }
        Expr::Unary(_, x, _) => calls.extend(calls_from_expr(x)),
        Expr::Index(obj, idx, _) => {
            calls.extend(calls_from_expr(obj));
            calls.extend(calls_from_expr(idx));
        }
        Expr::Selector(obj, _, _) => calls.extend(calls_from_expr(obj)),
        Expr::Paren(inner, _) => calls.extend(calls_from_expr(inner)),
        Expr::TypeAssert(expr, typ, _) => {
            calls.extend(calls_from_expr(expr));
            calls.extend(calls_from_expr(typ));
        }
        Expr::Slice(expr, from, to, _) => {
            calls.extend(calls_from_expr(expr));
            if let Some(f) = from.as_ref() {
                calls.extend(calls_from_expr(f));
            }
            if let Some(t) = to.as_ref() {
                calls.extend(calls_from_expr(t));
            }
        }
        Expr::Array(items, _) => {
            for item in items {
                calls.extend(calls_from_expr(item));
            }
        }
        Expr::StructLit(_, fields, _) => {
            for f in fields {
                calls_from_field_init(f, &mut calls);
            }
        }
        Expr::MapLit(pairs, _) => {
            for (k, v) in pairs {
                calls.extend(calls_from_expr(k));
                calls.extend(calls_from_expr(v));
            }
        }
        Expr::FuncLit(_, body, _) => {
            calls.extend(calls_from_block(body));
        }
        Expr::CompositeLit(typ, values, _) => {
            calls.extend(calls_from_expr(typ));
            for v in values {
                calls.extend(calls_from_expr(v));
            }
        }
        _ => {}
    }
    calls
}

fn calls_from_field_init(field: &FieldInit, calls: &mut Vec<ParsedCall>) {
    if let Some(val) = &field.value {
        calls.extend(calls_from_expr(val));
    }
}
