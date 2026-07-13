use super::ast::*;
use crate::facts::*;
use crate::ExtractionResult;

pub fn extract_facts(program: &Program) -> ExtractionResult {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();
    let mut calls = Vec::new();
    let mut variables = Vec::new();

    walk_stmts(
        &program.stmts,
        None,
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

fn walk_stmts(
    stmts: &[Stmt],
    class_name: Option<&str>,
    symbols: &mut Vec<ParsedSymbol>,
    imports: &mut Vec<ParsedImport>,
    calls: &mut Vec<ParsedCall>,
    variables: &mut Vec<ParsedVariable>,
) {
    for stmt in stmts {
        walk_stmt(stmt, class_name, symbols, imports, calls, variables);
    }
}

fn walk_stmt(
    stmt: &Stmt,
    class_name: Option<&str>,
    symbols: &mut Vec<ParsedSymbol>,
    imports: &mut Vec<ParsedImport>,
    calls: &mut Vec<ParsedCall>,
    variables: &mut Vec<ParsedVariable>,
) {
    match stmt {
        Stmt::FuncDef(fd, span) => {
            extract_func_symbol(fd, span, class_name, symbols);
            for arg in &fd.args {
                let mut var =
                    ParsedVariable::builder(&arg.name, VarKind::Parameter).line(span.start.line);
                if let Some(ref ann) = arg.type_ann {
                    var = var.type_ann(Some(expr_to_string(ann)));
                }
                variables.push(var.build());
            }
            if let Some(ref vararg) = fd.vararg {
                variables.push(
                    ParsedVariable::builder(&vararg.name, VarKind::Parameter)
                        .line(span.start.line)
                        .type_ann(vararg.type_ann.as_ref().map(|t| expr_to_string(t)))
                        .build(),
                );
            }
            if let Some(ref kwarg) = fd.kwarg {
                variables.push(
                    ParsedVariable::builder(&kwarg.name, VarKind::Parameter)
                        .line(span.start.line)
                        .type_ann(kwarg.type_ann.as_ref().map(|t| expr_to_string(t)))
                        .build(),
                );
            }
            walk_exprs_for_calls(&fd.decorators, calls);
            walk_expr_for_calls_from_type_ann(&fd.returns, calls);
            if let Some(vararg) = &fd.vararg {
                walk_expr_for_calls_from_type_ann(&vararg.type_ann, calls);
            }
            if let Some(kwarg) = &fd.kwarg {
                walk_expr_for_calls_from_type_ann(&kwarg.type_ann, calls);
            }
            walk_exprs_for_calls(&fd.defaults, calls);
            walk_exprs_for_calls(&fd.kw_defaults, calls);
            walk_stmts(&fd.body, class_name, symbols, imports, calls, variables);
        }
        Stmt::ClassDef(cd, span) => {
            let bases: Vec<String> = cd.bases.iter().map(expr_to_string).collect();
            let is_test = is_test_name(&cd.name) || cd.decorators.iter().any(is_pytest_decorator);
            let mut sym = ParsedSymbol::builder(SymbolKind::Class, &cd.name)
                .lines(span.start.line, span.end.line)
                .exported(true)
                .is_test(is_test)
                .build();
            sym.base_classes = bases;
            symbols.push(sym);

            walk_exprs_for_calls(&cd.bases, calls);
            for kw in &cd.keywords {
                walk_expr_for_calls(&kw.value, calls);
            }
            walk_exprs_for_calls(&cd.decorators, calls);

            walk_stmts(&cd.body, Some(&cd.name), symbols, imports, calls, variables);
        }
        Stmt::Import(aliases, span) => {
            for alias in aliases {
                let imp = ParsedImport::builder(ImportKind::PythonImport, &alias.name)
                    .local(alias.asname.as_deref().unwrap_or(&alias.name))
                    .line(span.start.line)
                    .build();
                imports.push(imp);
            }
        }
        Stmt::ImportFrom(module, aliases, _, span) => {
            let source = module.as_deref().unwrap_or("");
            for alias in aliases {
                let imp = ParsedImport::builder(ImportKind::FromImport, source)
                    .imported(&alias.name)
                    .local(alias.asname.as_deref().unwrap_or(&alias.name))
                    .line(span.start.line)
                    .build();
                imports.push(imp);
            }
        }
        Stmt::Assign(target, value, span) => {
            if let Expr::Ident(name, _) = target.as_ref() {
                let var = ParsedVariable::builder(name.as_str(), VarKind::Let)
                    .line(span.start.line)
                    .build();
                variables.push(var);
            }
            walk_expr_for_calls(target, calls);
            walk_expr_for_calls(value, calls);
        }
        Stmt::AnnAssign(target, ann, value, span) => {
            let name = extract_ident_name(target);
            let type_str = expr_to_string(ann);
            if let Some(name) = name {
                let var = ParsedVariable::builder(name, VarKind::Let)
                    .type_ann(Some(type_str))
                    .line(span.start.line)
                    .build();
                variables.push(var);
            }
            walk_expr_for_calls(target, calls);
            walk_expr_for_calls(ann, calls);
            if let Some(v) = value {
                walk_expr_for_calls(v, calls);
            }
        }
        Stmt::For(target, iter, body, _, span) => {
            if let Expr::Ident(name, _) = target.as_ref() {
                let var = ParsedVariable::builder(name.as_str(), VarKind::ForLoop)
                    .line(span.start.line)
                    .build();
                variables.push(var);
            }
            walk_expr_for_calls(target, calls);
            walk_expr_for_calls(iter, calls);
            walk_stmts(body, class_name, symbols, imports, calls, variables);
        }
        Stmt::Return(Some(v), _) => {
            walk_expr_for_calls(v, calls);
        }
        Stmt::Expr(e, _) => {
            walk_expr_for_calls(e, calls);
        }
        Stmt::If(cond, body, else_, _) => {
            walk_expr_for_calls(cond, calls);
            walk_stmts(body, class_name, symbols, imports, calls, variables);
            walk_stmts(else_, class_name, symbols, imports, calls, variables);
        }
        Stmt::While(cond, body, else_, _) => {
            walk_expr_for_calls(cond, calls);
            walk_stmts(body, class_name, symbols, imports, calls, variables);
            if let Some(e) = else_ {
                walk_stmts(e, class_name, symbols, imports, calls, variables);
            }
        }
        Stmt::With(items, body, _) => {
            for item in items {
                walk_expr_for_calls(&item.context, calls);
                if let Some(target) = &item.target {
                    walk_expr_for_calls(target, calls);
                }
            }
            walk_stmts(body, class_name, symbols, imports, calls, variables);
        }
        Stmt::Async(stmt, _) => {
            walk_stmt(stmt, class_name, symbols, imports, calls, variables);
        }
        Stmt::AugAssign(target, _, value, _) => {
            walk_expr_for_calls(target, calls);
            walk_expr_for_calls(value, calls);
        }
        Stmt::Delete(expr, _) => {
            walk_expr_for_calls(expr, calls);
        }
        Stmt::Raise(expr, cause, _) => {
            if let Some(e) = expr {
                walk_expr_for_calls(e, calls);
            }
            if let Some(e) = cause {
                walk_expr_for_calls(e, calls);
            }
        }
        Stmt::Assert(test, msg, _) => {
            walk_expr_for_calls(test, calls);
            if let Some(m) = msg {
                walk_expr_for_calls(m, calls);
            }
        }
        Stmt::Yield(Some(v), _) => {
            walk_expr_for_calls(v, calls);
        }
        Stmt::Match(expr, cases, _) => {
            walk_expr_for_calls(expr, calls);
            for case in cases {
                walk_stmts(&case.body, class_name, symbols, imports, calls, variables);
            }
        }
        Stmt::Try(body, handlers, else_, final_, _) => {
            walk_stmts(body, class_name, symbols, imports, calls, variables);
            for handler in handlers {
                if let Some(t) = &handler.type_ {
                    walk_expr_for_calls(t, calls);
                }
                walk_stmts(
                    &handler.body,
                    class_name,
                    symbols,
                    imports,
                    calls,
                    variables,
                );
            }
            if let Some(e) = else_ {
                walk_stmts(e, class_name, symbols, imports, calls, variables);
            }
            if let Some(f) = final_ {
                walk_stmts(f, class_name, symbols, imports, calls, variables);
            }
        }
        Stmt::Block(body, _) => {
            walk_stmts(body, class_name, symbols, imports, calls, variables);
        }
        _ => {}
    }
}

fn extract_func_symbol(
    fd: &FuncDef,
    span: &crate::span::Span,
    class_name: Option<&str>,
    symbols: &mut Vec<ParsedSymbol>,
) {
    let is_test = is_test_name(&fd.name) || fd.decorators.iter().any(is_pytest_decorator);

    let return_type = fd.returns.as_ref().map(|r| expr_to_string(r));

    let is_constructor = class_name.is_some() && fd.name == "__init__";
    let is_destructor = class_name.is_some() && fd.name == "__del__";

    let kind = if is_constructor {
        SymbolKind::Constructor
    } else if is_destructor {
        SymbolKind::Destructor
    } else {
        SymbolKind::Function
    };

    let sym = ParsedSymbol::builder(kind, &fd.name)
        .lines(span.start.line, span.end.line)
        .exported(true)
        .is_test(is_test)
        .is_async(fd.is_async)
        .return_type(return_type)
        .constructor(is_constructor)
        .destructor(is_destructor)
        .build();
    symbols.push(sym);
}

fn walk_expr_for_calls(expr: &Expr, calls: &mut Vec<ParsedCall>) {
    match expr {
        Expr::Call(func, args, keywords, span) => {
            extract_call(func, span, calls);
            walk_expr_for_calls(func, calls);
            for arg in args {
                walk_expr_for_calls(arg, calls);
            }
            for kw in keywords {
                walk_expr_for_calls(&kw.value, calls);
            }
        }
        Expr::Attribute(obj, _, _) => {
            walk_expr_for_calls(obj, calls);
        }
        Expr::Binary(left, _, right, _) => {
            walk_expr_for_calls(left, calls);
            walk_expr_for_calls(right, calls);
        }
        Expr::Unary(_, operand, _) => {
            walk_expr_for_calls(operand, calls);
        }
        Expr::IfElse(cond, then, else_, _) => {
            walk_expr_for_calls(cond, calls);
            walk_expr_for_calls(then, calls);
            walk_expr_for_calls(else_, calls);
        }
        Expr::Subscript(base, slice, _) => {
            walk_expr_for_calls(base, calls);
            walk_expr_for_calls(slice, calls);
        }
        Expr::Slice(start, stop, step, _) => {
            if let Some(s) = start {
                walk_expr_for_calls(s, calls);
            }
            if let Some(s) = stop {
                walk_expr_for_calls(s, calls);
            }
            if let Some(s) = step {
                walk_expr_for_calls(s, calls);
            }
        }
        Expr::Await(inner, _) => {
            walk_expr_for_calls(inner, calls);
        }
        Expr::YieldFrom(inner, _) => {
            walk_expr_for_calls(inner, calls);
        }
        Expr::Starred(inner, _) => {
            walk_expr_for_calls(inner, calls);
        }
        Expr::Walrus(target, value, _) => {
            walk_expr_for_calls(target, calls);
            walk_expr_for_calls(value, calls);
        }
        Expr::Compare(left, _, comparators, _) => {
            walk_expr_for_calls(left, calls);
            for c in comparators {
                walk_expr_for_calls(c, calls);
            }
        }
        Expr::Paren(inner, _) => {
            walk_expr_for_calls(inner, calls);
        }
        Expr::List(items, _) | Expr::Tuple(items, _) | Expr::Set(items, _) => {
            for item in items {
                walk_expr_for_calls(item, calls);
            }
        }
        Expr::Dict(items, _) => {
            for (k, v) in items {
                walk_expr_for_calls(k, calls);
                walk_expr_for_calls(v, calls);
            }
        }
        Expr::ListComp(elt, generators, _)
        | Expr::SetComp(elt, generators, _)
        | Expr::Generator(elt, generators, _) => {
            walk_expr_for_calls(elt, calls);
            for r#gen in generators {
                walk_expr_for_calls(&r#gen.target, calls);
                walk_expr_for_calls(&r#gen.iter, calls);
                for if_expr in &r#gen.ifs {
                    walk_expr_for_calls(if_expr, calls);
                }
            }
        }
        Expr::DictComp(key, generators, _) => {
            walk_expr_for_calls(key, calls);
            for r#gen in generators {
                walk_expr_for_calls(&r#gen.target, calls);
                walk_expr_for_calls(&r#gen.iter, calls);
                for if_expr in &r#gen.ifs {
                    walk_expr_for_calls(if_expr, calls);
                }
            }
        }
        Expr::FString(parts, _) => {
            for part in parts {
                if let FStringPart::Expr(e, _) = part {
                    walk_expr_for_calls(e, calls);
                }
            }
        }
        Expr::Lambda(_, body, _) => {
            walk_expr_for_calls(body, calls);
        }
        Expr::Match(expr, _cases, _) => {
            walk_expr_for_calls(expr, calls);
        }
        Expr::Yield(inner, _) => {
            if let Some(e) = inner {
                walk_expr_for_calls(e, calls);
            }
        }
        Expr::Ident(_, _) | Expr::Literal(_, _) | Expr::Ellipsis(_) | Expr::Error(_) => {}
    }
}

fn walk_exprs_for_calls(exprs: &[Expr], calls: &mut Vec<ParsedCall>) {
    for expr in exprs {
        walk_expr_for_calls(expr, calls);
    }
}

fn walk_expr_for_calls_from_type_ann(ann: &Option<Box<Expr>>, calls: &mut Vec<ParsedCall>) {
    if let Some(e) = ann {
        walk_expr_for_calls(e, calls);
    }
}

fn extract_call(func: &Expr, span: &crate::span::Span, calls: &mut Vec<ParsedCall>) {
    match func {
        Expr::Ident(name, _) => {
            let call = ParsedCall::builder(CallKind::FunctionCall, name.as_str())
                .pos(span.start.line, span.start.column)
                .build();
            calls.push(call);
        }
        Expr::Attribute(obj, attr, _) => {
            let object_str = expr_to_string(obj);
            let call = ParsedCall::builder(CallKind::MethodCall, attr.as_str())
                .object(object_str)
                .pos(span.start.line, span.start.column)
                .build();
            calls.push(call);
        }
        _ => {
            let text = expr_to_string(func);
            if !text.is_empty() {
                let call = ParsedCall::builder(CallKind::FunctionCall, text)
                    .pos(span.start.line, span.start.column)
                    .build();
                calls.push(call);
            }
        }
    }
}

fn extract_ident_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Ident(name, _) => Some(name.as_str()),
        _ => None,
    }
}

