use super::ast::*;
use crate::facts::*;
use crate::ExtractionResult;

pub fn extract_facts(program: &Program) -> ExtractionResult {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();
    let mut calls = Vec::new();
    let mut variables = Vec::new();

    walk_top_level(
        &program.decls,
        &mut symbols,
        &mut imports,
        &mut calls,
        &mut variables,
    );

    ExtractionResult {
        symbols,
        imports,
        calls,
        variables,
    }
}

pub fn extract_symbols(program: &Program) -> Vec<ParsedSymbol> {
    extract_facts(program).symbols
}

pub fn extract_imports(program: &Program) -> Vec<ParsedImport> {
    extract_facts(program).imports
}

pub fn extract_calls(program: &Program) -> Vec<ParsedCall> {
    extract_facts(program).calls
}

pub fn extract_variables(program: &Program) -> Vec<ParsedVariable> {
    extract_facts(program).variables
}

/// Conditional directives are retained in the AST, but this extractor does
/// not evaluate preprocessor expressions. Declarations in a conditional
/// region are therefore omitted rather than being reported as active facts.
fn walk_top_level(
    stmts: &[Stmt],
    symbols: &mut Vec<ParsedSymbol>,
    imports: &mut Vec<ParsedImport>,
    calls: &mut Vec<ParsedCall>,
    variables: &mut Vec<ParsedVariable>,
) {
    let mut conditional_depth = 0usize;
    for stmt in stmts {
        if let Stmt::Preprocessor(directive, _) = stmt {
            walk_stmt(stmt, None, symbols, imports, calls, variables);
            match directive {
                PreprocDirective::If(..)
                | PreprocDirective::Ifdef(..)
                | PreprocDirective::Ifndef(..) => {
                    conditional_depth = conditional_depth.saturating_add(1);
                }
                PreprocDirective::Endif(..) => {
                    conditional_depth = conditional_depth.saturating_sub(1);
                }
                _ => {}
            }
        } else if conditional_depth == 0 {
            walk_stmt(stmt, None, symbols, imports, calls, variables);
        }
    }
}

fn walk_stmts(
    stmts: &[Stmt],
    scope_symbol: Option<&str>,
    symbols: &mut Vec<ParsedSymbol>,
    imports: &mut Vec<ParsedImport>,
    calls: &mut Vec<ParsedCall>,
    variables: &mut Vec<ParsedVariable>,
) {
    for stmt in stmts {
        walk_stmt(stmt, scope_symbol, symbols, imports, calls, variables);
    }
}

fn walk_stmt(
    stmt: &Stmt,
    scope_symbol: Option<&str>,
    symbols: &mut Vec<ParsedSymbol>,
    imports: &mut Vec<ParsedImport>,
    calls: &mut Vec<ParsedCall>,
    variables: &mut Vec<ParsedVariable>,
) {
    match stmt {
        Stmt::Decl(fd, span) => {
            extract_func_symbol(fd, span, symbols);
            for param in &fd.params {
                if let Some(name) = &param.name {
                    let type_str = expr_type_to_string(&param.type_);
                    let type_kind = expr_to_type_kind(&param.type_);
                    variables.push(
                        ParsedVariable::builder(name, VarKind::Parameter)
                            .type_ann(Some(type_str))
                            .type_kind(type_kind)
                            .line(span.start.line)
                            .build(),
                    );
                }
            }
            if let Some(body) = &fd.body {
                walk_stmts(
                    &body.stmts,
                    Some(&fd.name),
                    symbols,
                    imports,
                    calls,
                    variables,
                );
            }
        }
        Stmt::VarDecl(vd, span) => {
            extract_var(vd, span, scope_symbol, variables);
            // A `struct Foo { ... };` or `enum E { ... };` definition carries
            // its members inside the type specifier; emit them as facts too.
            if let Expr::DeclSpec(declspec, _) = vd.type_.as_ref() {
                extract_type_members(declspec, Some(vd.name.as_str()), span, symbols, variables);
            }
            if let Some(init) = &vd.init {
                walk_expr_for_calls(init, calls);
            }
        }
        Stmt::Block(block, _) => {
            walk_stmts(
                &block.stmts,
                scope_symbol,
                symbols,
                imports,
                calls,
                variables,
            );
        }
        Stmt::Expr(expr, _) => {
            walk_expr_for_calls(expr, calls);
        }
        Stmt::Return(Some(expr), _) => {
            walk_expr_for_calls(expr, calls);
        }
        Stmt::Return(None, _) => {}
        Stmt::If(cond, body, else_, _) => {
            walk_expr_for_calls(cond, calls);
            walk_stmt(body, scope_symbol, symbols, imports, calls, variables);
            if let Some(e) = else_ {
                walk_stmt(e, scope_symbol, symbols, imports, calls, variables);
            }
        }
        Stmt::Switch(expr, cases, _) => {
            walk_expr_for_calls(expr, calls);
            for case in cases {
                walk_stmts(
                    &case.stmts,
                    scope_symbol,
                    symbols,
                    imports,
                    calls,
                    variables,
                );
            }
        }
        Stmt::While(cond, body, _) => {
            walk_expr_for_calls(cond, calls);
            walk_stmt(body, scope_symbol, symbols, imports, calls, variables);
        }
        Stmt::Do(body, cond, _) => {
            walk_stmt(body, scope_symbol, symbols, imports, calls, variables);
            walk_expr_for_calls(cond, calls);
        }
        Stmt::For(init, cond, post, body, _) => {
            if let Some(init_stmt) = init {
                walk_stmt(init_stmt, scope_symbol, symbols, imports, calls, variables);
            }
            if let Some(cond_expr) = cond {
                walk_expr_for_calls(cond_expr, calls);
            }
            if let Some(post_stmt) = post {
                walk_stmt(post_stmt, scope_symbol, symbols, imports, calls, variables);
            }
            walk_stmt(body, scope_symbol, symbols, imports, calls, variables);
        }
        Stmt::Preprocessor(directive, span) => {
            if let PreprocDirective::Include(path, _) = directive {
                let imp = ParsedImport::builder(ImportKind::NamedImport, path.as_str())
                    .line(span.start.line)
                    .build();
                imports.push(imp);
            }
        }
        Stmt::Break(_)
        | Stmt::Continue(_)
        | Stmt::Goto(_, _)
        | Stmt::Label(_, _)
        | Stmt::Empty(_) => {}
    }
}

fn extract_func_symbol(fd: &FuncDecl, span: &crate::span::Span, symbols: &mut Vec<ParsedSymbol>) {
    let return_type = Some(expr_type_to_string(&fd.return_type));
    let storage = storage_class_from_str(fd.storage_class.as_deref());
    let is_static = fd.storage_class.as_deref() == Some("static");

    let sym = ParsedSymbol::builder(SymbolKind::Function, &fd.name)
        .lines(span.start.line, span.end.line)
        .exported(true)
        .return_type(return_type)
        .storage(storage)
        .static_(is_static)
        .is_test(is_c_test_fn(&fd.name))
        .build();
    symbols.push(sym);
}

/// C test-function heuristic. C has no language-level test attribute; unit-test
/// frameworks (Unity, CUnit, cmocka) conventionally name test functions
/// `test_*` / `Test_*`. gtest-style `TEST(Suite, Name)` is a macro expansion
/// and would not appear as a function declaration.
fn is_c_test_fn(name: &str) -> bool {
    name.starts_with("test_") || name.starts_with("Test_") || name.starts_with("TEST_")
}

fn extract_var(
    vd: &VarDecl,
    span: &crate::span::Span,
    scope_symbol: Option<&str>,
    variables: &mut Vec<ParsedVariable>,
) {
    let kind = if vd.is_const {
        VarKind::Const
    } else {
        VarKind::Let
    };
    let is_mutable = !vd.is_const;
    let storage = storage_class_from_str(vd.storage_class.as_deref());
    let type_kind = expr_to_type_kind(&vd.type_);
    let type_str = Some(expr_type_to_string(&vd.type_));

    let mut builder = ParsedVariable::builder(&vd.name, kind)
        .type_ann(type_str)
        .mutable(is_mutable)
        .line(span.start.line)
        .storage(storage)
        .type_kind(type_kind);

    if let Some(scope) = scope_symbol {
        builder = builder.scope(Some(scope.to_string()), 0, 0);
    }

    variables.push(builder.build());
}