fn expr_to_string(expr: &Expr) -> String {
    match expr {
        Expr::Ident(s, _) => s.clone(),
        Expr::Attribute(obj, attr, _) => format!("{}.{}", expr_to_string(obj), attr),
        Expr::Call(func, _, _, _) => expr_to_string(func),
        Expr::Subscript(base, slice, _) => {
            format!("{}[{}]", expr_to_string(base), expr_to_string(slice))
        }
        Expr::Binary(left, _, right, _) => {
            format!("{}{}", expr_to_string(left), expr_to_string(right))
        }
        Expr::Unary(_, operand, _) => expr_to_string(operand),
        Expr::List(items, _) => {
            let inner: Vec<String> = items.iter().map(expr_to_string).collect();
            format!("[{}]", inner.join(", "))
        }
        Expr::Tuple(items, _) => {
            let inner: Vec<String> = items.iter().map(expr_to_string).collect();
            format!("({})", inner.join(", "))
        }
        Expr::Starred(inner, _) => format!("*{}", expr_to_string(inner)),
        Expr::Await(inner, _) => format!("await {}", expr_to_string(inner)),
        Expr::Paren(inner, _) => format!("({})", expr_to_string(inner)),
        _ => String::new(),
    }
}

fn is_test_name(name: &str) -> bool {
    name.starts_with("test_")
}

fn is_pytest_decorator(expr: &Expr) -> bool {
    let s = expr_to_string(expr);
    s == "pytest" || s.starts_with("pytest.")
}