/// Emits a container symbol (`struct`/`union`/`enum`) plus one `field` /
/// `enum_member` variable per member, when the type specifier carries a body.
/// Used for definitions like `struct Foo { int x; };` and `enum E { A, B };`.
fn extract_type_members(
    declspec: &DeclSpec,
    var_name: Option<&str>,
    span: &crate::span::Span,
    symbols: &mut Vec<ParsedSymbol>,
    variables: &mut Vec<ParsedVariable>,
) {
    match declspec {
        DeclSpec::Struct(name, Some(fields)) | DeclSpec::Union(name, Some(fields)) => {
            let container = if name.is_empty() {
                var_name.unwrap_or("").to_string()
            } else {
                name.clone()
            };
            if !container.is_empty() {
                symbols.push(
                    ParsedSymbol::builder(
                        if matches!(declspec, DeclSpec::Union(_, _)) {
                            SymbolKind::Class
                        } else {
                            SymbolKind::Struct
                        },
                        &container,
                    )
                    .lines(span.start.line, span.end.line)
                    .exported(true)
                    .build(),
                );
            }
            for field in fields {
                let mut b = ParsedVariable::builder(&field.name, VarKind::Field)
                    .type_ann(Some(expr_type_to_string(&field.type_)))
                    .type_kind(expr_to_type_kind(&field.type_))
                    .line(field.span.start.line);
                if !container.is_empty() {
                    b = b.scope(Some(container.clone()), 0, 0);
                }
                variables.push(b.build());
            }
        }
        DeclSpec::Enum(name, Some(constants)) => {
            let container = if name.is_empty() {
                var_name.unwrap_or("").to_string()
            } else {
                name.clone()
            };
            if !container.is_empty() {
                symbols.push(
                    ParsedSymbol::builder(SymbolKind::Enum, &container)
                        .lines(span.start.line, span.end.line)
                        .exported(true)
                        .build(),
                );
            }
            for c in constants {
                let mut b =
                    ParsedVariable::builder(&c.name, VarKind::EnumMember).line(c.span.start.line);
                if !container.is_empty() {
                    b = b.scope(Some(container.clone()), 0, 0);
                }
                variables.push(b.build());
            }
        }
        _ => {}
    }
}

fn walk_expr_for_calls(expr: &Expr, calls: &mut Vec<ParsedCall>) {
    match expr {
        Expr::Call(func, args, span) => {
            extract_call(func, span, calls);
            walk_expr_for_calls(func, calls);
            for arg in args {
                walk_expr_for_calls(arg, calls);
            }
        }
        Expr::Binary(left, _, right, _) => {
            walk_expr_for_calls(left, calls);
            walk_expr_for_calls(right, calls);
        }
        Expr::Unary(_, operand, _) => {
            walk_expr_for_calls(operand, calls);
        }
        Expr::Ternary(cond, then, else_, _) => {
            walk_expr_for_calls(cond, calls);
            walk_expr_for_calls(then, calls);
            walk_expr_for_calls(else_, calls);
        }
        Expr::Cast(_, expr, _) => {
            walk_expr_for_calls(expr, calls);
        }
        Expr::Paren(inner, _) => {
            walk_expr_for_calls(inner, calls);
        }
        Expr::Comma(exprs, _) => {
            for e in exprs {
                walk_expr_for_calls(e, calls);
            }
        }
        Expr::Assign(target, value, _) => {
            walk_expr_for_calls(target, calls);
            walk_expr_for_calls(value, calls);
        }
        Expr::Index(base, index, _) => {
            walk_expr_for_calls(base, calls);
            walk_expr_for_calls(index, calls);
        }
        Expr::Member(obj, _, _) => {
            walk_expr_for_calls(obj, calls);
        }
        Expr::Arrow(obj, _, _) => {
            walk_expr_for_calls(obj, calls);
        }
        Expr::Deref(inner, _)
        | Expr::Ref(inner, _)
        | Expr::Sizeof(inner, _)
        | Expr::Alignof(inner, _) => {
            walk_expr_for_calls(inner, calls);
        }
        Expr::StmtExpr(stmts, _) => {
            for stmt in stmts {
                if let super::stmt::Stmt::Expr(e, _) = stmt {
                    walk_expr_for_calls(e, calls);
                }
            }
        }
        Expr::Int(_, _)
        | Expr::UInt(_, _)
        | Expr::Float(_, _)
        | Expr::String(_, _)
        | Expr::Char(_, _)
        | Expr::Ident(_, _)
        | Expr::DeclSpec(_, _)
        | Expr::StringConcat(_, _)
        | Expr::Error(_) => {}
    }
}

fn extract_call(func: &Expr, span: &crate::span::Span, calls: &mut Vec<ParsedCall>) {
    match func {
        Expr::Ident(name, _) => {
            if let Ok(call) = ParsedCall::builder(CallKind::FunctionCall, name.as_str())
                .pos(span.start.line, span.start.column)
                .try_build()
            {
                calls.push(call);
            }
        }
        Expr::Member(obj, method, _) => {
            let obj_str = expr_to_string(obj);
            if let Ok(call) = ParsedCall::builder(CallKind::MethodCall, method.as_str())
                .object(obj_str)
                .pos(span.start.line, span.start.column)
                .try_build()
            {
                calls.push(call);
            }
        }
        Expr::Arrow(obj, method, _) => {
            let obj_str = expr_to_string(obj);
            if let Ok(call) = ParsedCall::builder(CallKind::MethodCall, method.as_str())
                .object(obj_str)
                .pos(span.start.line, span.start.column)
                .try_build()
            {
                calls.push(call);
            }
        }
        _ => {
            let text = expr_to_string(func);
            if let Ok(call) = ParsedCall::builder(CallKind::FunctionCall, text)
                .pos(span.start.line, span.start.column)
                .try_build()
            {
                calls.push(call);
            }
        }
    }
}

fn storage_class_from_str(s: Option<&str>) -> StorageClass {
    match s {
        Some("static") => StorageClass::Static,
        Some("extern") => StorageClass::Extern,
        Some("register") => StorageClass::Register,
        Some("auto") => StorageClass::Auto,
        _ => StorageClass::Unknown,
    }
}

fn expr_to_type_kind(expr: &Expr) -> TypeKind {
    match expr {
        Expr::Ident(name, _) => TypeKind::Simple(name.clone()),
        Expr::Unary(UnaryOp::Deref, inner, _) => {
            TypeKind::Pointer(Box::new(expr_to_type_kind(inner)))
        }
        _ => TypeKind::Unknown,
    }
}

fn expr_type_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Ident(s, _) => s.clone(),
        Expr::Unary(UnaryOp::Deref, inner, _) => {
            format!("{}*", expr_type_to_string(inner))
        }
        _ => String::new(),
    }
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Ident(s, _) => s.clone(),
        Expr::Member(obj, member, _) => format!("{}.{}", expr_to_string(obj), member),
        Expr::Arrow(obj, member, _) => format!("{}->{}", expr_to_string(obj), member),
        Expr::Call(func, _, _) => expr_to_string(func),
        Expr::Paren(inner, _) => format!("({})", expr_to_string(inner)),
        Expr::Unary(_, operand, _) => expr_to_string(operand),
        Expr::Binary(left, _, right, _) => {
            format!("{}{}", expr_to_string(left), expr_to_string(right))
        }
        Expr::Deref(inner, _) => format!("(*{})", expr_to_string(inner)),
        Expr::Ref(inner, _) => format!("(&{})", expr_to_string(inner)),
        Expr::Cast(_, expr, _) => expr_to_string(expr),
        Expr::Index(base, index, _) => {
            format!("{}[{}]", expr_to_string(base), expr_to_string(index))
        }
        _ => String::new(),
    }
}
